# Changelog

All notable changes to QuickAccent are documented here.
This project adheres to [Semantic Versioning](https://semver.org/).

## [1.1.0] - 2026-08-18

Omarchy / Hyprland support.

### Added

- **Hyprland is a first-class desktop.** The keymap extension that makes
  accents typeable on layouts without them (`é` on US) is now enabled with
  `hyprctl keyword input:kb_options` instead of GNOME's gsettings. It applies
  immediately — no re-login — and `hyprland.conf` is left untouched, because
  QuickAccent re-applies the option on every start.
- The keyboard layout and xkb options are read from Hyprland
  (`input:kb_layout` / `kb_variant` / `kb_options`) when it is the compositor,
  ahead of the GNOME and `localectl` sources.
- The picker follows the focused window on Hyprland via
  `hyprctl activewindow`, so it opens on the monitor being typed on. The GNOME
  helper extension is no longer installed there.
- An Omarchy bar-widget plugin (`dist/omarchy/`): shows whether the daemon is
  armed and toggles it. Ready to publish to omarchyplugins.com.
- `LICENSE` (MIT) — previously only declared in `Cargo.toml` and the README.

## [1.0.0] - 2026-08-18

First stable release. QuickAccent brings macOS-style press-and-hold accent
picking to Linux and macOS: hold a letter, press Space, pick a variant,
release to insert.

### Typing

- **Press-and-hold picker** — hold a letter, press Space (or the arrow keys)
  to open the overlay, cycle with Space/arrows, release the letter to insert.
  Escape cancels and types the plain letter.
- **The letter is never typed before you choose.** Accent-capable letters are
  held back on key-down and appear on key-release, exactly like macOS
  press-and-hold, so there is no character to delete and no cursor jump.
- **Shift** switches the overlay between lower- and uppercase variants live.
- **37 languages**, merged and hot-reloaded from
  `~/.config/quickaccent/config.toml` without restarting.

### Linux

- **Direct injection, no clipboard, no permission prompts.** Accents are typed
  as real keystrokes through a uinput virtual keyboard, so they work in every
  app — native Wayland, XWayland, Electron and terminals alike.
- **Characters missing from your keyboard layout** (`é` on US, `É` on AZERTY…)
  are added to the keymap at startup through a generated XKB option
  (`quickaccent:accents` in `~/.config/xkb`); existing options are preserved.
- **The picker follows the monitor you are typing on** (GNOME) via a
  self-installed micro shell extension that reports the focused window.
- **Layout aware** — AZERTY, QWERTZ, Dvorak and friends resolve accents by the
  character produced, not by physical key position.
- Works on Wayland and X11; the overlay renders through XWayland so it never
  steals keyboard focus from the app you are typing in.
- Hot-plugged keyboards (USB re-plug, Bluetooth reconnect) are picked up
  mid-session.
- Falls back to XDG RemoteDesktop keysym injection, then clipboard paste, when
  the keymap route is unavailable.

### macOS

- Menu bar item with a template icon that follows light and dark menu bars.
- Accessory app (no Dock icon), universal binary for Apple Silicon and Intel.

### Install

- Prebuilt binaries for Linux x86_64 and macOS universal, installed by
  `dist/linux/install.sh` / `dist/macos/install.sh`, with a systemd user unit
  on Linux and a LaunchAgent-style `.app` bundle on macOS.
- Homebrew formula for building from source on macOS.
- Application icons on both platforms.

### Known limitations

- The multi-monitor overlay needs one log out/in after first install so GNOME
  loads the helper extension; without it the overlay is centered on the
  primary monitor.
- The clipboard fallback pastes with Ctrl+V, which terminals treat literally.
  It is only reached when neither the keymap nor the portal route is
  available.
- The overlay is centered rather than placed at the caret — Wayland
  compositors do not expose caret position to applications.

[1.1.0]: https://github.com/victormasson/QuickAccent/releases/tag/v1.1.0
[1.0.0]: https://github.com/victormasson/QuickAccent/releases/tag/v1.0.0
