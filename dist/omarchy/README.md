# QuickAccent for Omarchy

Hold a letter, press Space, pick an accent character — `é è ê ë` typed
directly, on a US keyboard, in every app.

![QuickAccent](https://raw.githubusercontent.com/victormasson/QuickAccent/master/assets/icon-app-256.png)

QuickAccent is a standalone daemon (evdev grab + uinput injection), not a
Quickshell component. This plugin is the Omarchy front-end for it: a bar
widget showing whether the picker is armed, click to toggle.

## 1. Install the daemon

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
```

This installs `~/.local/bin/quickaccent`, a systemd user unit, a udev rule
for `/dev/input` + `/dev/uinput`, and adds you to the `input` group.
**Reboot once** afterwards so the user session picks up the new group.

## 2. Install the plugin

```bash
git clone https://github.com/victormasson/omarchy-plugin-quickaccent \
  ~/.config/omarchy/plugins/io.github.victormasson.quickaccent
omarchy plugin validate ~/.config/omarchy/plugins/io.github.victormasson.quickaccent
```

Then add the QuickAccent widget to your bar from the Omarchy bar settings.

## How it types accents

Characters your keyboard layout cannot produce (`é` on a US layout) are added
to the keymap at startup: QuickAccent generates the xkb option
`quickaccent:accents` in `~/.config/xkb` and enables it at runtime — through
`hyprctl eval 'hl.config({ input = { kb_options = "…" } })'` on Omarchy 4's Lua
config, or `hyprctl keyword` on a legacy config — keeping your existing
`kb_options` and confirming the result by reading it back. Accents are then
ordinary keystrokes sent through a uinput virtual keyboard — no clipboard, no
portal prompt, works in terminals and Electron apps alike. Your config files
are not modified; the option is re-applied on each start and after every
`configreloaded` (Omarchy reloads Hyprland on theme changes).

If you prefer it in your own config, add it to `~/.config/hypr/input.lua`,
repeating the options you already have (the Lua config replaces the value):

```lua
hl.config({
  input = {
    kb_options = "compose:caps,shift:both_capslock_cancel,quickaccent:accents",
  },
})
```

Note: `xdg-desktop-portal-hyprland` implements no `RemoteDesktop` portal, so
the keymap option is the only direct-typing route on Hyprland; without it
QuickAccent falls back to clipboard paste.

The picker opens centred on the focused window (`hyprctl activewindow`), so it
appears on the monitor you are typing on.

## Permissions and dependencies

| Needs | Why |
|-------|-----|
| Membership of group `input` | Reading `/dev/input/event*` to grab keys |
| `/dev/uinput` (udev rule) | Injecting the accent as a real keystroke |
| `hyprctl` | Enabling the keymap option and locating the focused window |
| `systemd --user` | Running and toggling the daemon |

Group `input` can read every keystroke on the machine — install only if you
trust the source. All code is MIT and auditable at
[victormasson/QuickAccent](https://github.com/victormasson/QuickAccent).

## Configuration

`~/.config/quickaccent/config.toml`, hot-reloaded:

```toml
languages = ["French", "German", "Spanish"]
# hold_delay_ms = 250
```

37 languages are available; see the
[main README](https://github.com/victormasson/QuickAccent#config).

## License

MIT
