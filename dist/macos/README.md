# QuickAccent on macOS

## Install (prebuilt, no Rust)

```bash
curl -fsSL https://raw.githubusercontent.com/victormasson/QuickAccent/master/dist/macos/install.sh | bash
open ~/Applications/QuickAccent.app
```

Grant **Accessibility** (System Settings → Privacy & Security → Accessibility).

Downloads `QuickAccent-macos-universal.tar.gz` from the rolling [`continuous`](https://github.com/victormasson/QuickAccent/releases/tag/continuous) release (arm64 + x86_64).

| Env | Default |
|-----|---------|
| `GITHUB_REPO` | `victormasson/QuickAccent` |
| `QUICKACCENT_VERSION` | `continuous` |
| `PREFIX` | `~/Applications` |
| `INSTALL_FROM_SOURCE` | `0` |

## Homebrew (from source)

```bash
brew install --HEAD ./dist/brew/Formula/quickaccent.rb
```

## From source

```bash
INSTALL_FROM_SOURCE=1 ./dist/macos/install.sh
# or
cargo build --release
```
