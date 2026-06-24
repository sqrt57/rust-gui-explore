# Text editor — spec

A minimal plain-text editor with system-tray integration.

## Window

- Default size: 900 × 650, resizable
- Title bar format:
  - No file open: `<app name>`
  - File open, unmodified: `filename — <app name>`
  - File open, modified: `• filename — <app name>`
  - No file, modified: `• Untitled — <app name>`

## Layout

```
┌─────────────────────────────┐
│ Menu bar (full width)       │
├─────────────────────────────┤
│                             │
│  Text editor (remainder)    │
│                             │
└─────────────────────────────┘
```

Both regions resize with the window.

## Text editor

- Monospace font, ~14 pt
- Line number gutter
- Word wrap at window bounds

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

Any operation that would discard edits (New, Open, Quit) shows a modal dialog:

> "You have unsaved changes. Discard them?"
> `[Cancel]` `[Discard]`

Proceeding requires explicit "Discard". Cancelling aborts the operation.

## System tray

- Icon: solid blue 32 × 32 px (`#268bd2`)
- Tooltip: app name
- Tray menu:
  - **Show / Hide** — toggle window visibility
  - *(separator)*
  - **Quit** — confirm discard if modified → exit

## Close button

Clicking × hides the window to the tray instead of quitting.

## Tray icon click

Left-click on the tray icon toggles window visibility (same as Show / Hide menu item).
