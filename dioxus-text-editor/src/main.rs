#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::desktop::{
    Config, LogicalSize, WindowBuilder, WindowCloseBehaviour,
    use_tray_icon_event_handler, use_tray_menu_event_handler, use_window,
};
use dioxus::document::eval;
use dioxus::prelude::*;
use rfd::AsyncFileDialog;
use std::path::PathBuf;
use std::sync::OnceLock;
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuId, MenuItem, PredefinedMenuItem},
};

const APP_NAME: &str = "dioxus-text-editor";

static TOGGLE_ID: OnceLock<MenuId> = OnceLock::new();
static QUIT_ID: OnceLock<MenuId> = OnceLock::new();

#[derive(Clone, PartialEq, Debug)]
enum Pending {
    New,
    Open,
    Quit,
}

fn main() {
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&toggle_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&quit_item).unwrap();

    TOGGLE_ID.set(toggle_item.id().clone()).unwrap();
    QUIT_ID.set(quit_item.id().clone()).unwrap();

    let icon_rgba: Vec<u8> = (0..32 * 32)
        .flat_map(|_| [0x26u8, 0x8b, 0xd2, 0xff])
        .collect();
    let tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon_rgba, 32, 32).unwrap())
        .with_menu(Box::new(tray_menu))
        .with_tooltip(APP_NAME)
        .build()
        .unwrap();
    Box::leak(Box::new(tray));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title(APP_NAME)
                        .with_inner_size(LogicalSize::new(900.0_f64, 650.0_f64)),
                )
                .with_close_behaviour(WindowCloseBehaviour::LastWindowHides),
        )
        .launch(app);
}

