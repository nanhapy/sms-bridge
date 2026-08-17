mod code;
pub mod commands;
pub mod desktop;
pub mod model;
pub mod receiver;
pub mod runtime;
pub mod startup;
pub mod storage;

use runtime::AppRuntime;
use std::sync::Arc;
use storage::Storage;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            desktop::show_main(app);
        }))
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_snapshot,
            commands::copy_text,
            commands::clear_history,
            commands::get_autostart,
            commands::set_autostart,
        ])
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let storage = Storage::new(data_dir);
            let (history, storage_warning) = storage.load_history();
            let autostart_enabled = startup::initialize(&app.handle(), &storage)?;
            let runtime = Arc::new(AppRuntime::new(
                app.handle().clone(),
                storage,
                history,
                autostart_enabled,
                storage_warning,
            ));
            app.manage(runtime.clone());
            let exit_flag = desktop::new_exit_flag();
            desktop::create_tray(&app.handle(), exit_flag.clone())?;
            desktop::install_close_handling(&app.handle(), exit_flag);
            receiver::start(runtime);
            if !std::env::args().any(|argument| argument == "--autostart") {
                desktop::show_main(&app.handle());
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
