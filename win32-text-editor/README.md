# win32-text-editor

Text editor implemented with raw Win32 API via [windows-rs](https://github.com/microsoft/windows-rs).

## Architecture

Single `WndProc` message loop with no GUI framework. All UI is constructed from stock Win32 controls:

- **Window**: `CreateWindowExW` with `WS_OVERLAPPEDWINDOW`
- **Menu**: `CreateMenu` / `AppendMenuW` / `SetMenu` with an accelerator table (`CreateAcceleratorTableW`)
- **Editor**: multiline `EDIT` control with `WS_VSCROLL` and word-wrap (no `WS_HSCROLL`)
- **Line numbers**: second read-only `EDIT` control, subclassed out of focus and scroll-synced via `EM_LINESCROLL`
- **Tray**: `Shell_NotifyIconW` with a `uCallbackMessage` back to `WndProc`; tray context menu via `CreatePopupMenu` + `TrackPopupMenu`
- **File dialogs**: `GetOpenFileNameW` / `GetSaveFileNameW`
- **Discard dialog**: `MessageBoxW` with `MB_YESNO`

## Scroll sync

`SetWindowSubclass` patches the editor `EDIT` control. On `WM_VSCROLL`, `WM_MOUSEWHEEL`, and `WM_KEYDOWN` the subclass proc queries `EM_GETFIRSTVISIBLELINE` on both controls and adjusts the gutter with `EM_LINESCROLL`.

## State

`AppState` is heap-allocated and stored in `GWLP_USERDATA` of the main window — the standard C pattern for per-window state, used directly here without any wrapper.

## Running

```powershell
cd win32-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```
