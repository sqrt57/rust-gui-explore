#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};

#[derive(Default)]
struct AppState {
    modified: bool,
    current_path: Option<String>,
}

#[tauri::command]
fn update_state(state: tauri::State<'_, Mutex<AppState>>, modified: bool, path: Option<String>) {
    let mut s = state.lock().unwrap();
    s.modified = modified;
    s.current_path = path;
}

#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn show_open_dialog() -> Option<String> {
    rfd::FileDialog::new()
        .pick_file()
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn show_save_dialog(filename: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(ref name) = filename {
        dialog = dialog.set_file_name(name);
    }
    dialog.save_file().map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn quit_app() {
    std::process::exit(0);
}

fn main() {
    tauri::Builder::default()
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            update_state,
            read_file,
            write_file,
            show_open_dialog,
            show_save_dialog,
            quit_app,
        ])
        .setup(|app| {
            let icon_rgba: Vec<u8> = (0..32u32 * 32)
                .flat_map(|_| [0x26u8, 0x8bu8, 0xd2u8, 0xffu8])
                .collect();
            let icon = Image::new(&icon_rgba, 32, 32);

            let show_hide =
                MenuItem::with_id(app, "toggle", "Show / Hide", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&show_hide, &sep, &quit_item])?;

            let tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("tauri-text-editor")
                .on_menu_event(|app, event| {
                    let win = app.get_webview_window("main").unwrap();
                    match event.id.as_ref() {
                        "toggle" => {
                            if win.is_visible().unwrap_or(false) {
                                let _ = win.hide();
                            } else {
                                let _ = win.show();
                                let _ = win.set_focus();
                            }
                        }
                        "quit" => {
                            let state = app.state::<Mutex<AppState>>();
                            let modified = state.lock().unwrap().modified;
                            if modified {
                                let _ = win.show();
                                let _ = win.set_focus();
                                let _ = win.emit("tray-quit", ());
                            } else {
                                std::process::exit(0);
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let win = tray.app_handle().get_webview_window("main").unwrap();
                        if win.is_visible().unwrap_or(false) {
                            let _ = win.hide();
                        } else {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Keep tray alive for the lifetime of the process.
            std::mem::forget(tray);

            // Close button → hide to tray instead of quitting.
            let win = app.get_webview_window("main").unwrap();
            let win_clone = win.clone();
            win.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = win_clone.hide();
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
