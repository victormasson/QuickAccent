# Manual E2E checklist

Automated CI covers state machine, mappings, config, and pure helpers.  
The following needs a real desktop session (permissions + compositor).

## Linux

- [ ] **Install** prebuilt or `cargo run --release`
- [ ] User in group `input` + `uinput` module loaded (reboot after `usermod`)
- [ ] GNOME: startup log says "keyboard layout extended with N accent
      characters"; `gsettings get org.gnome.desktop.input-sources xkb-options`
      contains `quickaccent:accents`; previous options (e.g. caps:escape) kept
- [ ] No permission dialog at any point; normal typing unaffected after the
      keymap reload
- [ ] Accents are **typed directly** — clipboard content is untouched after
      inserting an accent
- [ ] **Normal typing**: letters appear on key *release*; fast typing
      ("bonjour évidemment") has no swapped/doubled/lost letters (rollover)
- [ ] **Quick `e`+Space** before hold_delay → "e " typed, no overlay
- [ ] **Hold `e`** (> hold_delay, no Space) → nothing typed, no auto-repeat;
      release → single "e". A letter without variants still auto-repeats
- [ ] **Hold `e` → Space** → overlay opens, no "e" ever typed → cycle
      Space/arrows → release → accent inserted, no cursor movement
- [ ] Works in: GNOME Text Editor, gnome-terminal, VS Code, Firefox,
      xterm (XWayland), LibreOffice
- [ ] **Shift**: overlay flips to uppercase; commit inserts É directly
- [ ] **Escape** closes overlay and types the plain letter
- [ ] **Shortcuts unaffected**: Ctrl+E, Ctrl+C, Alt+Tab, Super
- [ ] **Hotplug**: connect a second keyboard (USB or Bluetooth) mid-session →
      suppression + accents work on it too
- [ ] Without `input` group: clean error at startup, typing is NOT eaten
- [ ] **Multi-monitor** (GNOME, after one re-login): overlay opens on the
      monitor of the focused window; without the shell extension it stays
      centered on the primary monitor
- [ ] **AZERTY** (or non-QWERTY): accents match the character typed, not US physical key
- [ ] Config hot-reload: edit `~/.config/quickaccent/config.toml` languages without restart

## macOS

- [ ] Grant **Accessibility**
- [ ] Same hold / Space / cycle / release flow
- [ ] Menu bar icon + Quit
- [ ] LaunchAgent / brew service starts at login (if installed that way)

## Config knobs

- [ ] `activation_key = "Space"` — arrows do not open overlay
- [ ] `activation_key = "LeftRightArrow"` — Space does not open overlay
- [ ] `hold_delay_ms` feels right (Space before/after the delay); `input_time_ms` is macOS-only
