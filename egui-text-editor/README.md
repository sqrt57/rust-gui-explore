# egui-text-editor

A minimal text editor built with Rust and [egui](https://github.com/emilk/egui) / eframe 0.29.

## Features

- Open and save plain text files
- Line numbers rendered as a separate column of widgets
- Unsaved-changes indicator in the title bar (`• filename — App`)
- File menu with New / Open / Save / Save As / Quit and keyboard shortcuts
- Close button hides the window to the system tray
- System tray icon (solid blue 32×32); left-click or Show/Hide menu item toggles window visibility

## Notes

eframe stops repainting when the window is hidden, so tray events and Win32 window manipulation are handled on a dedicated background thread that captures the `HWND` on the first frame via `raw-window-handle`.

## Build & run

```powershell
cd egui-text-editor
cargo run            # debug build (console visible)
cargo run --release  # release build (no console)
```

## Project structure

```
src/
  main.rs   — app state, UI, background tray thread
```

## License

MIT
