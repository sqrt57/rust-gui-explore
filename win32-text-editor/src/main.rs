#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(unused_must_use)]

use std::path::PathBuf;
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::Dialogs::*,
            Shell::*,        // includes SetWindowSubclass, DefSubclassProc
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

// EDIT control messages not exported by the windows crate features used here
const EM_LINESCROLL: u32 = 0x00B6;
const EM_GETLINECOUNT: u32 = 0x00BA;
const EM_GETFIRSTVISIBLELINE: u32 = 0x00CE;

// EDIT styles (i32 in windows-rs; wrap via WINDOW_STYLE when combined with WS_*)
const ES_LEFT: i32 = 0x0000;
const ES_MULTILINE: i32 = 0x0004;
const ES_AUTOVSCROLL: i32 = 0x0040;
const ES_READONLY: i32 = 0x0800;
const ES_NOHIDESEL: i32 = 0x0100;
const ES_WANTRETURN: i32 = 0x1000;

const APP_NAME: &str = "win32-text-editor";
const IDM_NEW: usize = 101;
const IDM_OPEN: usize = 102;
const IDM_SAVE: usize = 103;
const IDM_SAVE_AS: usize = 104;
const IDM_QUIT: usize = 105;
const IDM_TRAY_TOGGLE: usize = 201;
const IDM_TRAY_QUIT: usize = 202;
const IDC_EDITOR: usize = 1001;
const IDC_LINENOS: usize = 1002;
const WM_TRAY: u32 = WM_USER + 1;
const TRAY_UID: u32 = 1;
const SUBCLASS_EDITOR: usize = 1;
const LN_WIDTH: i32 = 52;

