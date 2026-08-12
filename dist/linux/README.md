# QuickAccent on Linux

```bash
./dist/linux/install.sh
# re-login if added to group `input`
```

## What you need

| Need | Why |
|------|-----|
| Group `input` + udev rule | Evdev keyboard grab (`rdev`) |
| Portal approve (GNOME) | libei character injection |
| Rust ≥ 1.85 | Build |

**Security:** `input` can read all keystrokes. Trusted accounts only.

## Injection backends (enigo)

| Session | Backend |
|---------|---------|
| GNOME Wayland | libei + XDG portal |
| KDE / Hyprland / Sway | Wayland virtual-keyboard |
| X11 | XTEST (real X11 only, not XWayland) |

## Layout

Keys are mapped through XKB (`XKB_DEFAULT_LAYOUT` / `localectl`). Overlay is centered (GNOME has no client-side window placement).

## Manual install

```bash
cargo build --release
cp target/release/quickaccent ~/.local/bin/
cp dist/linux/quickaccent.service ~/.config/systemd/user/
sudo cp dist/linux/60-quickaccent.rules /etc/udev/rules.d/
sudo usermod -aG input "$USER"   # re-login
systemctl --user enable --now quickaccent.service
```

Build deps: `libxkbcommon`, `libevdev`, `wayland`, `fontconfig` (+ pkg-config / C toolchain).

```bash
RUST_LOG=debug quickaccent
journalctl --user -u quickaccent -f
```
