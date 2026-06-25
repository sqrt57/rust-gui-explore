# tauri-text-editor

A minimal text editor built with Rust and [Tauri](https://tauri.app) 2.x. The frontend is plain HTML/CSS/JS rendered in WebView2; the Rust backend exposes commands for file I/O, dialogs, and tray state.

## Features

- Open and save plain text files
- Line numbers rendered as a `<div>` alongside `<textarea>`, scroll-synced via `scrollTop`
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts
- Close button hides the window to the system tray
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Requirements

- Windows with [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) runtime (pre-installed on Windows 10 1803+ and Windows 11)

## Build & run

Run from the `src-tauri/` subdirectory:

```powershell
cd tauri-text-editor\src-tauri
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

## Project structure

```
frontend/
  index.html           — HTML/CSS/JS UI; all editing state lives here
src-tauri/
  src/
    main.rs            — Tauri setup, tray, window event handlers
    lib.rs             — Tauri commands: read_file, write_file, show_open_dialog,
                         show_save_dialog, quit_app, update_state
  tauri.conf.json      — app config; frontendDist points to ../frontend
  capabilities/
    default.json       — Tauri capability grants
```

## License

MIT
