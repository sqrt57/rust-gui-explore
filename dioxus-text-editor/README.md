# dioxus-text-editor

A minimal text editor built with Rust and [Dioxus](https://dioxuslabs.com) 0.6 (desktop feature). Dioxus renders a WebView window (via Wry) driven by React-like components and signals.

## Features

- Open and save plain text files
- Line numbers rendered as a `<div>` alongside `<textarea>`, scroll-synced via `eval()`
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts
- Close button hides the window to the system tray (`WindowCloseBehaviour::LastWindowHides`)
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Notes

Tray events are handled with the `use_tray_icon_event_handler` / `use_tray_menu_event_handler` hooks directly inside the component — no polling loop is needed. File operations use `rfd::AsyncFileDialog` via `spawn()`.

## Build & run

```powershell
cd dioxus-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

## Project structure

```
src/
  main.rs   — Dioxus app component, tray setup, action helpers, inline CSS
```

## License

MIT
