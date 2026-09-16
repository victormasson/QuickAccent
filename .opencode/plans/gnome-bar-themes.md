# GNOME top-bar + themes, then install `continuous`

Approved: macOS-style panel menu (Settings… / Quit); keep System/Light/Dark + Dracula, all Catppuccin, all Rosé Pine; push `master` → CI `continuous`; reinstall from GitHub (not local cargo).

## 1. GNOME panel + D-Bus

- `dist/linux/gnome-extension/extension.js`: keep FocusedWindow D-Bus; add `PanelMenu.Button` (`Q̄`), menu Settings… / Quit calling `io.github.victormasson.QuickAccent`.
- `metadata.json`: name **QuickAccent**, description covers the menu.
- New `src/dbus_service.rs`: zbus export `OpenSettings` / `Quit` on `/io/github/victormasson/QuickAccent`.
- `src/app.rs`: `UiEvent::Quit` → `iced::exit()`.
- `src/main.rs`: `mod dbus_service`; `dbus_service::start()` on Linux at startup.
- `src/shell_ext.rs`: on file change, `gnome-extensions disable` then `enable` to reload.

## 2. Themes

- Expand `ThemeChoice` in `src/config.rs`:
  - `system`, `light`, `dark`
  - `dracula`
  - `catppuccin-latte|frappe|macchiato|mocha`
  - `rose-pine`, `rose-pine-moon`, `rose-pine-dawn`
- New `src/theme.rs`: map to iced `Theme` (built-ins + `Theme::custom` for Rosé Pine); `is_dark`; Linux System via `org.gnome.desktop.interface color-scheme`; overlay chip/panel from `palette()`; macOS glass only for System/Light/Dark.
- Settings: `pick_list` instead of 3 radios.
- Overlay + settings both use `iced_theme()`.

## 3. Docs / tests

- README Config: GNOME top-bar Settings…; theme value list.
- CHANGELOG `[Unreleased]`.
- `dist/linux/README.md` extension description.
- Tests: theme parse round-trip, iced mapping, glass classification.

## 4. Ship and reinstall

```bash
# after commit + push master, wait for Release workflow
QUICKACCENT_VERSION=continuous ./dist/linux/install.sh
systemctl --user restart quickaccent.service
```

Must set `QUICKACCENT_VERSION=continuous` (checkout is still 1.2.0). Then disable/enable the GNOME extension (or log out) so the top-bar item loads.

No version bump, no `v*` tag, no AUR update.