struct AppState {
    editor: HWND,
    linenos: HWND,
    tray_icon: HICON,
    gray_brush: HBRUSH,
    modified: bool,
    current_path: Option<PathBuf>,
    ignore_change: bool,
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn main() {
    unsafe { run() }
}

unsafe fn run() {
    let hinstance = GetModuleHandleW(None).unwrap();
    let class_name = w!("Win32TextEditor");

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wnd_proc),
        hInstance: hinstance.into(),
        hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
        hbrBackground: HBRUSH((COLOR_WINDOW.0 + 1) as *mut _),
        lpszClassName: class_name,
        ..Default::default()
    };
    RegisterClassExW(&wc);

    let hwnd = CreateWindowExW(
        WINDOW_EX_STYLE::default(),
        class_name,
        w!("win32-text-editor"),
        WS_OVERLAPPEDWINDOW,
        CW_USEDEFAULT, CW_USEDEFAULT, 900, 650,
        None, None, hinstance, None,
    )
    .unwrap();

    let accels = [
        ACCEL { fVirt: FVIRTKEY | FCONTROL,          key: b'N' as u16, cmd: IDM_NEW as u16 },
        ACCEL { fVirt: FVIRTKEY | FCONTROL,          key: b'O' as u16, cmd: IDM_OPEN as u16 },
        ACCEL { fVirt: FVIRTKEY | FCONTROL,          key: b'S' as u16, cmd: IDM_SAVE as u16 },
        ACCEL { fVirt: FVIRTKEY | FCONTROL | FSHIFT, key: b'S' as u16, cmd: IDM_SAVE_AS as u16 },
        ACCEL { fVirt: FVIRTKEY | FCONTROL,          key: b'Q' as u16, cmd: IDM_QUIT as u16 },
    ];
    let haccel = CreateAcceleratorTableW(&accels).unwrap();

    ShowWindow(hwnd, SW_SHOW);
    UpdateWindow(hwnd);

    let mut msg = MSG::default();
    loop {
        let ret = GetMessageW(&mut msg, None, 0, 0);
        if ret.0 <= 0 {
            break;
        }
        if TranslateAcceleratorW(hwnd, haccel, &msg) == 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => on_create(hwnd),

        WM_SIZE => {
            let w = (lparam.0 & 0xFFFF) as i32;
            let h = ((lparam.0 >> 16) & 0xFFFF) as i32;
            let s = state_ptr(hwnd);
            if !s.is_null() {
                SetWindowPos((*s).linenos, None, 0, 0, LN_WIDTH, h, SWP_NOZORDER).unwrap();
                SetWindowPos((*s).editor, None, LN_WIDTH, 0, w - LN_WIDTH, h, SWP_NOZORDER)
                    .unwrap();
            }
            LRESULT(0)
        }

        WM_COMMAND => {
            let id = wparam.0 & 0xFFFF;
            let notif = (wparam.0 >> 16) & 0xFFFF;
            let s = state_ptr(hwnd);
            if s.is_null() {
                return LRESULT(0);
            }
            if notif == EN_CHANGE as usize && id == IDC_EDITOR {
                if !(*s).ignore_change {
                    let was = (*s).modified;
                    (*s).modified = true;
                    if !was {
                        let path = (*s).current_path.clone();
                        update_title(hwnd, &path, true);
                    }
                    let (ed, ln) = ((*s).editor, (*s).linenos);
                    update_line_numbers(ed, ln);
                }
            } else {
                match id {
                    IDM_NEW                       => handle_new(hwnd),
                    IDM_OPEN                      => handle_open(hwnd),
                    IDM_SAVE                      => handle_save(hwnd, false),
                    IDM_SAVE_AS                   => handle_save(hwnd, true),
                    IDM_QUIT | IDM_TRAY_QUIT      => handle_quit(hwnd),
                    IDM_TRAY_TOGGLE               => toggle_visibility(hwnd),
                    _ => {}
                }
            }
            LRESULT(0)
        }

        // Gray background + muted text color for read-only line-numbers strip
        WM_CTLCOLORSTATIC => {
            let ctl = HWND(lparam.0 as *mut _);
            let hdc = HDC(wparam.0 as *mut _);
            let s = state_ptr(hwnd);
            if !s.is_null() && ctl == (*s).linenos {
                SetBkColor(hdc, COLORREF(0x00F7F7F7));
                SetTextColor(hdc, COLORREF(0x00BBBBBB));
                return LRESULT((*s).gray_brush.0 as isize);
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }

        WM_TRAY => {
            match (lparam.0 & 0xFFFF) as u32 {
                WM_LBUTTONUP => toggle_visibility(hwnd),
                WM_RBUTTONUP => show_tray_menu(hwnd),
                _ => {}
            }
            LRESULT(0)
        }

        WM_CLOSE => {
            ShowWindow(hwnd, SW_HIDE);
            LRESULT(0)
        }

        WM_DESTROY => {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_UID;
            let _ = Shell_NotifyIconW(NIM_DELETE, &nid);

            let s = state_ptr(hwnd);
            if !s.is_null() {
                let _ = DestroyIcon((*s).tray_icon);
                DeleteObject(HGDIOBJ((*s).gray_brush.0));
                drop(Box::from_raw(s));
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

unsafe fn on_create(hwnd: HWND) -> LRESULT {
    let hinstance = GetModuleHandleW(None).unwrap();

    // Monospace font — Consolas 14 pt at 96 DPI (height = -18 logical units)
    let font = CreateFontW(
        -18, 0, 0, 0,
        FW_NORMAL.0 as i32,
        0, 0, 0,
        ANSI_CHARSET.0 as u32,
        OUT_DEFAULT_PRECIS.0 as u32,
        CLIP_DEFAULT_PRECIS.0 as u32,
        DEFAULT_QUALITY.0 as u32,
        (FIXED_PITCH.0 | FF_MODERN.0) as u32,
        w!("Consolas"),
    );

    // Line-numbers strip — read-only, no scrollbars
    let ln_style = WINDOW_STYLE((ES_MULTILINE | ES_READONLY | ES_LEFT) as u32);
    let linenos = CreateWindowExW(
        WINDOW_EX_STYLE::default(), w!("EDIT"), None,
        WS_CHILD | WS_VISIBLE | ln_style,
        0, 0, LN_WIDTH, 1,
        hwnd, HMENU(IDC_LINENOS as *mut _), hinstance, None,
    )
    .unwrap();

    // Main editor — multiline, word-wrap (no WS_HSCROLL), vertical scrollbar
    let ed_style = WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN | ES_NOHIDESEL) as u32);
    let editor = CreateWindowExW(
        WINDOW_EX_STYLE::default(), w!("EDIT"), None,
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | ed_style,
        LN_WIDTH, 0, 1, 1,
        hwnd, HMENU(IDC_EDITOR as *mut _), hinstance, None,
    )
    .unwrap();

    SendMessageW(linenos, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
    SendMessageW(editor,  WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));

    // File menu
    let hmenu = CreateMenu().unwrap();
    let hfile = CreatePopupMenu().unwrap();
    AppendMenuW(hfile, MF_STRING,    IDM_NEW,     w!("New\tCtrl+N")).unwrap();
    AppendMenuW(hfile, MF_STRING,    IDM_OPEN,    w!("Open...\tCtrl+O")).unwrap();
    AppendMenuW(hfile, MF_STRING,    IDM_SAVE,    w!("Save\tCtrl+S")).unwrap();
    AppendMenuW(hfile, MF_STRING,    IDM_SAVE_AS, w!("Save As...\tCtrl+Shift+S")).unwrap();
    AppendMenuW(hfile, MF_SEPARATOR, 0,           PCWSTR::null()).unwrap();
    AppendMenuW(hfile, MF_STRING,    IDM_QUIT,    w!("Quit\tCtrl+Q")).unwrap();
    AppendMenuW(hmenu, MF_POPUP,     hfile.0 as usize, w!("File")).unwrap();
    SetMenu(hwnd, hmenu).unwrap();

    // Tray icon — solid blue #268bd2
    let tray_icon = create_blue_icon();
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_UID;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAY;
    nid.hIcon = tray_icon;
    let tip = to_wide(APP_NAME);
    let len = tip.len().min(nid.szTip.len());
    nid.szTip[..len].copy_from_slice(&tip[..len]);
    let _ = Shell_NotifyIconW(NIM_ADD, &nid);

    let gray_brush = CreateSolidBrush(COLORREF(0x00F7F7F7));

    // Subclass editor — stores parent HWND as ref-data for scroll sync
    SetWindowSubclass(editor, Some(editor_subclass_proc), SUBCLASS_EDITOR, hwnd.0 as usize)
        .unwrap();

    let state = Box::new(AppState {
        editor,
        linenos,
        tray_icon,
        gray_brush,
        modified: false,
        current_path: None,
        ignore_change: false,
    });
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(state) as isize);

    update_line_numbers(editor, linenos);
    LRESULT(0)
}

unsafe extern "system" fn editor_subclass_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _uid: usize,
    data: usize,
) -> LRESULT {
    let result = DefSubclassProc(hwnd, msg, wparam, lparam);
    match msg {
        WM_VSCROLL | WM_MOUSEWHEEL | WM_KEYDOWN | WM_KEYUP | WM_LBUTTONUP => {
            let parent = HWND(data as *mut _);
            let s = state_ptr(parent);
            if !s.is_null() {
                sync_scroll(hwnd, (*s).linenos);
            }
        }
        _ => {}
    }
    result
}