fn app() -> Element {
    let mut content: Signal<String> = use_signal(String::new);
    let current_path: Signal<Option<PathBuf>> = use_signal(|| None);
    let mut modified: Signal<bool> = use_signal(|| false);
    let mut pending: Signal<Option<Pending>> = use_signal(|| None);
    let mut show_dialog: Signal<bool> = use_signal(|| false);
    let mut error_msg: Signal<Option<String>> = use_signal(|| None);
    let mut file_menu_open: Signal<bool> = use_signal(|| false);

    let desktop = use_window();

    // Update native window title whenever path or modified flag changes.
    let d_effect = desktop.clone();
    use_effect(move || {
        let path_guard = current_path.read();
        let mod_flag = *modified.read();
        let name = path_guard
            .as_deref()
            .and_then(|p| p.file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".to_string());
        let has_path = path_guard.is_some();
        drop(path_guard);
        let title = match (has_path, mod_flag) {
            (_, true) => format!("• {} — {APP_NAME}", name),
            (true, false) => format!("{} — {APP_NAME}", name),
            (false, false) => APP_NAME.to_string(),
        };
        d_effect.window.set_title(&title);
    });

    // Tray menu events (forwarded by Dioxus from tray-icon's global handler).
    let d_menu = desktop.clone();
    use_tray_menu_event_handler(move |event| {
        let toggle_id = TOGGLE_ID.get().unwrap();
        let quit_id = QUIT_ID.get().unwrap();
        if event.id == *quit_id {
            if *modified.peek() {
                pending.set(Some(Pending::Quit));
                show_dialog.set(true);
                d_menu.window.set_visible(true);
            } else {
                std::process::exit(0);
            }
        } else if event.id == *toggle_id {
            let vis = d_menu.window.is_visible();
            d_menu.window.set_visible(!vis);
        }
    });

    // Tray icon left-click → toggle visibility.
    let d_tray = desktop.clone();
    use_tray_icon_event_handler(move |event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let vis = d_tray.window.is_visible();
            d_tray.window.set_visible(!vis);
        }
    });

    // Line numbers string: one line per content line, right-aligned.
    let line_count = content.read().lines().count().max(1);
    let line_nums: String = (1..=line_count)
        .map(|i| format!("{i:>4}"))
        .collect::<Vec<_>>()
        .join("\n");

    rsx! {
        style { "{CSS}" }

        // Transparent full-screen overlay dismisses open menu on outside click.
        if file_menu_open() {
            div {
                style: "position:fixed;inset:0;z-index:99;",
                onclick: move |_| file_menu_open.set(false),
            }
        }

        div { id: "menubar",
            div { style: "position:relative;z-index:100;",
                span {
                    class: if file_menu_open() { "menu-title active" } else { "menu-title" },
                    onclick: move |e| {
                        e.stop_propagation();
                        file_menu_open.set(!file_menu_open());
                    },
                    "File"
                }
                if file_menu_open() {
                    div { class: "menu-dropdown",
                        div {
                            class: "menu-item",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                if *modified.peek() {
                                    pending.set(Some(Pending::New));
                                    show_dialog.set(true);
                                } else {
                                    do_new(content, current_path, modified);
                                }
                            },
                            span { "New" }
                            span { class: "shortcut", "Ctrl+N" }
                        }
                        div {
                            class: "menu-item",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                if *modified.peek() {
                                    pending.set(Some(Pending::Open));
                                    show_dialog.set(true);
                                } else {
                                    trigger_open(content, current_path, modified, error_msg);
                                }
                            },
                            span { "Open…" }
                            span { class: "shortcut", "Ctrl+O" }
                        }
                        div {
                            class: "menu-item",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                trigger_save(false, content, current_path, modified, error_msg);
                            },
                            span { "Save" }
                            span { class: "shortcut", "Ctrl+S" }
                        }
                        div {
                            class: "menu-item",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                trigger_save(true, content, current_path, modified, error_msg);
                            },
                            span { "Save As…" }
                            span { class: "shortcut", "Ctrl+Shift+S" }
                        }
                        div { class: "menu-separator" }
                        div {
                            class: "menu-item",
                            onclick: move |_| {
                                file_menu_open.set(false);
                                if *modified.peek() {
                                    pending.set(Some(Pending::Quit));
                                    show_dialog.set(true);
                                } else {
                                    std::process::exit(0);
                                }
                            },
                            span { "Quit" }
                            span { class: "shortcut", "Ctrl+Q" }
                        }
                    }
                }
            }
        }

        div { id: "editor-wrap",
            div { id: "line-numbers", "{line_nums}" }
            textarea {
                id: "editor",
                value: "{content}",
                spellcheck: false,
                oninput: move |e| {
                    content.set(e.value());
                    modified.set(true);
                },
                onkeydown: move |e| {
                    let mods = e.modifiers();
                    if !mods.ctrl() || mods.alt() {
                        return;
                    }
                    let shift = mods.shift();
                    let key = e.key().to_string().to_lowercase();
                    match key.as_str() {
                        "n" => {
                            if *modified.peek() {
                                pending.set(Some(Pending::New));
                                show_dialog.set(true);
                            } else {
                                do_new(content, current_path, modified);
                            }
                        }
                        "o" => {
                            if *modified.peek() {
                                pending.set(Some(Pending::Open));
                                show_dialog.set(true);
                            } else {
                                trigger_open(content, current_path, modified, error_msg);
                            }
                        }
                        "s" => trigger_save(shift, content, current_path, modified, error_msg),
                        "q" => {
                            if *modified.peek() {
                                pending.set(Some(Pending::Quit));
                                show_dialog.set(true);
                            } else {
                                std::process::exit(0);
                            }
                        }
                        _ => {}
                    }
                },
                onscroll: move |_| {
                    eval(
                        "document.getElementById('line-numbers').scrollTop \
                         = document.getElementById('editor').scrollTop",
                    );
                },
            }
        }

        if show_dialog() {
            div { class: "overlay",
                div { class: "dialog",
                    div { class: "dialog-title", "Unsaved Changes" }
                    div { class: "dialog-msg",
                        "You have unsaved changes. Discard them?"
                    }
                    div { class: "dialog-buttons",
                        button {
                            class: "btn",
                            onclick: move |_| {
                                show_dialog.set(false);
                                pending.set(None);
                            },
                            "Cancel"
                        }
                        button {
                            class: "btn btn-danger",
                            onclick: move |_| {
                                show_dialog.set(false);
                                match pending.write().take() {
                                    Some(Pending::New) => do_new(content, current_path, modified),
                                    Some(Pending::Open) => {
                                        trigger_open(content, current_path, modified, error_msg)
                                    }
                                    Some(Pending::Quit) => std::process::exit(0),
                                    None => {}
                                }
                            },
                            "Discard"
                        }
                    }
                }
            }
        }

        if let Some(msg) = error_msg() {
            div { class: "overlay",
                div { class: "dialog",
                    div { class: "dialog-title", "Error" }
                    div { class: "dialog-msg", "{msg}" }
                    div { class: "dialog-buttons",
                        button {
                            class: "btn",
                            onclick: move |_| error_msg.set(None),
                            "OK"
                        }
                    }
                }
            }
        }
    }
}

