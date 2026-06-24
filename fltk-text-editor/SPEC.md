# fltk-text-editor — spec

Derived from the current implementation.

## Window

- Size: 900 × 650, resizable
- GTK scheme (`app::Scheme::Gtk`)
- Title bar format:
  - No file open: `fltk-text-editor`
  - File open, unmodified: `filename — fltk-text-editor`
  - File open, modified: `• filename — fltk-text-editor`
  - No file, modified: `• Untitled — fltk-text-editor`

## Layout

```
┌─────────────────────────────┐
│ Menu bar (full width, 25px) │
├─────────────────────────────┤
│                             │
│  Text editor (remainder)    │
│                             │
└─────────────────────────────┘
```

Both regions resize with the window.

## Text editor

- Font: Courier (monospace), 14 pt
- Line numbers: 48 px gutter
- Word wrap: at window bounds

## File menu

| Item | Shortcut | Behaviour |
|------|----------|-----------|
| New | Ctrl+N | Confirm discard if modified → clear buffer, reset path |
| Open… | Ctrl+O | Confirm discard if modified → file-open dialog → read UTF-8 file |
| Save | Ctrl+S | Save to current path; if no path, behaves like Save As |
| Save As… | Ctrl+Shift+S | Save-as dialog → write file, update current path |
| *(separator)* | | |
| Quit | Ctrl+Q | Confirm discard if modified → exit |

## Unsaved-changes guard

Any file operation that would discard edits (New, Open, Quit) shows a modal dialog:

> "You have unsaved changes. Discard them?"
> `[Cancel]` `[Discard]`

Proceeding requires explicit "Discard". Cancelling aborts the operation.

## System tray

- Icon: solid blue 32 × 32 px square (RGBA `#268bd2`)
- Tooltip: `fltk-text-editor`
- Tray menu:
  - **Show / Hide** — toggle window visibility
  - *(separator)*
  - **Quit** — confirm discard if modified → exit

## Close button

Clicking the window's × button hides the window to the tray instead of quitting.

## Tray icon click

Left-click on the tray icon toggles window visibility (same as Show / Hide menu item).

## Window hide / show

Hiding and showing is done via Win32 `ShowWindow` / `IsWindowVisible` / `SetForegroundWindow` rather than FLTK's `Window::hide()`, to keep the FLTK event loop alive while the window is invisible.
