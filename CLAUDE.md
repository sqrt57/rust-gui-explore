# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository structure

Six independent crates, each a self-contained binary implementing the same text editor spec (`SPEC.md`). There is no workspace — each crate has its own `Cargo.toml` and `Cargo.lock`. Work inside one crate at a time.

| Crate | Framework | Architecture |
|-------|-----------|--------------|
| `fltk-text-editor` | fltk-rs (C++ bindings) | Callback-based, FLTK idle loop |
| `egui-text-editor` | eframe 0.29 | Immediate-mode, background tray thread |
| `iced-text-editor` | iced 0.13 | Elm MVU, subscriptions + async Tasks |
| `slint-text-editor` | Slint 1.x | Declarative `.slint` markup + Rust callbacks |
| `tauri-text-editor` | Tauri 2.x | WebView2 frontend (HTML/JS) + Rust command layer |
| `dioxus-text-editor` | Dioxus 0.6 | React-like signals + WebView (Wry) desktop |

## Build and run

All commands run from inside the crate directory:

```powershell
cd slint-text-editor   # or whichever crate
cargo build            # debug build
cargo run              # run (console visible)
cargo run --release    # release build (no console; windows_subsystem = "windows")
```

**fltk-text-editor** requires a C++ compiler on first build (FLTK compiles from source via `fltk-bundled`).

**slint-text-editor** has a two-stage build: `build.rs` compiles `ui/app.slint` → generates Rust code → binary compiles. The generated type `AppWindow` is imported via `slint::include_modules!()`.

**tauri-text-editor** has a different layout: `src-tauri/` holds the Rust crate; `frontend/index.html` is the HTML/JS UI. Run from `src-tauri/`: `cargo run`. The `frontendDist` in `tauri.conf.json` points to `../frontend` — keeping it in a subdirectory prevents `tauri::generate_context!()` from scanning the `target/` directory. `gen/` (Tauri-generated capability schemas) is gitignored.

**dioxus-text-editor** uses Dioxus 0.6 desktop, which renders into a Wry WebView. Run from `dioxus-text-editor/`: `cargo run`.

There are no tests and no lint scripts beyond `cargo check` / `cargo clippy`.

## Shared spec

Every experiment implements `SPEC.md` identically:
- 900×650 resizable window; title shows `• filename — App` when modified
- File menu: New/Open/Save/Save As/Quit with standard shortcuts
- Unsaved-changes guard: modal confirm before any destructive action
- Close button hides to tray; left-click tray icon or Show/Hide menu item toggles visibility
- System tray: solid blue 32×32 icon (`#268bd2`), built at runtime as RGBA bytes

## Windows-specific patterns

**Console suppression**: All crates use `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` so `cargo run` shows a console in debug but not in release.

**Window hide/restore**: The close button must hide the window, not quit. Each framework solves this differently:
- **fltk/egui**: Win32 `ShowWindow(hwnd, SW_HIDE/SW_RESTORE)` — use `SW_RESTORE (9)`, not `SW_SHOW (5)`, to properly restore a `SW_HIDE`-hidden window
- **iced**: `window::change_mode(id, window::Mode::Hidden)`
- **slint**: Return `CloseRequestResponse::HideWindow` from `on_close_requested`
- **tauri**: `api.prevent_close()` inside `on_window_event` → `WindowEvent::CloseRequested`, then `window.hide()`
- **dioxus**: `WindowCloseBehaviour::LastWindowHides` in `Config::new()` — the runtime handles hiding automatically

**Tray event polling**: fltk/egui/iced/slint poll `tray-icon` events at ~16 ms via an idle callback, subscription timer, or `slint::Timer`. Tauri uses built-in `on_menu_event` / `on_tray_icon_event` callbacks — no polling needed. Dioxus uses `use_tray_icon_event_handler` / `use_tray_menu_event_handler` hooks directly in the component — no polling needed. The tray icon and menu are built from the same RGBA byte buffer across all crates.

**egui threading**: eframe stops repainting when hidden, so egui uses a background thread that captures the HWND on first frame (via `raw-window-handle`) and manipulates it independently.

## Key implementation patterns

**Modified flag + title**: Each crate tracks `modified: bool` and `current_path: Option<PathBuf>`, regenerating the title string on every state change.

**Line numbers**: Only fltk has built-in gutter support (`set_linenumber_width`). egui and iced render a separate column of widgets; slint builds a newline-joined string in a fixed-width rectangle; tauri and dioxus render a `<div>` alongside `<textarea>` with scroll-synced `scrollTop` (dioxus syncs via `eval()`).

**Discard guard**: Implemented as a pending-action enum (`PendingAction` / `Pending`) stored in state. The dialog confirms, then re-fires the deferred action.

**Slint UI/logic split**: All UI state lives in `.slint` properties; Rust only sets properties and registers callbacks. Dialog visibility is a boolean property (`show-discard-dialog`). Keyboard shortcuts are handled in a `FocusScope` inside the `.slint` file.

**iced async**: File operations use `rfd::AsyncFileDialog` wrapped in iced `Task`s. The `tokio` feature on iced is required for `time::every` subscriptions.

**Tauri UI/logic split**: All editing state (`modified`, `currentPath`, discard guard, line numbers) lives in JS. Rust exposes six commands: `update_state`, `read_file`, `write_file`, `show_open_dialog`, `show_save_dialog`, `quit_app`. The `update_state` mirror is needed so the tray Quit handler can decide whether to show the discard dialog without a JS round-trip. `Emitter` must be explicitly imported to call `window.emit()`. `Image::new` takes `&[u8]`, not `Cow`.

## Environment notes

- `CARGO_HOME` is `D:\Packages\cargo` (non-default location)
- Corporate TLS: `[http] check-revoke = false` in `~/.cargo/config.toml` suppresses schannel errors