// ── Action helpers ────────────────────────────────────────────────────────────

fn do_new(
    mut content: Signal<String>,
    mut current_path: Signal<Option<PathBuf>>,
    mut modified: Signal<bool>,
) {
    content.set(String::new());
    current_path.set(None);
    modified.set(false);
}

fn trigger_open(
    mut content: Signal<String>,
    mut current_path: Signal<Option<PathBuf>>,
    mut modified: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
) {
    spawn(async move {
        if let Some(handle) = AsyncFileDialog::new().pick_file().await {
            let path = handle.path().to_owned();
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    content.set(text);
                    current_path.set(Some(path));
                    modified.set(false);
                }
                Err(e) => error_msg.set(Some(e.to_string())),
            }
        }
    });
}

fn trigger_save(
    force_dialog: bool,
    content: Signal<String>,
    mut current_path: Signal<Option<PathBuf>>,
    mut modified: Signal<bool>,
    mut error_msg: Signal<Option<String>>,
) {
    let existing = if force_dialog {
        None
    } else {
        current_path.peek().clone()
    };
    let text = content.peek().clone();
    spawn(async move {
        let save_path = match existing {
            Some(p) => p,
            None => match AsyncFileDialog::new().save_file().await {
                Some(h) => h.path().to_owned(),
                None => return,
            },
        };
        match std::fs::write(&save_path, text.as_bytes()) {
            Ok(_) => {
                current_path.set(Some(save_path));
                modified.set(false);
            }
            Err(e) => error_msg.set(Some(e.to_string())),
        }
    });
}

// ── Styles ────────────────────────────────────────────────────────────────────

const CSS: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }

html, body {
    height: 100%;
    overflow: hidden;
    font-family: system-ui, sans-serif;
    background: #fff;
}

body { display: flex; flex-direction: column; }

#menubar {
    display: flex;
    background: #f0f0f0;
    border-bottom: 1px solid #ccc;
    flex-shrink: 0;
    user-select: none;
}

.menu-title {
    display: block;
    padding: 5px 14px;
    cursor: default;
    font-size: 13px;
}

.menu-title:hover, .menu-title.active {
    background: #0078d4;
    color: white;
}

.menu-dropdown {
    position: absolute;
    top: 100%;
    left: 0;
    background: white;
    border: 1px solid #bbb;
    min-width: 220px;
    z-index: 1000;
    box-shadow: 2px 4px 10px rgba(0,0,0,0.2);
    padding: 3px 0;
}

.menu-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 5px 16px;
    cursor: default;
    font-size: 13px;
    white-space: nowrap;
}

.menu-item:hover { background: #0078d4; color: white; }

.shortcut {
    font-size: 11px;
    color: #999;
    margin-left: 28px;
}

.menu-item:hover .shortcut { color: #cce4ff; }

.menu-separator {
    height: 1px;
    background: #e0e0e0;
    margin: 3px 0;
}

#editor-wrap {
    display: flex;
    flex: 1;
    overflow: hidden;
    font-family: 'Cascadia Code', 'Consolas', 'Courier New', monospace;
    font-size: 14px;
    line-height: 1.5;
}

#line-numbers {
    padding: 4px 8px;
    min-width: 52px;
    background: #f7f7f7;
    border-right: 1px solid #ddd;
    color: #bbb;
    text-align: right;
    overflow: hidden;
    white-space: pre;
    user-select: none;
    flex-shrink: 0;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
}

#editor {
    flex: 1;
    padding: 4px 8px;
    border: none;
    outline: none;
    resize: none;
    font-family: inherit;
    font-size: inherit;
    line-height: inherit;
    word-wrap: break-word;
    overflow-y: scroll;
    overflow-x: auto;
    tab-size: 4;
}

.overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    z-index: 500;
    display: flex;
    align-items: center;
    justify-content: center;
}

.dialog {
    background: white;
    border-radius: 6px;
    padding: 24px 28px;
    width: 340px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.3);
}

.dialog-title {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 10px;
}

.dialog-msg {
    font-size: 13px;
    color: #555;
    margin-bottom: 20px;
    line-height: 1.5;
}

.dialog-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
}

.btn {
    padding: 6px 18px;
    font-size: 13px;
    border: 1px solid #bbb;
    border-radius: 4px;
    cursor: pointer;
    background: white;
}

.btn:hover { background: #f0f0f0; }

.btn-danger { background: #c0392b; color: white; border-color: #c0392b; }
.btn-danger:hover { background: #a93226; }
"#;