// ── Helpers ───────────────────────────────────────────────────────────────────

unsafe fn state_ptr(hwnd: HWND) -> *mut AppState {
    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState
}

unsafe fn sync_scroll(editor: HWND, linenos: HWND) {
    let ed = SendMessageW(editor,  EM_GETFIRSTVISIBLELINE, WPARAM(0), LPARAM(0)).0 as i32;
    let ln = SendMessageW(linenos, EM_GETFIRSTVISIBLELINE, WPARAM(0), LPARAM(0)).0 as i32;
    let delta = ed - ln;
    if delta != 0 {
        SendMessageW(linenos, EM_LINESCROLL, WPARAM(0), LPARAM(delta as isize));
    }
}

unsafe fn update_line_numbers(editor: HWND, linenos: HWND) {
    let count = SendMessageW(editor, EM_GETLINECOUNT, WPARAM(0), LPARAM(0)).0.max(1) as usize;
    let text: String = (1..=count).map(|i| format!("{i:>4}\r\n")).collect();
    let wide = to_wide(&text);
    SetWindowTextW(linenos, PCWSTR(wide.as_ptr())).unwrap();
    sync_scroll(editor, linenos);
}

unsafe fn update_title(hwnd: HWND, path: &Option<PathBuf>, modified: bool) {
    let name = path
        .as_deref()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Untitled".to_string());
    let title = match (path.is_some(), modified) {
        (_, true)      => format!("• {} — {APP_NAME}", name),
        (true, false)  => format!("{} — {APP_NAME}", name),
        (false, false) => APP_NAME.to_string(),
    };
    let wide = to_wide(&title);
    SetWindowTextW(hwnd, PCWSTR(wide.as_ptr())).unwrap();
}

