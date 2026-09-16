# Changelog

All notable changes to QuickAccent are documented here.
This project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- **Some accents never appeared in Chromium apps (Brave, VS Code, Teams…).**
  Characters beyond the F13–F23 slots were parked on evdev codes KEY_HP,
  KEY_QUESTION and KEY_ALTERASE, which Chromium's key table does not know,
  so it dropped the key events before looking up the character — û, ú, ü and
  ÿ on a French/US setup, for instance, while ù on KEY_BASSBOOST worked. The
  spare keys are now nine codes Chromium recognises but binds to nothing
  (KEY_BASSBOOST, KEY_PRINT, KEY_KPPLUSMINUS, KEY_PHONE, KEY_EXIT, KEY_REDO,
  KEY_SAVE, KEY_DOCUMENTS, KEY_BRIGHTNESS_AUTO), which also raises the keymap
  capacity from 60 to 80 characters. The keymap is regenerated on the next
  start.
- **Keymap slots go to the characters that matter.** Slots used to be handed
  out in codepoint order, so with a symbol set enabled € or ú could fall off
  the end while rarer letters got in. Languages now come first (in config
  order), then symbol sets, each in its own listed order, and caseless
  symbols share a slot two by two instead of wasting the Shift level. With
  French + Currency on a US layout everything but four currency signs fits.

## [1.3.0] - 2026-09-16

### Added

- **Custom palettes for "System" appearance.** `theme_light` / `theme_dark`
  in `config.toml` (and two pick lists under *Appearance* in Settings while
  *System* is selected) choose which palette a light or dark desktop maps
  to, e.g. Rosé Pine Dawn by day and Catppuccin Mocha by night.
- **Settings without a panel icon.** Launching QuickAccent while the daemon
  runs now opens its Settings window over D-Bus instead of exiting with
  "already running", so the launcher or dock entry works as the settings
  menu on Hyprland and other desktops without the GNOME top-bar menu. The
  desktop entry gains *Settings* and *Quit QuickAccent* actions,
  `quickaccent --settings` / `--quit` do the same from a terminal, and a
  right-click on the picker opens Settings.
- **GNOME top-bar menu.** The helper Shell extension now shows a *QuickAccent*
  panel button with *Settings…* and *Quit QuickAccent*, matching the macOS
  menu-bar item. Settings and quit go over D-Bus to the running daemon.
- **Named palettes.** Appearance in Settings is a theme selector: System /
  Light / Dark, plus Dracula, Catppuccin (Latte, Frappé, Macchiato, Mocha)
  and Rosé Pine (Main, Moon, Dawn). Values are stored as `theme = "..."` in
  `config.toml`. The picker overlay follows the same palette. On Linux,
  System follows GNOME `color-scheme`.
- **GNOME-style picker.** Rounded, translucent overlay on Linux and macOS
  (`overlay_opacity`, `overlay_radius`, `chip_radius`), with sliders in
  Settings. macOS still uses a glass/blur backdrop behind the panel.
- **Full Settings.** Hold delay, input time and activation key from
  `config.toml` are editable in Settings and apply immediately.

### Fixed

- **Daemon could silently lose its D-Bus name.** zbus requests names with
  replace-existing by default, so any second `quickaccent` process took the
  name and, once gone, left the daemon unreachable for `--settings` and the
  GNOME menu. The daemon now refuses replacement and keeps retrying the claim
  while a previous instance is still shutting down; `QUICKACCENT_DEMO`
  instances no longer touch the bus or the single-instance lock.
- **Hyprland: overlay missing or misplaced on some monitors.** The overlay is
  an X11 window, but Hyprland lays XWayland monitors out on its own terms
  (packed left to right from x=0 in monitor order, at physical size with
  `xwayland:force_zero_scaling`), so the logical coordinates from `hyprctl
  activewindow` only matched the X screen for a plain left-to-right layout at
  scale 1. A monitor above or below the first, out of order, or scaled sent
  the overlay elsewhere or off the X screen. The anchor is now mapped through
  XWayland's RandR layout (matched by output name) and the overlay is drawn at
  that monitor's scale. winit's own guess of an X11 scale factor, taken from
  the monitor under the mouse pointer, is pinned to 1 on Hyprland since it
  moved the window too.

## [1.2.0] - 2026-09-14

### Added

- **Verifiable Linux releases and an AUR package.** Releases now ship
  `SHA256SUMS` and a Sigstore build-provenance attestation per archive
  (`gh attestation verify <asset> --repo victormasson/QuickAccent`), and the
  Linux tarball carries the license. `dist/arch/` packages the tarball as
  `quickaccent-bin` (sha256-pinned; binary, user unit, udev rule,
  `modules-load.d` entry, desktop file and icons under `/usr`). The Omarchy
  plugin README installs the daemon from the AUR instead of piping a script
  to `bash`, which the Omarchy plugin marketplace rejects.

- **macOS: Liquid Glass picker.** The accent overlay now sits on a native
  glass backdrop (`NSGlassEffectView` on macOS 26+, a blurred
  `NSVisualEffectView` before that) with chips and text that follow the
  light/dark appearance, instead of an opaque dark panel.
