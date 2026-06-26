# snap-rat

A terminal user interface (TUI) for browsing and managing snaps.

## Description

snap-rat is a Ratatui-based TUI that provides an interactive interface for:
- Browsing the Snap Store
- Installing, updating, and removing snaps
- Managing snap channels and classic confinement
- Viewing snap details, connections, and active changes
- Managing plug/slot connections

## Building

### Prerequisites

snap-rat statically links [libchafa](https://hpjansson.org/chafa/) for terminal image rendering. Install the development dependencies:

```bash
sudo apt install libchafa-dev libglib2.0-dev libsysprof-capture-4-dev pkg-config
```

### Build with Cargo

```bash
cargo build --release
```

The binary will be available at `target/release/snap-rat-vibes`.

### Run

```bash
cargo run
```

**Note:** Write operations (install, remove, etc.) require root privileges. snap-rat will attempt to escalate and re-exec when needed.

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Terminal manipulation
- [snapd-rs-artie](https://github.com/artiepoole/snapd-rs-artie) - Unofficial snapd API bindings
- [ratatui-image](https://github.com/benjajaja/ratatui-image) - Terminal image rendering with libchafa
- [tokio](https://tokio.rs/) - Async runtime

## Features

- Rich terminal graphics on supported terminals (Kitty, Sixel, iTerm2)
- Character-art fallback for minimal terminals
- Vim-style keyboard navigation
- Mouse support (click to select, double-click to execute)
- Real-time change tracking
- Interface connection management

## License

GPL-3.0
