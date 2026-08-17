# QuickAccent on Linux

## Install from CI (prebuilt, no Rust)

Every push to `master` publishes `quickaccent-linux-x86_64.tar.gz` on the
rolling [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous)
release.

```bash
# Fedora / GNOME Wayland
sudo dnf install -y wl-clipboard

curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
sudo reboot
```

Debian / Ubuntu: `sudo apt install wl-clipboard` then the same `curl | bash`.

From a clone (still downloads the CI asset, does not compile):

```bash
./dist/linux/install.sh
```

| Env | Default | Meaning |
|-----|---------|---------|
| `GITHUB_REPO` | `victormasson/QuickAccent` | Repo for assets |
| `QUICKACCENT_VERSION` | `continuous` | Release tag |
| `PREFIX` | `~/.local` | Install prefix (`bin/`) |
| `INSTALL_FROM_SOURCE` | `0` | Set `1` to `cargo build` instead |

## Checklist (all desktops)

1. Group `input` + udev rule (`/dev/input/event*` and `/dev/uinput`) — the
   installer does this with sudo.
2. **Reboot** after being added to `input`. A GNOME logout does not restart
   `systemd --user`, so grab stays `Permission denied`.

Injection is always direct — no permission prompt, no clipboard. Accents your
layout lacks (`é` on US, `É` on AZERTY…) are **added to the keymap** at
startup: QuickAccent generates the xkb option `quickaccent:accents` in
`~/.config/xkb/` (mapping them onto spare keycodes F13–F24) and enables it in
GNOME's `xkb-options`; the compositor reloads the keymap live and the
characters are typed as ordinary keystrokes through uinput. Remove with
`gsettings reset org.gnome.desktop.input-sources xkb-options` (restores your
previous options minus ours) and delete `~/.config/xkb/symbols/quickaccent`.

Do not also enable the desktop autostart entry if the systemd user unit is
on — two instances fight over the evdev grab (`Device or resource busy`).

```bash
journalctl --user -u quickaccent -f
systemctl --user restart quickaccent.service
```

## What you need

| Need | Why |
|------|-----|
| Group `input` + udev rule | Evdev grab + uinput virtual keyboard |
| `uinput` module loaded | The installer adds `/etc/modules-load.d/uinput.conf` |
| Reboot after `usermod` | User systemd session picks up `input` |
| `wl-clipboard` (optional) | Emergency fallback only |
| x86_64 | Prebuilt asset (else build from source) |

**Security:** `input` can read all keystrokes. Trusted accounts only.

## How typing works (macOS-style)

- A letter with accent variants is held back until you release the key — it
  appears on key **release** (~a keystroke later when typing normally), and
  holding it does **not** auto-repeat, exactly like macOS press-and-hold.
- Hold the letter, press Space → the picker opens and the plain letter is
  never typed. Cycle with Space/arrows, release the letter to insert the
  accent — no backspace, no cursor jump.
- Escape closes the picker and types the plain letter.

## Injection backend (all sessions: Wayland, X11)

Most direct mechanism first — the clipboard is a last resort only:

| Case | Mechanism |
|------|-----------|
| Accent in the keymap — natively, or added by the auto-installed `quickaccent:accents` xkb option (GNOME) | uinput key combo — instant, no authorization, all apps incl. terminals |
| Keymap extension unavailable (non-GNOME desktops, slot overflow) | Portal keysym injection — one-time authorization, persisted via restore token. Note: mutter only types keysyms already in the keymap |
| Portal denied/unavailable | `wl-copy` + virtual Ctrl+V, loudly logged. Terminals treat Ctrl+V literally |

## From source

```bash
INSTALL_FROM_SOURCE=1 ./dist/linux/install.sh
# or
cargo build --release
```

Fedora build deps:

```bash
sudo dnf install -y gcc pkgconf-pkg-config libxkbcommon-devel libxkbcommon-x11-devel \
  libevdev-devel wayland-devel fontconfig-devel alsa-lib-devel \
  libX11-devel libXi-devel libXtst-devel libXcursor-devel libXrandr-devel \
  libXinerama-devel mesa-libEGL-devel mesa-libGL-devel vulkan-loader-devel
```