- **Settings window.** The menu-bar icon gains *Settings…* (⌘,): a checkbox
  per language and symbol set, and an Appearance choice (System / Light /
  Dark, `theme = "..."` in `config.toml`) for the picker and the settings
  window. Changes apply immediately and rewrite only the affected line of
  `config.toml`, keeping your comments and other settings.
- `QUICKACCENT_DEMO=overlay|settings` opens that window at startup without
  taking the keyboard grab — for screenshots and UI work.

### Changed

- **Installers verify before installing.** `dist/linux/install.sh` and
  `dist/macos/install.sh` download the asset for the tag of the checkout they
  run from, check it against `SHA256SUMS` (and the attestation when `gh` is
  logged in) and refuse to continue on a mismatch. `curl | bash` is no longer
  documented anywhere; clone the tag and run the script. The shipped systemd
  unit defaults to `/usr/bin/quickaccent` (the user-local installer still
  rewrites `ExecStart`).

### Fixed

- **macOS: picker opened on the primary display when typing on another
  screen.** The focused-window lookup queried `AXFocusedApplication` on the
  system-wide accessibility element, which fails with
  `kAXErrorCannotComplete` on current macOS even for a trusted process, so the
  overlay always fell back to centering on the primary display. It now resolves
  the frontmost app through `NSWorkspace` and asks that app's own AX element
  for its focused window.
- **macOS: Accessibility grant never applied.** The release `.app` shipped
  without a bundle-level code signature, so TCC had no stable requirement to
  bind the grant to — toggling QuickAccent on in Accessibility did nothing and
  duplicate entries accumulated. The release workflow, installer and brew
  formula now ad-hoc sign the bundle. Launchers (brew service, LaunchAgent
  template) start the app via `open -a` instead of exec'ing the binary, which
  is the other case where TCC ignores the grant.

## [1.1.1] - 2026-09-02

Fixes for Omarchy 4 / Hyprland reported in
[#9](https://github.com/victormasson/QuickAccent/issues/9).

### Fixed

- **Deaf after suspend/resume.** On resume the kernel can return an error
  from the grabbed devices that ends the evdev event loop; the daemon then
  stayed alive but stopped seeing the keyboard, needing a manual restart. The
  grab is now supervised — it re-establishes itself (re-opening and
  re-grabbing every keyboard, which also recovers from device
  re-enumeration), and only a grab that keeps failing immediately exits so
  the service manager restarts it.

### Changed

- **Uppercase choice now sticks.** Releasing Shift before the letter used to
  flip the picker back to lowercase at the worst moment — the committed
  character. The case now latches uppercase whenever Shift is involved (held
  at the letter press or while choosing); a Shift release never downgrades,
  and pressing Shift while the picker is open toggles the case either way.
  This matches how plain letters already behaved.

### Fixed

- **Keymap option silently not enabled on Hyprland with a Lua config.**
  `hyprctl keyword` is rejected by the non-legacy parser ("Use eval") while
  `hyprctl` still exits 0, so the daemon believed the option was active and
  fell through to the clipboard path without a word. The option is now set
  with `hyprctl eval 'hl.config({ … })'` on Lua configs (`keyword` on legacy
  ones, each falling back to the other), the user's existing `kb_options` are
  carried forward, and success is decided solely by reading the value back.
  On failure the exact `input.lua` line — with the merged value — is logged.
- **Option lost on config reload.** Omarchy reloads Hyprland on every theme
  change, dropping runtime-set options. QuickAccent now watches Hyprland's
  event socket and re-applies the option after each `configreloaded`.
- **Dead-end portal advice.** The RemoteDesktop tier is only started when a
  portal backend actually implements it (Hyprland's does not); otherwise the
  log names the fallback in use and the keymap fix that applies to this
  desktop. The "accept the dialog" hint is GNOME-only now.
- **Accents committed into the picker instead of the app (Hyprland).** The
  overlay is a native-Wayland toplevel, so its X11 `override_redirect`
  focus-proofing did nothing on Hyprland — the compositor gave the overlay
  keyboard focus and the accent landed in it, not your document. QuickAccent
  now sets a `no_focus` window rule for the overlay via `hyprctl`, at startup
  and on every config reload. GNOME is unaffected (overlay runs on XWayland).
- **Lost Shift on accent-capable letters (Hyprland).** Letters QuickAccent
  replays go through its own virtual keyboard; Hyprland reports the emitting
  device's modifier state, so with Shift held on the physical keyboard the
  replayed vowels came out lowercase (`uPeRcaseS`). Shift/Ctrl/Alt/Meta/AltGr
  events are now mirrored onto the virtual keyboard as they pass through. Caps
  Lock is intentionally not mirrored (its action toggles on press); Caps Lock
  with accent letters on Hyprland remains a known gap.
- **Duplicate instances.** Launching QuickAccent from the app grid while the
  systemd service runs produced a second copy that silently lost the evdev
  grab (`EBUSY`), and the two were indistinguishable. A second instance now
  exits with "already running"; when the service itself starts and finds a
  stray holding the lock, it asks the stray to quit and takes over.

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

[1.1.1]: https://github.com/victormasson/QuickAccent/releases/tag/v1.1.1
[1.1.0]: https://github.com/victormasson/QuickAccent/releases/tag/v1.1.0
[1.0.0]: https://github.com/victormasson/QuickAccent/releases/tag/v1.0.0
