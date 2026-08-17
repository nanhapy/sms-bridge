use crate::storage::{AppConfig, ConfigLoad, Storage};
use std::{env, io};
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;
#[cfg(windows)]
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE},
    RegKey,
};

#[cfg(windows)]
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";

pub fn initialize(app: &AppHandle, storage: &Storage) -> Result<bool, String> {
    match storage.load_config() {
        ConfigLoad::Missing => {
            let enabled = set_enabled(app, true)?;
            if !enabled {
                return Err("启用开机自启动后未在 Windows 中生效".to_string());
            }
            storage.save_config(&AppConfig {
                autostart_initialized: true,
            })?;
            Ok(enabled)
        }
        ConfigLoad::Loaded(_) => is_enabled(app),
        ConfigLoad::Recovered(mut config, warning) => {
            let enabled = is_enabled(app);
            config.autostart_initialized = true;
            storage
                .save_config(&config)
                .map_err(|error| format!("保存恢复的配置失败：{warning}；{error}"))?;
            log::warn!("configuration recovery completed: {warning}");
            enabled
        }
    }
}

pub fn set_enabled(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let autostart = app.autolaunch();
    if enabled {
        autostart
            .enable()
            .map_err(|error| format!("启用开机自启动失败：{error}"))?;
        write_expected_run_value(app)?;
    } else {
        if let Err(error) = autostart.disable() {
            if is_disabled(app)? {
                return Ok(false);
            }
            return Err(format!("关闭开机自启动失败：{error}"));
        }
    }
    let verified = is_enabled(app)?;
    if enabled {
        if !verified {
            return Err("启用开机自启动后未在 Windows 中生效".to_string());
        }
        return Ok(true);
    }
    if verified || !is_disabled(app)? {
        return Err("关闭开机自启动后仍在 Windows 中生效".to_string());
    }
    Ok(false)
}

pub fn is_enabled(app: &AppHandle) -> Result<bool, String> {
    let plugin_enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| format!("读取开机自启动状态失败：{error}"))?;
    #[cfg(windows)]
    {
        let run_value_matches = run_value_matches(app)?;
        Ok(plugin_enabled && run_value_matches)
    }
    #[cfg(not(windows))]
    {
        Ok(plugin_enabled)
    }
}

fn is_disabled(app: &AppHandle) -> Result<bool, String> {
    let plugin_enabled = app
        .autolaunch()
        .is_enabled()
        .map_err(|error| format!("读取开机自启动状态失败：{error}"))?;
    #[cfg(windows)]
    {
        Ok(!plugin_enabled && !run_value_exists(app)?)
    }
    #[cfg(not(windows))]
    {
        Ok(!plugin_enabled)
    }
}

#[cfg(windows)]
fn write_expected_run_value(app: &AppHandle) -> Result<(), String> {
    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let (run_key, _) = current_user
        .create_subkey_with_flags(RUN_KEY, KEY_SET_VALUE)
        .map_err(|error| format!("修正 Windows 开机启动项失败：{error}"))?;
    let command = expected_run_command()?;
    run_key
        .set_value(app.package_info().name.as_str(), &command)
        .map_err(|error| format!("修正 Windows 开机启动项失败：{error}"))
}

#[cfg(not(windows))]
fn write_expected_run_value(_: &AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(windows)]
fn run_value_matches(app: &AppHandle) -> Result<bool, String> {
    let expected = expected_run_command()?;
    Ok(read_run_value(app)?.as_deref() == Some(expected.as_str()))
}

#[cfg(windows)]
fn run_value_exists(app: &AppHandle) -> Result<bool, String> {
    Ok(read_run_value(app)?.is_some())
}

#[cfg(windows)]
fn read_run_value(app: &AppHandle) -> Result<Option<String>, String> {
    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let run_key = match current_user.open_subkey_with_flags(RUN_KEY, KEY_READ) {
        Ok(run_key) => run_key,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("读取 Windows 开机启动项失败：{error}")),
    };
    match run_key.get_value::<String, _>(app.package_info().name.as_str()) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("读取 Windows 开机启动项失败：{error}")),
    }
}

#[cfg(windows)]
fn expected_run_command() -> Result<String, String> {
    let executable = env::current_exe().map_err(|error| format!("解析应用程序路径失败：{error}"))?;
    Ok(format!("\"{}\" --autostart", executable.display()))
}
