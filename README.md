# QuickAccent

A cross-platform accent character picker for Linux and macOS, inspired by [PowerAccent](https://learn.microsoft.com/en-us/windows/powertoys/quick-accent) from Windows PowerToys.

Hold a letter key, press space to open a selection overlay with accent variants, press space to cycle through them, and release the letter to insert the selected character.

## How It Works

1. Hold a letter key (e.g. `e`)
2. Press `Space` — an overlay appears with accent variants (e.g. `é è ê ë`)
3. Press `Space` again to cycle through options
4. Release the letter key to insert the selected accent
5. Press `Escape` to cancel

## Installation

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- macOS or Linux

### macOS

On macOS, the app uses CoreGraphics event taps for keyboard interception. You must grant **Accessibility** permissions to the terminal or app running QuickAccent (System Settings > Privacy & Security > Accessibility).

### Linux

On Linux, the app uses `rdev` with the `unstable_grab` feature. You may need to run with elevated privileges or add your user to the `input` group.

### Build

```bash
cargo build --release
```

The binary will be at `target/release/quickaccent`.

## Configuration

On first run, a default config file is created at: `~/.config/quickaccent/config.toml`

### Example config

```toml
# Select which languages to include for accent variants.
# Variants from all selected languages are merged together.
languages = ["French", "German", "Spanish"]
```

### Available Languages

Catalan, CrimeanTatar, Croatian, Czech, Danish, Dutch, Esperanto, Estonian, Finnish, French, German, Greek, Hungarian, IPA, Iceland, Irish, Italian, Kurdish, Lithuanian, Maltese, Maori, Norwegian, Pinyin, Polish, Portuguese, ProtoIndoEuropean, Romanian, Romanization, ScottishGaelic, Serbian, Slovak, Slovenian, Spanish, Swedish, Turkish, Vietnamese, Welsh

## Usage

```bash
# Run with default config
./target/release/quickaccent

# Run with debug logging
RUST_LOG=debug ./target/release/quickaccent
```

The app runs as a background daemon with no visible window until an accent selection is triggered.

## Tech Stack

- **[iced](https://github.com/iced-rs/iced)** — UI overlay (daemon mode, dynamic window)
- **CoreGraphics** (macOS) / **rdev** (Linux) — global keyboard interception
- **[enigo](https://github.com/enigo-rs/enigo)** — cross-platform character injection
- **tokio** — async channel communication between grab thread and UI

## License

MIT
