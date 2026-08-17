use crate::model::HistoryRecord;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const HISTORY_LIMIT: usize = 15;

#[derive(Clone)]
pub struct Storage {
    data_dir: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub autostart_initialized: bool,
}

#[derive(Clone, Debug)]
pub enum ConfigLoad {
    Missing,
    Loaded(AppConfig),
    Recovered(AppConfig, String),
}

impl Storage {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn load_history(&self) -> (Vec<HistoryRecord>, Option<String>) {
        let path = self.history_path();
        if let Err(warning) = self.promote_valid_temp(&path, "history", |bytes| {
            serde_json::from_slice::<Vec<HistoryRecord>>(bytes).map(|_| ())
        }) {
            return (Vec::new(), Some(warning));
        }

        match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<Vec<HistoryRecord>>(&bytes) {
                Ok(records) => (trim_history(records), None),
                Err(_) => (
                    Vec::new(),
                    Some(self.recover_corrupt(&path, "history", "历史记录文件已损坏，已重置")),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Vec::new(), None),
            Err(error) => (Vec::new(), Some(format!("读取历史记录失败：{error}"))),
        }
    }

    pub fn save_history(&self, records: &[HistoryRecord]) -> Result<(), String> {
        let records = trim_history(records.to_vec());
        self.write_json(&self.history_path(), &records)
    }

    pub fn clear_history(&self) -> Result<(), String> {
        let path = self.history_path();
        let temp = path.with_extension("tmp");
        let mut errors = Vec::new();

        for file in [path, temp] {
            if let Err(error) = fs::remove_file(file) {
                if error.kind() != std::io::ErrorKind::NotFound {
                    errors.push(error.to_string());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("清除历史记录失败：{}", errors.join("；")))
        }
    }

    pub fn load_config(&self) -> ConfigLoad {
        let path = self.config_path();
        if let Err(warning) = self.promote_valid_temp(&path, "config", |bytes| {
            serde_json::from_slice::<AppConfig>(bytes).map(|_| ())
        }) {
            return ConfigLoad::Recovered(
                AppConfig {
                    autostart_initialized: true,
                },
                warning,
            );
        }

        match fs::read(&path) {
            Ok(bytes) => match serde_json::from_slice::<AppConfig>(&bytes) {
                Ok(config) => ConfigLoad::Loaded(config),
                Err(_) => ConfigLoad::Recovered(
                    AppConfig {
                        autostart_initialized: true,
                    },
                    self.recover_corrupt(&path, "config", "配置文件已损坏，已恢复为安全设置"),
                ),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => ConfigLoad::Missing,
            Err(error) => ConfigLoad::Recovered(
                AppConfig {
                    autostart_initialized: true,
                },
                format!("读取配置失败：{error}"),
            ),
        }
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        self.write_json(&self.config_path(), config)
    }

    fn history_path(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }

    fn config_path(&self) -> PathBuf {
        self.data_dir.join("config.json")
    }

    fn write_json<T: Serialize>(&self, destination: &Path, value: &T) -> Result<(), String> {
        fs::create_dir_all(&self.data_dir).map_err(|error| format!("创建应用数据目录失败：{error}"))?;
        let bytes = serde_json::to_vec(value).map_err(|error| format!("序列化数据失败：{error}"))?;
        let temp = destination.with_extension("tmp");
        let mut file = File::create(&temp).map_err(|error| format!("写入临时文件失败：{error}"))?;
        file.write_all(&bytes)
            .map_err(|error| format!("写入临时文件失败：{error}"))?;
        file.sync_all()
            .map_err(|error| format!("刷新临时文件失败：{error}"))?;
        drop(file);
        fs::rename(&temp, destination).map_err(|error| format!("替换数据文件失败：{error}"))
    }

    fn promote_valid_temp<F>(&self, destination: &Path, kind: &str, parse: F) -> Result<(), String>
    where
        F: FnOnce(&[u8]) -> Result<(), serde_json::Error>,
    {
        if destination.exists() {
            return Ok(());
        }

        let temp = destination.with_extension("tmp");
        let bytes = match fs::read(&temp) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(format!("读取临时数据文件失败：{error}")),
        };

        if parse(&bytes).is_ok() {
            fs::create_dir_all(&self.data_dir)
                .map_err(|error| format!("创建应用数据目录失败：{error}"))?;
            fs::rename(&temp, destination).map_err(|error| format!("恢复临时数据文件失败：{error}"))?;
            return Ok(());
        }

        Err(self.recover_corrupt(&temp, kind, "临时数据文件已损坏，已重置"))
    }

    fn recover_corrupt(&self, path: &Path, kind: &str, message: &str) -> String {
        let epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let corrupt = self.data_dir.join(format!("{kind}.corrupt-{epoch}.json"));
        match fs::rename(path, corrupt) {
            Ok(()) => message.to_string(),
            Err(error) => format!("{message}，但无法保留损坏文件：{error}"),
        }
    }
}

fn trim_history(mut records: Vec<HistoryRecord>) -> Vec<HistoryRecord> {
    records.truncate(HISTORY_LIMIT);
    records
}
