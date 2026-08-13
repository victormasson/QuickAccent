# QuickAccent

Cross-platform accent picker for Linux and macOS, inspired by [PowerAccent](https://learn.microsoft.com/en-us/windows/powertoys/quick-accent) (PowerToys).

Hold a letter → Space → pick an accent → release to insert.

## How it works

1. Hold a letter (e.g. `e`)
2. Press `Space` — overlay shows variants (`é è ê ë`…)
3. `Space` / arrows cycle; release the letter to insert
4. `Escape` cancels

## Install

Prebuilt binaries are published on every `master` push to the rolling
[`continuous` release](https://github.com/victormasson/QuickAccent/releases/tag/continuous).
Use that to avoid compiling on the laptop.

### Linux Wayland (GNOME, Fedora) — prebuilt, no Rust

```bash
# clipboard fallback for accents not on the current keymap
sudo dnf install -y wl-clipboard   # Debian/Ubuntu: sudo apt install wl-clipboard

curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
```

The script downloads `quickaccent-linux-x86_64.tar.gz` from the
[`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous)
release, installs `~/.local/bin/quickaccent`, a systemd user unit, the udev
rule, and adds you to group `input`.

Then:

1. **Reboot** (a GNOME logout is not enough — `systemd --user` keeps the old groups).
2. Approve the GNOME **Remote Desktop / input** portal when prompted.
3. Hold `e` → Space → release.

`input` can read all keyboards — trusted users only.  
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

Runs as a background daemon. On Linux Wayland the overlay is centered (compositors block caret-relative placement).

If inject fails on GNOME Wayland, install `wl-clipboard` — accents not on the
keymap are pasted via `wl-copy` + Ctrl+V.

## CI / releases

| Workflow | When | Output |
|----------|------|--------|
| [CI](.github/workflows/ci.yml) | every push / PR | `cargo test` + release build (Linux + macOS) |
| [Release](.github/workflows/release.yml) | push to `master` | Assets on [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous) |

Assets: `quickaccent-linux-x86_64.tar.gz`, `QuickAccent-macos-universal.tar.gz`.

Unit tests cover the accent state machine, mappings, config, and helpers.  
Desktop grab/inject: [docs/MANUAL_TEST.md](docs/MANUAL_TEST.md).

## Stack

- [iced](https://github.com/iced-rs/iced) — overlay
- CoreGraphics (macOS) / rdev (Linux) — grab
- [enigo](https://github.com/enigo-rs/enigo) — inject (VK / libei / X11)
- `wl-copy` (Linux Wayland) — fallback when the keymap has no accent key
- xkbcommon (Linux) — layout-aware accents

## License

MIT
