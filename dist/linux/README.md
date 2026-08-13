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

## GNOME Wayland checklist

1. Group `input` + udev rule (`/dev/input/event*` and `/dev/uinput`) — the
   installer does this with sudo.
2. **Reboot** after being added to `input`. A GNOME logout does not restart
   `systemd --user`, so grab stays `Permission denied`.
3. Approve the **Remote Desktop / input** portal when GNOME asks.
4. Install `wl-clipboard`. Accents missing from the keymap (`é` on US, etc.)
   are injected with `wl-copy` + Ctrl+V.

Do not also enable the desktop autostart entry if the systemd user unit is
on — two instances fight over the evdev grab (`Device or resource busy`).

```bash
journalctl --user -u quickaccent -f
systemctl --user restart quickaccent.service
```

## What you need

| Need | Why |
|------|-----|
| Group `input` + udev rule | Evdev grab + uinput replay |
| Reboot after `usermod` | User systemd session picks up `input` |
| Portal approve (GNOME) | libei injection |
| `wl-clipboard` | Paste fallback for unmapped accents |
| x86_64 | Prebuilt asset (else build from source) |

**Security:** `input` can read all keystrokes. Trusted accounts only.

## Injection backends

| Session | Backend |
|---------|---------|
| GNOME Wayland | libei + XDG portal, then `wl-copy` + Ctrl+V if the key is not mapped |
| KDE / Hyprland / Sway | Wayland virtual-keyboard (same clipboard fallback) |
| X11 | XTEST (not XWayland) |

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
