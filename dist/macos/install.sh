#!/usr/bin/env bash
# Install QuickAccent.app from GitHub Releases (universal binary).
set -euo pipefail

REPO="${GITHUB_REPO:-victormasson/QuickAccent}"
RELEASE_TAG="${QUICKACCENT_VERSION:-continuous}"
ASSET="QuickAccent-macos-universal.tar.gz"
APP_DIR="${PREFIX:-$HOME/Applications}"
# Piped to bash (curl | bash) there is no script file: BASH_SOURCE is unset,
# which `set -u` would report as an error. No source tree then — the release
# asset carries everything.
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd 2>/dev/null || true)"
else
  ROOT=""
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: this installer is for macOS" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

fetch_release() {
  local url="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${ASSET}"
  echo "==> Download $url"
  curl -fsSL "$url" -o "$tmpdir/$ASSET"
}

build_from_source() {
  echo "==> Build from source"
  if [[ -z "${ROOT:-}" || ! -f "$ROOT/Cargo.toml" ]]; then
    echo "error: source tree not found; clone the repo or set ROOT." >&2
    exit 1
  fi
  command -v cargo >/dev/null || { echo "error: install Rust (https://rustup.rs/)" >&2; exit 1; }
  (cd "$ROOT" && cargo build --release)
  APP="$tmpdir/QuickAccent.app"
  mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
  cp "$ROOT/target/release/quickaccent" "$APP/Contents/MacOS/quickaccent"
  chmod +x "$APP/Contents/MacOS/quickaccent"
  if [[ -f "$ROOT/dist/macos/QuickAccent.app/Contents/Info.plist" ]]; then
    cp "$ROOT/dist/macos/QuickAccent.app/Contents/Info.plist" "$APP/Contents/Info.plist"
  fi
  if [[ -f "$ROOT/dist/macos/AppIcon.icns" ]]; then
    cp "$ROOT/dist/macos/AppIcon.icns" "$APP/Contents/Resources/AppIcon.icns"
  fi
}

if [[ "${INSTALL_FROM_SOURCE:-0}" == "1" ]]; then
  build_from_source
  APP_SRC="$tmpdir/QuickAccent.app"
else
  if fetch_release; then
    tar -xzf "$tmpdir/$ASSET" -C "$tmpdir"
    APP_SRC="$(find "$tmpdir" -maxdepth 2 -type d -name 'QuickAccent.app' | head -1)"
    [[ -n "$APP_SRC" ]] || { echo "error: QuickAccent.app missing from archive" >&2; exit 1; }
  else
    echo "warning: download failed; falling back to source build" >&2
    build_from_source
    APP_SRC="$tmpdir/QuickAccent.app"
  fi
fi

echo "==> Install to $APP_DIR"
mkdir -p "$APP_DIR"
rm -rf "$APP_DIR/QuickAccent.app"
cp -R "$APP_SRC" "$APP_DIR/QuickAccent.app"

echo
echo "Installed $APP_DIR/QuickAccent.app"
echo "  • Grant Accessibility: System Settings → Privacy & Security → Accessibility"
echo "  • Launch: open \"$APP_DIR/QuickAccent.app\""
echo "  • Or brew from source: brew install --HEAD ./dist/brew/Formula/quickaccent.rb"
