use fltk::{
    app,
    dialog,
    enums::{Event, Shortcut},
    menu::{MenuBar, MenuFlag},
    prelude::*,
    text::{TextBuffer, TextEditor},
    window::Window,
};
use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};
use tray_icon::{
    Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

const TITLE: &str = "fltk-text-editor";

fn main() {
    let app = app::App::default().with_scheme(app::Scheme::Gtk);

    let mut win = Window::default().with_size(900, 650).with_label(TITLE);
    win.make_resizable(true);
    let mut menu = MenuBar::default().with_size(900, 25);
    let buf = TextBuffer::default();
    let mut editor = TextEditor::default().with_pos(0, 25).with_size(900, 625);
    editor.set_buffer(buf.clone());
    editor.set_linenumber_width(48);
    editor.set_text_font(fltk::enums::Font::Courier);
    editor.set_text_size(14);
    editor.wrap_mode(fltk::text::WrapMode::AtBounds, 0);
    win.end();
    win.show();

    let current_path: Rc<RefCell<Option<PathBuf>>> = Rc::new(RefCell::new(None));
    let modified: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));

    {
        let modified = modified.clone();
        let mut win2 = win.clone();
        let path2 = current_path.clone();
        buf.clone().add_modify_callback(move |_, _, _, _, _| {
            if !*modified.borrow() {
                *modified.borrow_mut() = true;
                win2.set_label(&title_label(path2.borrow().as_deref(), true));
            }
        });
    }

    let new_cb = {
        let modified = modified.clone();
        let current_path = current_path.clone();
        let mut buf = buf.clone();
        let mut win2 = win.clone();
        move || {
            if confirm_discard(&modified) {
                buf.set_text("");
                *current_path.borrow_mut() = None;
                *modified.borrow_mut() = false;
                win2.set_label(TITLE);
            }
        }
    };

    let open_cb = {
        let modified = modified.clone();
        let current_path = current_path.clone();
        let mut buf = buf.clone();
        let mut win2 = win.clone();
        move || {
            if !confirm_discard(&modified) {
                return;
            }
            let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseFile);
            dlg.set_title("Open file");
            dlg.show();
            let path = dlg.filename();
            if path.as_os_str().is_empty() {
                return;
            }
            match fs::read_to_string(&path) {
                Ok(text) => {
                    buf.set_text(&text);
                    *current_path.borrow_mut() = Some(path.clone());
                    *modified.borrow_mut() = false;
                    win2.set_label(&title_label(Some(&path), false));
                }
                Err(e) => dialog::alert_default(&format!("Could not open file:\n{e}")),
            }
        }
    };

    let save_cb = {
        let modified = modified.clone();
        let current_path = current_path.clone();
        let buf = buf.clone();
        let mut win2 = win.clone();
        move || save_file(&buf, &current_path, &modified, &mut win2, false)
    };

    let save_as_cb = {
        let modified = modified.clone();
        let current_path = current_path.clone();
        let buf = buf.clone();
        let mut win2 = win.clone();
        move || save_file(&buf, &current_path, &modified, &mut win2, true)
    };

    menu.add("&File/&New\t", Shortcut::Ctrl | 'n', MenuFlag::Normal, {
        let mut cb = new_cb.clone();
        move |_| cb()
    });
    menu.add("&File/&Open…\t", Shortcut::Ctrl | 'o', MenuFlag::Normal, {
        let mut cb = open_cb.clone();
        move |_| cb()
    });
    menu.add("&File/&Save\t", Shortcut::Ctrl | 's', MenuFlag::Normal, {
        let mut cb = save_cb.clone();
        move |_| cb()
    });
    menu.add(
        "&File/Save &As…\t",
        Shortcut::Ctrl | Shortcut::Shift | 's',
        MenuFlag::MenuDivider,
        {
            let mut cb = save_as_cb.clone();
            move |_| cb()
        },
    );
    menu.add("&File/&Quit\t", Shortcut::Ctrl | 'q', MenuFlag::Normal, {
        let modified = modified.clone();
        move |_| {
            if confirm_discard(&modified) {
                app::quit();
            }
        }
    });

    // --- Tray icon ---
    let tray_menu = Menu::new();
    let toggle_item = MenuItem::new("Show / Hide", true, None);
    let quit_item = MenuItem::new("Quit", true, None);
    let sep = PredefinedMenuItem::separator();
    tray_menu.append(&toggle_item).unwrap();
    tray_menu.append(&sep).unwrap();
    tray_menu.append(&quit_item).unwrap();

    let icon_rgba: Vec<u8> = (0u32..32 * 32)
        .flat_map(|_| [0x26u8, 0x8b, 0xd2, 0xff])
        .collect();
    let _tray = TrayIconBuilder::new()
        .with_icon(Icon::from_rgba(icon_rgba, 32, 32).unwrap())
        .with_menu(Box::new(tray_menu))
        .with_tooltip(TITLE)
        .build()
        .unwrap();

    let toggle_id = toggle_item.id().clone();
    let quit_id = quit_item.id().clone();

    // Close button → hide to tray instead of quitting
    win.set_callback(move |w| {
        if app::event() == Event::Close {
            unsafe { os_hide_show(w.raw_handle(), false) };
        }
    });

    // Idle: poll tray and menu events
    {
        let win2 = win.clone();
        let modified = modified.clone();
        app::add_idle3(move |_| {
            if let Ok(ev) = MenuEvent::receiver().try_recv() {
                if ev.id == toggle_id {
                    unsafe { os_toggle(win2.raw_handle()) };
                } else if ev.id == quit_id {
                    if confirm_discard(&modified) {
                        app::quit();
                    }
                }
            }
            if let Ok(TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }) = TrayIconEvent::receiver().try_recv()
            {
                unsafe { os_toggle(win2.raw_handle()) };
            }
        });
    }

    win.resize_callback(move |_, _, _, w, h| {
        menu.resize(0, 0, w, 25);
        editor.resize(0, 25, w, h - 25);
    });

    app.run().unwrap();
}

