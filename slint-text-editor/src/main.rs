#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

use rfd::FileDialog;
use std::{cell::RefCell, path::PathBuf, rc::Rc, time::Duration};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

const APP_NAME: &str = "slint-text-editor";

struct State {
    current_path: Option<PathBuf>,
    modified: bool,
    pending: Option<Pending>,
}

enum Pending {
    New,
    Open,
    Quit,
}

fn main() {
    // ── Tray ─────────────────────────────────────────────────────────────────
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    tray_menu.append(&toggle_item).unwrap();
    tray_menu.append(&PredefinedMenuItem::separator()).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon_rgba: Vec<u8> = (0..32 * 32).flat_map(|_| [0x26u8, 0x8b, 0xd2, 0xff]).collect();
    let tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon_rgba, 32, 32).unwrap())
        .with_menu(Box::new(tray_menu))
        .with_tooltip(APP_NAME)
        .build()
        .unwrap();
    Box::leak(Box::new(tray));

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    // ── Window ────────────────────────────────────────────────────────────────
    let window = AppWindow::new().unwrap();
    let state = Rc::new(RefCell::new(State {
        current_path: None,
        modified: false,
        pending: None,
    }));

    // ── Callbacks ─────────────────────────────────────────────────────────────

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_text_edited(move |text| {
            let win = w.unwrap();
            s.borrow_mut().modified = true;
            let title = make_title(&s.borrow().current_path, true);
            win.set_window_title(title.into());
            win.set_line_numbers_text(line_numbers(&text).into());
        });
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_new_file(move || {
            let win = w.unwrap();
            if s.borrow().modified {
                s.borrow_mut().pending = Some(Pending::New);
                win.set_show_discard_dialog(true);
                win.set_editor_enabled(false);
            } else {
                do_new(&win, &s);
            }
        });
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_open_file(move || {
            let win = w.unwrap();
            if s.borrow().modified {
                s.borrow_mut().pending = Some(Pending::Open);
                win.set_show_discard_dialog(true);
                win.set_editor_enabled(false);
            } else {
                do_open(&win, &s);
            }
        });
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_save_file(move || do_save(&w.unwrap(), &s, false));
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_save_file_as(move || do_save(&w.unwrap(), &s, true));
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_quit(move || {
            let win = w.unwrap();
            if s.borrow().modified {
                s.borrow_mut().pending = Some(Pending::Quit);
                win.set_show_discard_dialog(true);
                win.set_editor_enabled(false);
            } else {
                slint::quit_event_loop().unwrap();
            }
        });
    }

    {
        let w = window.as_weak();
        let s = state.clone();
        window.on_discard_confirmed(move || {
            let win = w.unwrap();
            win.set_show_discard_dialog(false);
            win.set_editor_enabled(true);
            let pending = s.borrow_mut().pending.take();
            match pending {
                Some(Pending::New) => do_new(&win, &s),
                Some(Pending::Open) => do_open(&win, &s),
                Some(Pending::Quit) => slint::quit_event_loop().unwrap(),
                None => {}
            }
        });
    }

    {
        let w = window.as_weak();
        window.on_discard_cancelled(move || {
            let win = w.unwrap();
            win.set_show_discard_dialog(false);
            win.set_editor_enabled(true);
        });
    }

    {
        let w = window.as_weak();
        window.on_error_dismissed(move || {
            let win = w.unwrap();
            win.set_show_error_dialog(false);
            win.set_editor_enabled(true);
        });
    }

    // Close button → hide to tray
    {
        window.window().on_close_requested(|| slint::CloseRequestResponse::HideWindow);
    }

    // ── Tray poll timer ───────────────────────────────────────────────────────
    {
        let w = window.as_weak();
        let s = state.clone();
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(16),
            move || {
                let win = w.unwrap();
                let mut do_toggle = false;
                let mut do_quit = false;

                while let Ok(ev) = MenuEvent::receiver().try_recv() {
                    if ev.id == toggle_id {
                        do_toggle = true;
                    } else if ev.id == quit_id {
                        do_quit = true;
                    }
                }
                while let Ok(ev) = TrayIconEvent::receiver().try_recv() {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev
                    {
                        do_toggle = true;
                    }
                }

                if do_quit {
                    win.show().ok();
                    if s.borrow().modified {
                        s.borrow_mut().pending = Some(Pending::Quit);
                        win.set_show_discard_dialog(true);
                        win.set_editor_enabled(false);
                    } else {
                        slint::quit_event_loop().ok();
                    }
                } else if do_toggle {
                    if win.window().is_visible() {
                        win.hide().ok();
                    } else {
                        win.show().ok();
                    }
                }
            },
        );
        // Keep timer alive for the program's lifetime.
        std::mem::forget(timer);
    }

    window.run().unwrap();
}

// ── File actions ──────────────────────────────────────────────────────────────

fn do_new(win: &AppWindow, state: &Rc<RefCell<State>>) {
    win.set_editor_text("".into());
    win.set_line_numbers_text("   1".into());
    let mut s = state.borrow_mut();
    s.current_path = None;
    s.modified = false;
    drop(s);
    win.set_window_title(APP_NAME.into());
}

fn do_open(win: &AppWindow, state: &Rc<RefCell<State>>) {
    let Some(path) = FileDialog::new().pick_file() else { return };
    match std::fs::read_to_string(&path) {
        Ok(text) => {
            win.set_line_numbers_text(line_numbers(&text).into());
            win.set_editor_text(text.into());
            let mut s = state.borrow_mut();
            s.current_path = Some(path);
            s.modified = false;
            let title = make_title(&s.current_path, false);
            drop(s);
            win.set_window_title(title.into());
        }
        Err(e) => {
            win.set_error_message(format!("Could not open file:\n{e}").into());
            win.set_show_error_dialog(true);
            win.set_editor_enabled(false);
        }
    }
}

fn do_save(win: &AppWindow, state: &Rc<RefCell<State>>, force_dialog: bool) {
    let existing = if force_dialog { None } else { state.borrow().current_path.clone() };
    let path = match existing {
        Some(p) => p,
        None => match FileDialog::new().save_file() {
            Some(p) => p,
            None => return,
        },
    };
    let text = win.get_editor_text();
    match std::fs::write(&path, text.as_str()) {
        Ok(_) => {
            let mut s = state.borrow_mut();
            s.current_path = Some(path);
            s.modified = false;
            let title = make_title(&s.current_path, false);
            drop(s);
            win.set_window_title(title.into());
        }
        Err(e) => {
            win.set_error_message(format!("Could not save file:\n{e}").into());
            win.set_show_error_dialog(true);
            win.set_editor_enabled(false);
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn line_numbers(text: &str) -> String {
    let count = text.split('\n').count().max(1);
    (1..=count).map(|n| format!("{n:>4}")).collect::<Vec<_>>().join("\n")
}

fn make_title(path: &Option<PathBuf>, modified: bool) -> String {
    let name = path
        .as_deref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string());
    match (path.is_some(), modified) {
        (_, true) => format!("• {} — {APP_NAME}", name),
        (true, false) => format!("{} — {APP_NAME}", name),
        (false, false) => APP_NAME.to_string(),
    }
}
