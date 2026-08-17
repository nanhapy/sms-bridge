use crate::{model::AppSnapshot, runtime::AppRuntime};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_clipboard_manager::ClipboardExt;

#[tauri::command]
pub async fn get_snapshot(state: State<'_, Arc<AppRuntime>>) -> Result<AppSnapshot, String> {
    Ok(state.snapshot().await)
}

#[tauri::command]
pub fn copy_text(app: AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(text)
        .map_err(|error| format!("写入剪贴板失败：{error}"))
}

#[tauri::command]
pub async fn clear_history(state: State<'_, Arc<AppRuntime>>) -> Result<AppSnapshot, String> {
    state.clear_history().await?;
    Ok(state.snapshot().await)
}

#[tauri::command]
pub async fn get_autostart(
    app: AppHandle,
    state: State<'_, Arc<AppRuntime>>,
) -> Result<bool, String> {
    let _autostart_operation = state.autostart_operation().await;
    let enabled = crate::startup::is_enabled(&app)?;
    state.set_autostart_state(enabled).await;
    Ok(enabled)
}

#[tauri::command]
pub async fn set_autostart(
    app: AppHandle,
    state: State<'_, Arc<AppRuntime>>,
    enabled: bool,
) -> Result<bool, String> {
    let _autostart_operation = state.autostart_operation().await;
    let verified = crate::startup::set_enabled(&app, enabled)?;
    state.set_autostart_state(verified).await;
    Ok(verified)
}
