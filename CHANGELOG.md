# Changelog

All notable changes to this project will be documented in this file.

The format follows Keep a Changelog, and this project uses semantic versioning.

## [1.0.3] - 2026-04-22

### Changed

- disabled default runtime logger initialization so the app stays silent unless logging is reintroduced explicitly
- removed per-key input logging to avoid recording typed key activity in logs
- refreshed `README.md` release metadata for `1.0.3`

## [1.0.2] - 2026-04-22

### Added

- added `blue_alps` sound profile for a sharper vintage click character

## [1.0.1] - 2026-04-21

### Changed

- bumped crate version to `1.0.1`
- refreshed `README.md` release metadata and command descriptions

### Notes

- maintenance release for documentation and packaging consistency

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
