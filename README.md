<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/icon.svg">
    <img src="assets/icon-dark.svg" width="96" height="96" alt="QuickAccent">
  </picture>
</p>

# QuickAccent

Cross-platform accent picker for Linux and macOS, inspired by [PowerAccent](https://learn.microsoft.com/en-us/windows/powertoys/quick-accent) (PowerToys).

Hold a letter → Space → pick an accent → release to insert.

## How it works

1. Hold a letter (e.g. `e`) — like on macOS, the letter is held back, nothing
   is typed yet
2. Press `Space` — overlay shows variants (`é è ê ë`…)
3. `Space` / arrows cycle; release the letter to insert the accent directly
4. `Escape` cancels and types the plain letter

Just tapping a letter types it normally (it appears on key release); accented
letters don't auto-repeat while held, exactly like macOS press-and-hold.

## Install

Prebuilt binaries, no Rust needed: the stable
[latest release](https://github.com/victormasson/QuickAccent/releases/latest),
plus a rolling [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous)
prerelease rebuilt on every `master` push.

### Linux (Wayland or X11; GNOME tested) — prebuilt, no Rust

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
sudo reboot
```

The script downloads `quickaccent-linux-x86_64.tar.gz` from the
[`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous)
release and sets everything up:

- `~/.local/bin/quickaccent` + a systemd user unit (starts with your session)
- udev rule for `/dev/input` + `/dev/uinput`, loads the `uinput` module at boot
- adds you to group `input`

The **reboot** is required once so `systemd --user` picks up the new group
(a GNOME logout is not enough).

That's it — no permission prompt, no clipboard tricks. Accents are typed as
real keystrokes in every app (terminals included): characters your keyboard
layout lacks (é on US, É on AZERTY…) are added to the keymap automatically at
startup (xkb option `quickaccent:accents` in `~/.config/xkb`, your other xkb
options are preserved).

Notes:

- `input` group members can read all keyboards — trusted users only.
- Optional: `wl-clipboard` enables the emergency paste fallback on non-GNOME
  desktops.
- Uninstall / undo the keymap extension: remove `quickaccent:accents` from
  `gsettings get org.gnome.desktop.input-sources xkb-options`, delete
  `~/.config/xkb/symbols/quickaccent`, and `systemctl --user disable --now quickaccent`.

Details: [dist/linux/README.md](dist/linux/README.md).

### macOS (universal, no Rust)

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/macos/install.sh | bash
open ~/Applications/QuickAccent.app
```

Grant **Accessibility** (System Settings → Privacy & Security).  
Details: [dist/macos/README.md](dist/macos/README.md).

Homebrew (builds from source):

```bash
brew install --HEAD ./dist/brew/Formula/quickaccent.rb
```

### From source

Only if you cannot use the prebuilt (other CPU, or the release is missing):

```bash
# Fedora
sudo dnf install -y gcc pkgconf-pkg-config libxkbcommon-devel libxkbcommon-x11-devel \
  libevdev-devel wayland-devel fontconfig-devel alsa-lib-devel \
  libX11-devel libXi-devel libXtst-devel libXcursor-devel libXrandr-devel \
  libXinerama-devel mesa-libEGL-devel mesa-libGL-devel vulkan-loader-devel wl-clipboard

INSTALL_FROM_SOURCE=1 ./dist/linux/install.sh
# or: cargo build --release   → target/release/quickaccent
```

## Config

`~/.config/quickaccent/config.toml` (created on first run; hot-reloaded):

```toml
languages = ["French", "German", "Spanish"]
# hold_delay_ms = 250
```

**Languages:** Catalan, CrimeanTatar, Croatian, Czech, Danish, Dutch, Esperanto, Estonian, Finnish, French, German, Greek, Hungarian, IPA, Iceland, Irish, Italian, Kurdish, Lithuanian, Maltese, Maori, Norwegian, Pinyin, Polish, Portuguese, ProtoIndoEuropean, Romanian, Romanization, ScottishGaelic, Serbian, Slovak, Slovenian, Spanish, Swedish, Turkish, Vietnamese, Welsh

## Usage

```bash
quickaccent
RUST_LOG=debug quickaccent
journalctl --user -u quickaccent -f
```

Runs as a background daemon. On Linux the overlay renders through XWayland so
it never steals keyboard focus, and on GNOME it opens centered on the window
you are typing in (right monitor on multi-head setups) via a self-installed
micro shell extension — log out/in once after install to activate it;
without it the overlay is centered on the primary monitor.

## CI / releases

| Workflow | When | Output |
|----------|------|--------|
| [CI](.github/workflows/ci.yml) | every push / PR | `cargo test` + release build (Linux + macOS) |
| [Release](.github/workflows/release.yml) | push to `master` | Rolling assets on [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous) (prerelease) |
| [Release](.github/workflows/release.yml) | tag `v*` | Stable release, notes taken from [CHANGELOG.md](CHANGELOG.md) |

Assets: `quickaccent-linux-x86_64.tar.gz`, `QuickAccent-macos-universal.tar.gz`.

Cutting a release: update `CHANGELOG.md`, bump the version in `Cargo.toml`,
`dist/macos/QuickAccent.app/Contents/Info.plist` and the brew formula, then
`git tag vX.Y.Z && git push origin vX.Y.Z`.

Unit tests cover the accent state machine, mappings, config, and helpers.  
Desktop grab/inject: [docs/MANUAL_TEST.md](docs/MANUAL_TEST.md).

## Stack

- [iced](https://github.com/iced-rs/iced) — overlay
- CoreGraphics (macOS) / rdev, vendored with a hotplug-grab fix (Linux) — grab
- uinput virtual keyboard (Linux) — all injection; accents the layout lacks
  are added to the keymap via a generated xkb option (`~/.config/xkb`)
- XDG RemoteDesktop portal keysym (Linux, non-GNOME fallback) — one-time
  authorization, persisted via restore token
- [enigo](https://github.com/enigo-rs/enigo) (macOS) — inject
- `wl-copy` (Linux) — emergency fallback only
- xkbcommon (Linux) — layout-aware accents + char→keycode lookup

## License

MIT
