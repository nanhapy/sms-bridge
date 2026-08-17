use crate::{
    code::{extract_code, valid_explicit_code},
    model::{AppSnapshot, HistoryRecord, PushPayload, PushResult, ReceiverStatus},
    storage::Storage,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_notification::NotificationExt;
use tokio::sync::{Mutex, MutexGuard, RwLock};

const HISTORY_LIMIT: usize = 15;
const REPEAT_WINDOW_SECONDS: u64 = 60;

pub struct AppRuntime {
    app: AppHandle,
    storage: Storage,
    history: RwLock<Vec<HistoryRecord>>,
    receiver_status: RwLock<ReceiverStatus>,
    autostart_enabled: RwLock<bool>,
    storage_warning: RwLock<Option<String>>,
    last_recognized: RwLock<Option<(String, Instant)>>,
    history_operation: Mutex<()>,
    receiver_status_operation: Mutex<()>,
    autostart_operation: Mutex<()>,
}

impl AppRuntime {
    pub fn new(
        app: AppHandle,
        storage: Storage,
        history: Vec<HistoryRecord>,
        autostart_enabled: bool,
        storage_warning: Option<String>,
    ) -> Self {
        Self {
            app,
            storage,
            history: RwLock::new(history),
            receiver_status: RwLock::new(ReceiverStatus::Starting),
            autostart_enabled: RwLock::new(autostart_enabled),
            storage_warning: RwLock::new(storage_warning),
            last_recognized: RwLock::new(None),
            history_operation: Mutex::new(()),
            receiver_status_operation: Mutex::new(()),
            autostart_operation: Mutex::new(()),
        }
    }

    pub async fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            history: self.history.read().await.clone(),
            receiver_status: self.receiver_status.read().await.clone(),
            autostart_enabled: *self.autostart_enabled.read().await,
            storage_warning: self.storage_warning.read().await.clone(),
        }
    }

    pub async fn set_receiver_status(&self, status: ReceiverStatus) {
        let _receiver_status_operation = self.receiver_status_operation.lock().await;
        *self.receiver_status.write().await = status;
        self.emit_snapshot().await;
    }

    pub async fn set_listening_unless_degraded(&self, port: u16) -> bool {
        let _receiver_status_operation = self.receiver_status_operation.lock().await;
        if matches!(
            *self.receiver_status.read().await,
            ReceiverStatus::Degraded { .. }
        ) {
            return false;
        }
        *self.receiver_status.write().await = ReceiverStatus::Listening { port };
        self.emit_snapshot().await;
        true
    }

    pub async fn set_degraded_unless_unavailable(&self, port: u16, message: String) -> bool {
        let _receiver_status_operation = self.receiver_status_operation.lock().await;
        if matches!(
            *self.receiver_status.read().await,
            ReceiverStatus::Unavailable { .. }
        ) {
            return false;
        }
        *self.receiver_status.write().await = ReceiverStatus::Degraded { port, message };
        self.emit_snapshot().await;
        true
    }

    pub async fn accept_push(&self, payload: PushPayload) -> PushResult {
        let _history_operation = self.history_operation.lock().await;
        let text = payload
            .text
            .unwrap_or_default()
            .chars()
            .take(500)
            .collect::<String>();
        let code = payload
            .code
            .as_deref()
            .and_then(valid_explicit_code)
            .or_else(|| extract_code(&text));

        if let Some(code) = code.as_ref() {
            let mut last_recognized = self.last_recognized.write().await;
            if let Some((last_code, last_time)) = last_recognized.as_ref() {
                if last_code == code && last_time.elapsed().as_secs() < REPEAT_WINDOW_SECONDS {
                    return PushResult {
                        ok: true,
                        code: Some(code.clone()),
                        repeated: true,
                        message: "重复推送，已忽略".to_string(),
                    };
                }
            }
            *last_recognized = Some((code.clone(), Instant::now()));
        }

        let record = HistoryRecord {
            received_at: unix_millis(),
            text: text.clone(),
            code: code.clone(),
            source: payload.source.unwrap_or_else(|| "unknown".to_string()),
        };
        let records = {
            let mut history = self.history.write().await;
            history.insert(0, record);
            history.truncate(HISTORY_LIMIT);
            history.clone()
        };

        match self.storage.save_history(&records) {
            Ok(()) => *self.storage_warning.write().await = None,
            Err(error) => *self.storage_warning.write().await = Some(error),
        }

        if let Some(code) = code.as_ref() {
            if let Err(error) = self.app.clipboard().write_text(code.clone()) {
                log::warn!("clipboard write failed: {error}");
            }
        }

        let title = code
            .as_ref()
            .map(|code| format!("验证码 {code}"))
            .unwrap_or_else(|| "收到短信（未识别出验证码）".to_string());
        let body = if text.is_empty() { "(内容为空)" } else { &text };
        if let Err(error) = self.app.notification().builder().title(title).body(body).show() {
            log::warn!("notification failed: {error}");
        }

        let snapshot = self.snapshot().await;
        if let Err(error) = self.app.emit("snapshot-updated", snapshot) {
            log::warn!("snapshot event failed: {error}");
        }

        PushResult {
            ok: true,
            code: code.clone(),
            repeated: false,
            message: if code.is_some() {
                "已弹窗并复制到剪贴板".to_string()
            } else {
                "未识别出验证码，仅弹窗".to_string()
            },
        }
    }

    pub async fn clear_history(&self) -> Result<(), String> {
        let _history_operation = self.history_operation.lock().await;
        match self.storage.clear_history() {
            Ok(()) => {
                self.history.write().await.clear();
                *self.storage_warning.write().await = None;
                self.emit_snapshot().await;
                Ok(())
            }
            Err(error) => {
                *self.storage_warning.write().await = Some(error.clone());
                Err(error)
            }
        }
    }

    pub async fn set_autostart_state(&self, enabled: bool) {
        *self.autostart_enabled.write().await = enabled;
        self.emit_snapshot().await;
    }

    pub async fn autostart_operation(&self) -> MutexGuard<'_, ()> {
        self.autostart_operation.lock().await
    }

    async fn emit_snapshot(&self) {
        let snapshot = self.snapshot().await;
        if let Err(error) = self.app.emit("snapshot-updated", snapshot) {
            log::warn!("snapshot event failed: {error}");
        }
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
