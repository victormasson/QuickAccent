# QuickAccent on Linux

## Install (prebuilt, no Rust)

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
# re-login if added to group `input`
```

From a clone:

```bash
./dist/linux/install.sh
```

Downloads `quickaccent-linux-x86_64.tar.gz` from the rolling [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous) release.

| Env | Default | Meaning |
|-----|---------|---------|
| `GITHUB_REPO` | `victormasson/QuickAccent` | Repo for assets |
| `QUICKACCENT_VERSION` | `continuous` | Release tag |
| `PREFIX` | `~/.local` | Install prefix (`bin/`) |
| `INSTALL_FROM_SOURCE` | `0` | Set `1` to `cargo build` instead |

## What you need

| Need | Why |
|------|-----|
| Group `input` + udev rule | Evdev keyboard grab |
| Portal approve (GNOME) | libei injection |
| x86_64 | Prebuilt asset (else build from source) |

**Security:** `input` can read all keystrokes. Trusted accounts only.

## Injection backends

| Session | Backend |
|---------|---------|
| GNOME Wayland | libei + XDG portal |
| KDE / Hyprland / Sway | Wayland virtual-keyboard |
| X11 | XTEST (not XWayland) |

## From source

```bash
INSTALL_FROM_SOURCE=1 ./dist/linux/install.sh
# or
cargo build --release
```

Build deps: `libxkbcommon`, `libevdev`, `wayland`, `fontconfig` (+ pkg-config / C toolchain).

```bash
RUST_LOG=debug quickaccent
journalctl --user -u quickaccent -f
```
