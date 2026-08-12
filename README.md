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

### Linux (x86_64)

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/linux/install.sh | bash
```

Requires group `input` (re-login after install). That group can read all keyboards — trusted users only.  
Details: [dist/linux/README.md](dist/linux/README.md).

### macOS (universal)

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

```bash
cargo build --release   # → target/release/quickaccent
# or: INSTALL_FROM_SOURCE=1 ./dist/linux/install.sh
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
```

Runs as a background daemon. On Linux Wayland the overlay is centered (compositors block caret-relative placement).

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
- xkbcommon (Linux) — layout-aware accents

## License

MIT
