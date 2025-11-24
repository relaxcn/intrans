// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use caret;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn toggle_main_window(app: tauri::AppHandle) {
    use tauri_plugin_positioner::{Position, WindowExt};

    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = app.emit("session-ended", ());
            let _ = window.hide();
        } else {
            // 先获取光标位置（在窗口成为前台之前）
            let caret_pos = caret::get_position();
            
            let _ = window.unminimize();

            if let Some(pos) = caret_pos {
                tracing::debug!(x = pos.x, y = pos.y, "设置窗口位置到光标处");
                let _ = window.set_position(tauri::Position::Physical(pos.into()));
            } else {
                tracing::warn!("所有光标获取方法都失败，将使用屏幕中央");
                let _ = window
                    .as_ref()
                    .window()
                    .move_window(Position::Center);
            }

            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![greet, toggle_main_window])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label().as_ref() {
                    "main" => {
                        let _ = window.app_handle().emit("session-ended", ());
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    "settings" => {
                        let _ = window.hide();
                        api.prevent_close();
                    }
                    _ => {}
                }
            }
        })
        .setup(|app| {
            // 初始化光标位置获取模块（提前启动 Windows MSAA 钩子等）
            caret::init();

            #[cfg(desktop)]
            {
                app.handle().plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            }

            let settings_item =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_item, &quit_item])?;

            TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "settings" => {
                        if let Some(window) = app.get_webview_window("settings") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);

                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}