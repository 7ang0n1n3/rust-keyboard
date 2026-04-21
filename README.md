# rust-keyboard

Wayland-first mechanical keyboard sound daemon for Arch Linux.

## What v1 does

`rust-keyboard` listens to Linux input devices through `evdev` and plays short synthesized keyboard clicks. This works on Wayland because it does not depend on compositor-specific global shortcuts or protocol support.

The tradeoff is access: the process needs permission to read `/dev/input/event*`.

## Why Wayland-first means `evdev`

On Linux, "global keyboard events" are straightforward on X11 and intentionally restricted on Wayland. For an Arch-focused v1, the least fragile approach is:

- read kernel input devices directly
- keep the app compositor-independent
- treat X11 support as incidental rather than architectural

This works under both Wayland and X11 sessions, but the design target is Wayland compatibility.

## Current scope

- detect readable keyboard devices
- play synthesized click sounds on key press
- configurable sound profiles: `apple`, `android`, `blue`, `brown`, `red`
- configurable device name filters
- small tray app for changing profile and volume
- `doctor` command for debugging Arch permissions

Not in v1 yet:

- tray icon or desktop UI
- packaged sound packs
- per-key sound samples
- compositor-specific overlays
- hotplug handling for newly attached keyboards

## Arch Linux setup

### 1. Install system packages

You will usually want:

```bash
sudo pacman -S base-devel pipewire
```

If you do not already have Rust installed:

```bash
sudo pacman -S rust cargo
```

### 2. Allow input-device access

The quickest route is adding your user to the `input` group:

```bash
sudo usermod -aG input "$USER"
```

Then log out and back in.

You can also use a dedicated udev rule if you want tighter access control.

### 3. Run diagnostics

```bash
cargo run -- doctor
```

### 4. Run the app

```bash
cargo run
```

The app starts as a tray process and starts the key-sound engine itself.

The tray now uses a direct StatusNotifierItem implementation instead of the appindicator-based `tray-icon` stack. Your desktop environment still needs an SNI-compatible tray host.

## Config

The config file is created automatically at:

```text
~/.config/rust-keyboard/config.toml
```

Default config:

```toml
[runtime]
backend = "evdev"
device_filters = []

[audio]
profile = "brown"
volume = 0.45
```

Use `device_filters` if you only want specific keyboards:

```toml
[runtime]
backend = "evdev"
device_filters = ["keychron", "zsa"]
```

## Notes

- This project is honest about Wayland constraints. It does not pretend that a portal or generic compositor API currently solves unrestricted global key capture.
- `evdev` sees physical input devices, so it may include external keyboards and built-in laptop keyboards at the same time.
- Reading `/dev/input` is powerful. Only grant access on systems where you trust the software you run.
