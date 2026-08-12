# QuickAccent

Cross-platform accent picker for Linux and macOS, inspired by [PowerAccent](https://learn.microsoft.com/en-us/windows/powertoys/quick-accent) (PowerToys).

Hold a letter → Space → pick an accent → release to insert.

## How it works

1. Hold a letter (e.g. `e`)
2. Press `Space` — overlay shows variants (`é è ê ë`…)
3. `Space` / arrows cycle; release the letter to insert
4. `Escape` cancels

## Install

**Prerequisites:** Rust ([rustup](https://rustup.rs/)), macOS or Linux.

### macOS

Grant **Accessibility** (System Settings → Privacy & Security).

```bash
brew install --HEAD ./dist/brew/Formula/quickaccent.rb
# or: cargo build --release
```

### Linux

```bash
./dist/linux/install.sh
```

Uses evdev grab + enigo (Wayland virtual-keyboard / libei on GNOME / X11). Details: [dist/linux/README.md](dist/linux/README.md).

Requires group `input` (re-login after install). That group can read all keyboards — trusted users only.

### From source

```bash
cargo build --release   # → target/release/quickaccent
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
./target/release/quickaccent
RUST_LOG=debug ./target/release/quickaccent
```

Runs as a background daemon. On Linux Wayland the overlay is centered (compositors block caret-relative placement).

## Stack

- [iced](https://github.com/iced-rs/iced) — overlay
- CoreGraphics (macOS) / rdev (Linux) — grab
- [enigo](https://github.com/enigo-rs/enigo) — inject (VK / libei / X11)
- xkbcommon (Linux) — layout-aware accents

## License

MIT
