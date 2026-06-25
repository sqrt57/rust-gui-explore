# slint-text-editor

A minimal text editor built with Rust and [Slint](https://slint.dev) 1.x.

## Features

- Open and save plain text files
- Line numbers displayed in a fixed-width pre-formatted rectangle
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts (handled in a `.slint` `FocusScope`)
- Close button hides the window to the system tray
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Notes

The build is two-stage: `build.rs` compiles `ui/app.slint` into Rust, and the generated `AppWindow` type is imported via `slint::include_modules!()`. All UI state lives in `.slint` properties; Rust only sets properties and registers callbacks.

**License note**: Slint is free for open-source use; commercial closed-source use requires a paid license.

## Build & run

```powershell
cd slint-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

## Project structure

```
src/
  main.rs     — Rust logic, callbacks, tray
ui/
  app.slint   — declarative UI, properties, keyboard shortcuts
build.rs      — compiles app.slint at build time
```

## License

MIT (app code) — Slint framework: see [Slint licensing](https://slint.dev/pricing)
