# fltk-text-editor

A minimal text editor built with Rust and [FLTK](https://github.com/fltk-rs/fltk-rs) (fltk-rs 1.x).

## Features

- Open and save plain text files
- Line numbers via FLTK's built-in gutter (`set_linenumber_width`)
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts
- Close button hides the window to the system tray
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Requirements

- Rust 1.85+
- A C++ compiler — required by `fltk` to compile the FLTK library from source
  - On Windows: install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and ensure `cl.exe` is on the PATH

## Build & run

```powershell
cd fltk-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

The first build compiles FLTK from source and takes a few minutes. Subsequent builds are fast.

## Project structure

```
src/
  main.rs   — window setup, menu, tray, event loop
```

## License

MIT
