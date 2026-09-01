use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};

pub type ExitFlag = Arc<AtomicBool>;

pub fn new_exit_flag() -> ExitFlag {
    Arc::new(AtomicBool::new(false))
}

pub fn create_tray(app: &AppHandle, exit_flag: ExitFlag) -> Result<(), String> {
    let open = MenuItem::with_id(app, "open", "打开", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let clear = MenuItem::with_id(app, "clear", "清空历史记录", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let menu = Menu::with_items(app, &[&open, &clear, &quit])
        .map_err(|error| format!("创建托盘菜单失败：{error}"))?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "缺少应用图标".to_string())?;

    let clear_app = app.clone();
    TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .icon(icon)
        .tooltip("码到成功")
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "open" => show_main(app),
            "clear" => {
                show_main(app);
                if let Err(error) = clear_app.emit("request-clear-history", ()) {
                    log::warn!("clear history event failed: {error}");
                }
            }
            "quit" => {
                exit_flag.store(true, Ordering::SeqCst);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(
                event,
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                }
            ) {
                show_main(tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| format!("创建托盘图标失败：{error}"))?;
    Ok(())
}

pub fn install_close_handling(app: &AppHandle, exit_flag: ExitFlag) {
    let main = app.get_webview_window("main");
    if let Some(main) = main {
        let window = main.clone();
        main.on_window_event(move |event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if !exit_flag.load(Ordering::SeqCst) {
                    api.prevent_close();
                    if let Err(error) = window.hide() {
                        log::warn!("hide main window failed: {error}");
                    }
                }
            }
        });
    }
}

pub fn show_main(app: &AppHandle) {
    if let Some(main) = app.get_webview_window("main") {
        if let Err(error) = main.show() {
            log::warn!("show main window failed: {error}");
            return;
        }
        if let Err(error) = main.unminimize() {
            log::warn!("unminimize main window failed: {error}");
        }
        if let Err(error) = main.set_focus() {
            log::warn!("focus main window failed: {error}");
        }
    }
}