fn title_label(path: Option<&std::path::Path>, modified: bool) -> String {
    let name = path
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string());
    if modified {
        format!("• {name} — {TITLE}")
    } else {
        format!("{name} — {TITLE}")
    }
}

fn confirm_discard(modified: &Rc<RefCell<bool>>) -> bool {
    if !*modified.borrow() {
        return true;
    }
    let choice = dialog::choice2_default(
        "You have unsaved changes. Discard them?",
        "Cancel",
        "Discard",
        "",
    );
    matches!(choice, Some(1))
}

fn save_file(
    buf: &TextBuffer,
    current_path: &Rc<RefCell<Option<PathBuf>>>,
    modified: &Rc<RefCell<bool>>,
    win: &mut Window,
    force_dialog: bool,
) {
    let path: Option<PathBuf> = if force_dialog {
        None
    } else {
        current_path.borrow().clone()
    };

    let path = match path {
        Some(p) => p,
        None => {
            let mut dlg = dialog::FileDialog::new(dialog::FileDialogType::BrowseSaveFile);
            dlg.set_title("Save file");
            dlg.show();
            let p = dlg.filename();
            if p.as_os_str().is_empty() {
                return;
            }
            p
        }
    };

    match fs::write(&path, buf.text()) {
        Ok(_) => {
            *current_path.borrow_mut() = Some(path.clone());
            *modified.borrow_mut() = false;
            win.set_label(&title_label(Some(&path), false));
        }
        Err(e) => dialog::alert_default(&format!("Could not save file:\n{e}")),
    }
}

// Hide/show the window at the Win32 level without touching FLTK's internal state.
// This keeps the FLTK event loop alive even when the window is invisible.
unsafe fn os_hide_show(hwnd: *mut std::ffi::c_void, show: bool) {
    unsafe extern "system" {
        fn ShowWindow(hwnd: *mut std::ffi::c_void, cmd: i32) -> i32;
        fn SetForegroundWindow(hwnd: *mut std::ffi::c_void) -> i32;
    }
    if show {
        unsafe {
            ShowWindow(hwnd, 5); // SW_SHOW
            SetForegroundWindow(hwnd);
        }
    } else {
        unsafe { ShowWindow(hwnd, 0) }; // SW_HIDE
    }
}

unsafe fn os_toggle(hwnd: *mut std::ffi::c_void) {
    unsafe extern "system" {
        fn IsWindowVisible(hwnd: *mut std::ffi::c_void) -> i32;
    }
    unsafe { os_hide_show(hwnd, IsWindowVisible(hwnd) == 0) };
}
