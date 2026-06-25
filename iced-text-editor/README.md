# iced-text-editor

A minimal text editor built with Rust and [iced](https://github.com/iced-rs/iced) 0.13 (Elm MVU architecture).

## Features

- Open and save plain text files
- Line numbers rendered as a separate widget column
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts
- Close button hides the window to the system tray
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Notes

File dialogs use `rfd::AsyncFileDialog` wrapped in iced `Task`s. The `tokio` feature on iced is required for the `time::every` subscription used to poll tray events.

## Build & run

```powershell
cd iced-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

## Project structure

```
src/
  main.rs   — Message enum, Model, update, view, subscriptions
```

## License

MIT
