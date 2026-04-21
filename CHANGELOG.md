# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog, and this project uses semantic versioning.

## [1.0.0] - 2026-04-21

### Added

- initial Arch Linux release of `rust-keyboard`
- tray-first runtime model
- Wayland-friendly keyboard input capture using `evdev`
- synthesized keyboard sound engine
- sound profiles: `apple`, `android`, `blue`, `brown`, `red`
- tray controls for profile and volume changes
- persistent config support in `~/.config/rust-keyboard/config.toml`
- `doctor` and `dump-config` utility commands

### Changed

- default app execution now starts the tray app directly

### Notes

- initial Git commit: `db4cd50`
