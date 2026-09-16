# QuickAccent for Omarchy

Hold a letter, press Space, pick an accent character — `é è ê ë` typed
directly, on a US keyboard, in every app.

![QuickAccent](https://raw.githubusercontent.com/victormasson/QuickAccent/master/assets/icon-app-256.png)

QuickAccent is a standalone daemon (evdev grab + uinput injection), not a
Quickshell component. This plugin is the Omarchy front-end for it: a bar
widget showing whether the picker is armed, click to toggle.

## 1. Install the daemon

The daemon is packaged for Arch as [`quickaccent-bin`](https://aur.archlinux.org/packages/quickaccent-bin)
(sources in [`dist/arch`](https://github.com/victormasson/QuickAccent/tree/master/dist/arch)
of the main repo). The PKGBUILD pins the release tarball by sha256, and the
release itself carries a Sigstore build-provenance attestation, so the binary
can be checked against what CI built from the tagged commit.

```bash
yay -S quickaccent-bin           # or: omarchy pkg add quickaccent-bin
sudo usermod -aG input "$USER"   # read keyboards + open /dev/uinput
```

**Reboot once** so the user session picks up the `input` group, then:

```bash
systemctl --user enable --now quickaccent
```

To check the release asset the package is pinned to (optional, needs
`gh auth login`):

```bash
gh release download v1.3.0 --repo victormasson/QuickAccent -p 'quickaccent-linux-x86_64.tar.gz' -p SHA256SUMS
sha256sum -c --ignore-missing SHA256SUMS
gh attestation verify quickaccent-linux-x86_64.tar.gz --repo victormasson/QuickAccent
yay -G quickaccent-bin && grep sha256sums quickaccent-bin/PKGBUILD   # same digest as SHA256SUMS
```

The package installs `/usr/bin/quickaccent`, a systemd user unit, a udev rule
for `/dev/input` + `/dev/uinput` and a `modules-load.d` entry for `uinput`.
Other distributions: see the
[QuickAccent README](https://github.com/victormasson/QuickAccent#install).

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

## Removal

Remove the plugin:

```bash
omarchy plugin disable io.github.victormasson.quickaccent 2>/dev/null || true
rm -rf ~/.config/omarchy/plugins/io.github.victormasson.quickaccent
```

Uninstall the daemon (optional — it is a separate program):

```bash
systemctl --user disable --now quickaccent
sudo pacman -Rns quickaccent-bin
sudo gpasswd -d "$USER" input      # if nothing else needs the group
rm -rf ~/.config/xkb/symbols/quickaccent ~/.config/quickaccent
# drop the runtime keymap option Hyprland is holding
hyprctl keyword input:kb_options "$(hyprctl getoption input:kb_options -j | \
  python3 -c 'import json,sys;print(",".join(o for o in json.load(sys.stdin)["str"].split(",") if o!="quickaccent:accents"))')" 2>/dev/null || true
```

## License

MIT