unsafe fn confirm_discard(hwnd: HWND) -> bool {
    MessageBoxW(
        hwnd,
        w!("You have unsaved changes. Discard them?"),
        w!("Unsaved Changes"),
        MB_YESNO | MB_ICONWARNING | MB_DEFBUTTON2,
    ) == IDYES
}

unsafe fn create_blue_icon() -> HICON {
    let size = 32i32;
    let screen_dc = GetDC(None);
    let mem_dc = CreateCompatibleDC(screen_dc);
    let hbm_color = CreateCompatibleBitmap(screen_dc, size, size);
    let old = SelectObject(mem_dc, HGDIOBJ(hbm_color.0));
    let brush = CreateSolidBrush(COLORREF(0x00D28B26)); // #268bd2 as BGR
    let rc = RECT { left: 0, top: 0, right: size, bottom: size };
    FillRect(mem_dc, &rc, brush);
    DeleteObject(HGDIOBJ(brush.0));
    SelectObject(mem_dc, old);
    DeleteDC(mem_dc);
    ReleaseDC(None, screen_dc);

    // 1-bpp mask: all zeros → colour bitmap alpha provides opacity
    let row_bytes = 4usize; // 32 pixels / 8 bits × word-align → 4 bytes
    let mask_bits = vec![0u8; row_bytes * size as usize];
    let hbm_mask = CreateBitmap(size, size, 1, 1, Some(mask_bits.as_ptr() as *const _));

    let ii = ICONINFO {
        fIcon: TRUE,
        xHotspot: 0,
        yHotspot: 0,
        hbmMask: hbm_mask,
        hbmColor: hbm_color,
    };
    let icon = CreateIconIndirect(&ii).unwrap();
    DeleteObject(HGDIOBJ(hbm_color.0));
    DeleteObject(HGDIOBJ(hbm_mask.0));
    icon
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let hmenu = CreatePopupMenu().unwrap();
    AppendMenuW(hmenu, MF_STRING,    IDM_TRAY_TOGGLE, w!("Show / Hide")).unwrap();
    AppendMenuW(hmenu, MF_SEPARATOR, 0,               PCWSTR::null()).unwrap();
    AppendMenuW(hmenu, MF_STRING,    IDM_TRAY_QUIT,   w!("Quit")).unwrap();
    let mut pt = POINT::default();
    GetCursorPos(&mut pt).unwrap();
    SetForegroundWindow(hwnd);
    let _ = TrackPopupMenu(hmenu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
    let _ = DestroyMenu(hmenu);
}

unsafe fn toggle_visibility(hwnd: HWND) {
    if IsWindowVisible(hwnd).as_bool() {
        ShowWindow(hwnd, SW_HIDE);
    } else {
        ShowWindow(hwnd, SW_RESTORE);
        SetForegroundWindow(hwnd);
    }
}

// ── File actions ──────────────────────────────────────────────────────────────

unsafe fn handle_new(hwnd: HWND) {
    let s = state_ptr(hwnd);
    if (*s).modified && !confirm_discard(hwnd) {
        return;
    }
    (*s).ignore_change = true;
    SetWindowTextW((*s).editor, w!("")).unwrap();
    (*s).ignore_change = false;
    (*s).modified = false;
    (*s).current_path = None;
    update_title(hwnd, &None, false);
    let (ed, ln) = ((*s).editor, (*s).linenos);
    update_line_numbers(ed, ln);
}

unsafe fn handle_open(hwnd: HWND) {
    let s = state_ptr(hwnd);
    if (*s).modified && !confirm_discard(hwnd) {
        return;
    }
    let mut buf = vec![0u16; 32768];
    let mut ofn: OPENFILENAMEW = std::mem::zeroed();
    ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
    ofn.hwndOwner = hwnd;
    ofn.lpstrFile = PWSTR(buf.as_mut_ptr());
    ofn.nMaxFile = buf.len() as u32;
    ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST;

    if !GetOpenFileNameW(&mut ofn).as_bool() {
        return;
    }
    let path_str: String = buf
        .iter()
        .take_while(|&&c| c != 0)
        .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
        .collect();
    let path = PathBuf::from(path_str);

    match std::fs::read_to_string(&path) {
        Ok(text) => {
            let wide = to_wide(&text);
            (*s).ignore_change = true;
            SetWindowTextW((*s).editor, PCWSTR(wide.as_ptr())).unwrap();
            (*s).ignore_change = false;
            (*s).modified = false;
            (*s).current_path = Some(path.clone());
            update_title(hwnd, &Some(path), false);
            let (ed, ln) = ((*s).editor, (*s).linenos);
            update_line_numbers(ed, ln);
        }
        Err(e) => {
            let msg = to_wide(&e.to_string());
            MessageBoxW(hwnd, PCWSTR(msg.as_ptr()), w!("Error"), MB_OK | MB_ICONERROR);
        }
    }
}

unsafe fn handle_save(hwnd: HWND, force_dialog: bool) {
    let s = state_ptr(hwnd);
    let existing = if force_dialog { None } else { (*s).current_path.clone() };

    let save_path: PathBuf = if let Some(p) = existing {
        p
    } else {
        let mut buf = vec![0u16; 32768];
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = hwnd;
        ofn.lpstrFile = PWSTR(buf.as_mut_ptr());
        ofn.nMaxFile = buf.len() as u32;
        ofn.Flags = OFN_OVERWRITEPROMPT;
        if !GetSaveFileNameW(&mut ofn).as_bool() {
            return;
        }
        buf.iter()
            .take_while(|&&c| c != 0)
            .map(|&c| char::from_u32(c as u32).unwrap_or('?'))
            .collect::<String>()
            .into()
    };

    let len = GetWindowTextLengthW((*s).editor) as usize;
    let mut buf = vec![0u16; len + 1];
    GetWindowTextW((*s).editor, &mut buf);
    let text = String::from_utf16_lossy(&buf[..len]).replace("\r\n", "\n");

    match std::fs::write(&save_path, text.as_bytes()) {
        Ok(_) => {
            (*s).modified = false;
            (*s).current_path = Some(save_path.clone());
            update_title(hwnd, &Some(save_path), false);
        }
        Err(e) => {
            let msg = to_wide(&e.to_string());
            MessageBoxW(hwnd, PCWSTR(msg.as_ptr()), w!("Error"), MB_OK | MB_ICONERROR);
        }
    }
}

unsafe fn handle_quit(hwnd: HWND) {
    let s = state_ptr(hwnd);
    if (*s).modified && !confirm_discard(hwnd) {
        return;
    }
    DestroyWindow(hwnd).unwrap();
}
