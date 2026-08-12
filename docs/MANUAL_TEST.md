# Manual E2E checklist

Automated CI covers state machine, mappings, config, and pure helpers.  
The following needs a real desktop session (permissions + compositor).

## Linux

- [ ] **Install** prebuilt or `cargo run --release`
- [ ] User in group `input` (re-login after `usermod`)
- [ ] **Hold `e` > hold_delay → Space** → overlay → cycle Space → release → inserts accent
- [ ] **Quick Space** before hold_delay → normal space (no overlay)
- [ ] **False start**: open overlay, release letter immediately → space replayed, no stuck key
- [ ] **Escape** cancels overlay
- [ ] **Arrows** cycle; **Shift** toggles upper/lower while selecting
- [ ] **Cooldown**: after inject, immediate `e`+Space is normal typing briefly
- [ ] **GNOME Wayland**: inject works after Remote Desktop / input portal approve
- [ ] **Native Wayland app** (e.g. GNOME Text Editor) and **XWayland app** both get accents
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
- [ ] `hold_delay_ms` / `input_time_ms` feel right for false-start vs commit
