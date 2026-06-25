# rust-gui-explore

A series of self-contained Rust GUI experiments. Each subdirectory is an independent crate exploring a different toolkit or pattern.

## Experiments

| Crate | Toolkit | Description |
|-------|---------|-------------|
| [fltk-text-editor](fltk-text-editor/) | [fltk-rs](https://github.com/fltk-rs/fltk-rs) | Text editor — callback-based, FLTK idle loop |
| [egui-text-editor](egui-text-editor/) | [egui](https://github.com/emilk/egui) / eframe | Text editor — immediate-mode, background tray thread |
| [iced-text-editor](iced-text-editor/) | [iced](https://github.com/iced-rs/iced) | Text editor — Elm MVU, async Tasks |
| [slint-text-editor](slint-text-editor/) | [Slint](https://slint.dev) | Text editor — declarative .slint markup + Rust callbacks |
| [tauri-text-editor](tauri-text-editor/) | [Tauri](https://tauri.app) | Text editor — WebView2 frontend + Rust command layer |
| [dioxus-text-editor](dioxus-text-editor/) | [Dioxus](https://dioxuslabs.com) | Text editor — React-like signals, WebView desktop |

## Frameworks on the radar

| Framework | Approach | Notes |
|-----------|----------|-------|
| [fltk-rs](https://github.com/fltk-rs/fltk-rs) | Bindings to FLTK (C++) | ✓ done — lightweight, fast build, good Win32 integration |
| [egui](https://github.com/emilk/egui) | Immediate-mode, pure Rust | ✓ done — easiest to get started; ~13M downloads; API changes between versions |
| [iced](https://github.com/iced-rs/iced) | Elm-inspired, declarative | ✓ done — production-ready; powers System76's COSMIC desktop |
| [Slint](https://slint.dev) | Declarative with own markup | ✓ done — native rendering; commercial license for closed-source |
| [Tauri](https://tauri.app) | WebView + Rust backend | ✓ done — best fit if you want a web frontend; small binaries |
| [Dioxus](https://dioxuslabs.com) | React-like components | ✓ done — desktop + web + mobile from one codebase |
| [gtk-rs](https://gtk-rs.org) | GTK 4 bindings | Mature; best on Linux; lower-level |
| [Xilem](https://github.com/linebender/xilem) | SwiftUI/Flutter-inspired | Experimental; API unstable |
| [floem](https://github.com/lapce/floem) | Reactive, lightweight | Early-stage; from the Lapce editor team |
| [windows-rs](https://github.com/microsoft/windows-rs) | Raw Win32 API | Maximum control; Windows-only; verbose but no abstraction overhead |
| windows-rs + Direct2D/DirectWrite | Custom-rendered UI (Windows) | Hardware-accelerated 2D drawing and text; no widget system |
| windows-rs + WinRT / Windows App SDK | Modern native Windows UI | Microsoft's current recommended path; WinUI 3 widgets; COM-based |

## Goals

- Try different Rust GUI frameworks hands-on
- Keep each experiment small and self-contained
- Document what works and what doesn't

## License

MIT — see [LICENSE](LICENSE)
