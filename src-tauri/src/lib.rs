// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .invoke_handler(tauri::generate_handler![greet])
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
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };
                use tauri_plugin_positioner::{Position, WindowExt};

                let toggle_main =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Space);
                let open_settings =
                    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::SHIFT), Code::Comma);

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler({
                            let toggle_main = toggle_main.clone();
                            let open_settings = open_settings.clone();
                            move |app, shortcut, event| {
                                use tauri::Manager;

                                if !matches!(event.state(), ShortcutState::Released) {
                                    return;
                                }

                                if shortcut == &toggle_main {
                                    if let Some(window) = app.get_webview_window("main") {
                                        if window.is_visible().unwrap_or(false) {
                                            let _ = app.emit("session-ended", ());
                                            let _ = window.hide();
                                        } else {
                                            let _ = window.unminimize();
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                            if let Some(pos) = get_caret_position() {
                                                let _ = window.set_position(tauri::Position::Physical(pos));
                                            } else {
                                                let _ = window
                                                    .as_ref()
                                                    .window()
                                                    .move_window(Position::Center);
                                            }
                                        }
                                    }
                                } else if shortcut == &open_settings {
                                    if let Some(window) = app.get_webview_window("settings") {
                                        let _ = window.unminimize();
                                        let _ = window.show();
                                        let _ = window.set_focus();
                                        let _ = window.center();
                                    }
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(toggle_main)?;
                app.global_shortcut().register(open_settings)?;
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

#[cfg(target_os = "windows")]
fn get_caret_position() -> Option<tauri::PhysicalPosition<i32>> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::Graphics::Gdi::ClientToScreen;
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetGUIThreadInfo, GetWindowThreadProcessId, GUITHREADINFO,
    };

    unsafe {
        let foreground_window = GetForegroundWindow();
        if foreground_window.0.is_null() {
            return None;
        }

        let mut process_id = 0;
        let thread_id = GetWindowThreadProcessId(foreground_window, Some(&mut process_id));

        let mut gui_info = GUITHREADINFO::default();
        gui_info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;

        if GetGUIThreadInfo(thread_id, &mut gui_info).is_ok() {
            let caret_rect = gui_info.rcCaret;

            if caret_rect.right == 0 && caret_rect.bottom == 0 {
                return None;
            }

            let mut point = POINT {
                x: caret_rect.left,
                y: caret_rect.bottom,
            };

            if !gui_info.hwndCaret.0.is_null() {
                if ClientToScreen(gui_info.hwndCaret, &mut point).as_bool() {
                    return Some(tauri::PhysicalPosition {
                        x: point.x,
                        y: point.y,
                    });
                }
            }
        }
    }
    None
}

// TODO: implement for macos
#[cfg(target_os = "macos")]
fn get_caret_position() -> Option<tauri::PhysicalPosition<i32>> {
    None
}