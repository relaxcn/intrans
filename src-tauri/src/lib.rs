// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, PhysicalPosition, PhysicalSize, WindowEvent,
};
use caret;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn inline_shell_safe_position(
    app: &tauri::AppHandle,
    desired: PhysicalPosition<i32>,
    window_size: PhysicalSize<u32>,
) -> PhysicalPosition<i32> {
    const MARGIN: i32 = 12;
    const GAP_BELOW: i32 = 12; // 光标下方的默认间距
    const GAP_ABOVE_EXTRA: i32 = 30; // 上方额外间距，避免贴得过近

    let monitors = app.available_monitors().ok();

    let target_monitor = monitors
        .as_ref()
        .and_then(|list| {
            list.iter().find(|m| {
                let pos = m.position();
                let size = m.size();
                desired.x >= pos.x
                    && desired.x <= pos.x + size.width as i32
                    && desired.y >= pos.y
                    && desired.y <= pos.y + size.height as i32
            })
        })
        .cloned()
        .or_else(|| monitors.and_then(|mut list| list.pop()))
        .or_else(|| app.primary_monitor().ok().flatten());

    if let Some(monitor) = target_monitor {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();

        let min_x = monitor_pos.x + MARGIN;
        let min_y = monitor_pos.y + MARGIN;
        let max_x = monitor_pos.x + monitor_size.width as i32 - window_size.width as i32 - MARGIN;
        let max_y = monitor_pos.y + monitor_size.height as i32 - window_size.height as i32 - MARGIN;

        // 水平尽量以光标为中心，对齐后再做边界夹紧
        let center_x = desired.x - (window_size.width as i32 / 2);
        let safe_x = if max_x < min_x {
            min_x
        } else {
            center_x.clamp(min_x, max_x)
        };

        // 优先放在光标下方（留 GAP_BELOW），若不够空间则放上方（留窗口高度 + MARGIN + GAP_ABOVE_EXTRA）
        let below_y = desired.y + GAP_BELOW;
        let safe_y = if max_y < min_y {
            min_y
        } else if below_y <= max_y {
            below_y.clamp(min_y, max_y)
        } else {
            let above_y = desired.y - window_size.height as i32 - MARGIN - GAP_ABOVE_EXTRA;
            above_y.clamp(min_y, max_y)
        };

        PhysicalPosition::new(safe_x, safe_y)
    } else {
        // 无法获取显示器信息时，简单放在光标下方并避免负值
        PhysicalPosition::new(desired.x.max(0), (desired.y + GAP_BELOW).max(0))
    }
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
            let window_size = window
                .outer_size()
                .unwrap_or_else(|_| PhysicalSize::new(700, 400));
            
            let _ = window.unminimize();

            if let Some(pos) = caret_pos {
                let desired = PhysicalPosition::new(pos.x, pos.y);
                let safe_pos = inline_shell_safe_position(&app, desired, window_size);

                tracing::debug!(x = safe_pos.x, y = safe_pos.y, "设置窗口位置到光标处并进行边界保护");
                let _ = window.set_position(tauri::Position::Physical(safe_pos));
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
                        // 打开设置时隐藏 main 窗口
                        if let Some(main_win) = app.get_webview_window("main") {
                            if main_win.is_visible().unwrap_or(false) {
                                let _ = app.emit("session-ended", ());
                                let _ = main_win.hide();
                            }
                        }
                        
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
