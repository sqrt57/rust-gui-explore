# rust-gui-explore

A series of self-contained Rust GUI experiments. Each subdirectory is an independent crate exploring a different toolkit or pattern.

## Experiments

| Crate | Toolkit | Description |
|-------|---------|-------------|
| [fltk-text-editor](fltk-text-editor/) | [fltk-rs](https://github.com/fltk-rs/fltk-rs) | Minimal text editor with system-tray hide/restore |

## Frameworks on the radar

| Framework | Approach | Notes |
|-----------|----------|-------|
| [fltk-rs](https://github.com/fltk-rs/fltk-rs) | Bindings to FLTK (C++) | Lightweight, fast build, good Win32 integration |
| [egui](https://github.com/emilk/egui) | Immediate-mode, pure Rust | Easiest to get started; ~13M downloads; API changes between versions |
| [iced](https://github.com/iced-rs/iced) | Elm-inspired, declarative | Production-ready; powers System76's COSMIC desktop |
| [Slint](https://slint.dev) | Declarative with own markup | Native rendering; commercial license for closed-source |
| [Tauri](https://tauri.app) | WebView + Rust backend | Best fit if you want a web frontend; small binaries |
| [Dioxus](https://dioxuslabs.com) | React-like components | Desktop + web + mobile from one codebase |
| [gtk-rs](https://gtk-rs.org) | GTK 4 bindings | Mature; best on Linux; lower-level |
| [Xilem](https://github.com/linebender/xilem) | SwiftUI/Flutter-inspired | Experimental; API unstable |
| [floem](https://github.com/lapce/floem) | Reactive, lightweight | Early-stage; from the Lapce editor team |

## Goals

- Try different Rust GUI frameworks hands-on
- Keep each experiment small and self-contained
- Document what works and what doesn't

## License

MIT — see [LICENSE](LICENSE)
