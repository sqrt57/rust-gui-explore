# nitrogen-daisy

A simple text editor built with Rust and [FLTK](https://github.com/fltk-rs/fltk-rs).

## Features

- Open and save plain text files
- Basic editing: type, select, cut, copy, paste
- Unsaved-changes indicator in the title bar
- Minimal UI — single window, menu bar, text area

## Requirements

- Rust 1.70+
- A C++ compiler (MSVC, GCC, or Clang) — required by the `fltk` crate to build the FLTK library

On Windows the easiest path is to install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and make sure `cl.exe` is on the PATH, or use the MSVC toolchain via `rustup`.

## Build & run

```sh
cargo run
```

The first build will compile FLTK from source and take a few minutes. Subsequent builds are fast.

## Usage

| Action | How |
|---|---|
| New file | File → New |
| Open file | File → Open… |
| Save | File → Save |
| Save as | File → Save As… |
| Quit | File → Quit |

## Project structure

```
src/
  main.rs   — entry point, window setup, event loop
```

## License

MIT
