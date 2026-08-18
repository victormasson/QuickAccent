#!/usr/bin/env bash
# Install QuickAccent for the current user (prebuilt binary from GitHub Releases).
set -euo pipefail

REPO="${GITHUB_REPO:-victormasson/QuickAccent}"
# Default: newest stable release. QUICKACCENT_VERSION=continuous pulls the
# rolling build from master; or pin a tag such as v1.0.0.
RELEASE_TAG="${QUICKACCENT_VERSION:-latest}"
ASSET="quickaccent-linux-x86_64.tar.gz"
BIN_DIR="${PREFIX:-$HOME/.local}/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_BASE="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor"
# Piped to bash (curl | bash) there is no script file: BASH_SOURCE is unset,
# which `set -u` would report as an error. No source tree then — the release
# asset carries everything.
if [[ -n "${BASH_SOURCE[0]:-}" ]]; then
  ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd 2>/dev/null || true)"
else
  ROOT=""
fi

arch="$(uname -m)"
if [[ "$arch" != "x86_64" && "$arch" != "amd64" ]]; then
  echo "error: prebuilt Linux binary is x86_64 only (this machine: $arch)." >&2
  echo "Build from source: INSTALL_FROM_SOURCE=1 $0" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
cleanup() { rm -rf "$tmpdir"; }
trap cleanup EXIT

fetch_release() {
  local url
  if [[ "$RELEASE_TAG" == "latest" ]]; then
    url="https://github.com/${REPO}/releases/latest/download/${ASSET}"
  else
    url="https://github.com/${REPO}/releases/download/${RELEASE_TAG}/${ASSET}"
  fi
  echo "==> Download $url"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$tmpdir/$ASSET"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$tmpdir/$ASSET" "$url"
  else
    return 1
  fi
}

build_from_source() {
  echo "==> Build from source"
  if [[ -z "${ROOT:-}" || ! -f "$ROOT/Cargo.toml" ]]; then
    echo "error: source tree not found; clone the repo or set ROOT." >&2
    exit 1
  fi
  command -v cargo >/dev/null || { echo "error: install Rust (https://rustup.rs/)" >&2; exit 1; }
  (cd "$ROOT" && cargo build --release)
  mkdir -p "$tmpdir/pkg"
  cp "$ROOT/target/release/quickaccent" "$tmpdir/pkg/"
  cp "$ROOT/dist/linux/quickaccent.service" "$tmpdir/pkg/" 2>/dev/null || true
  cp "$ROOT/dist/linux/quickaccent.desktop" "$tmpdir/pkg/" 2>/dev/null || true
  cp "$ROOT/dist/linux/60-quickaccent.rules" "$tmpdir/pkg/" 2>/dev/null || true
  cp "$ROOT/assets/icon-app.svg" "$tmpdir/pkg/quickaccent.svg" 2>/dev/null || true
  cp "$ROOT/assets/icon-app-256.png" "$tmpdir/pkg/quickaccent-256.png" 2>/dev/null || true
  cp "$ROOT/assets/icon-app-512.png" "$tmpdir/pkg/quickaccent-512.png" 2>/dev/null || true
}

if [[ "${INSTALL_FROM_SOURCE:-0}" == "1" ]]; then
  build_from_source
  PKG="$tmpdir/pkg"
else
  if fetch_release; then
    tar -xzf "$tmpdir/$ASSET" -C "$tmpdir"
    PKG="$(find "$tmpdir" -maxdepth 1 -type d -name 'quickaccent-linux-*' | head -1)"
    [[ -n "$PKG" && -x "$PKG/quickaccent" ]] || { echo "error: unexpected archive layout" >&2; exit 1; }
  else
    echo "warning: download failed; falling back to source build" >&2
    build_from_source
    PKG="$tmpdir/pkg"
  fi
fi

echo "==> Install binary + desktop files"
mkdir -p "$BIN_DIR" "$UNIT_DIR" "$APP_DIR" \
  "$ICON_BASE/scalable/apps" "$ICON_BASE/256x256/apps" "$ICON_BASE/512x512/apps"
install -m 755 "$PKG/quickaccent" "$BIN_DIR/quickaccent"

ICON_INSTALLED=""
if [[ -f "$PKG/quickaccent.svg" ]]; then
  install -m 644 "$PKG/quickaccent.svg" "$ICON_BASE/scalable/apps/quickaccent.svg"
  ICON_INSTALLED="$ICON_BASE/scalable/apps/quickaccent.svg"
elif [[ -n "${ROOT:-}" && -f "$ROOT/assets/icon-app.svg" ]]; then
  install -m 644 "$ROOT/assets/icon-app.svg" "$ICON_BASE/scalable/apps/quickaccent.svg"
  ICON_INSTALLED="$ICON_BASE/scalable/apps/quickaccent.svg"
fi
if [[ -f "$PKG/quickaccent-256.png" ]]; then
  install -m 644 "$PKG/quickaccent-256.png" "$ICON_BASE/256x256/apps/quickaccent.png"
elif [[ -n "${ROOT:-}" && -f "$ROOT/assets/icon-app-256.png" ]]; then
  install -m 644 "$ROOT/assets/icon-app-256.png" "$ICON_BASE/256x256/apps/quickaccent.png"
fi
if [[ -f "$PKG/quickaccent-512.png" ]]; then
  install -m 644 "$PKG/quickaccent-512.png" "$ICON_BASE/512x512/apps/quickaccent.png"
elif [[ -n "${ROOT:-}" && -f "$ROOT/assets/icon-app-512.png" ]]; then
  install -m 644 "$ROOT/assets/icon-app-512.png" "$ICON_BASE/512x512/apps/quickaccent.png"
fi
if [[ -z "$ICON_INSTALLED" && -f "$ICON_BASE/256x256/apps/quickaccent.png" ]]; then
  ICON_INSTALLED="$ICON_BASE/256x256/apps/quickaccent.png"
fi

if [[ -f "$PKG/quickaccent.service" ]]; then
  # Point ExecStart at installed binary
  sed "s|ExecStart=.*|ExecStart=$BIN_DIR/quickaccent|" "$PKG/quickaccent.service" \
    > "$UNIT_DIR/quickaccent.service"
elif [[ -n "${ROOT:-}" && -f "$ROOT/dist/linux/quickaccent.service" ]]; then
  sed "s|ExecStart=.*|ExecStart=$BIN_DIR/quickaccent|" "$ROOT/dist/linux/quickaccent.service" \
    > "$UNIT_DIR/quickaccent.service"
fi

DESKTOP_SRC=""
if [[ -f "$PKG/quickaccent.desktop" ]]; then
  DESKTOP_SRC="$PKG/quickaccent.desktop"
elif [[ -n "${ROOT:-}" && -f "$ROOT/dist/linux/quickaccent.desktop" ]]; then
  DESKTOP_SRC="$ROOT/dist/linux/quickaccent.desktop"
fi
if [[ -n "$DESKTOP_SRC" ]]; then
  sed "s|^Exec=.*|Exec=$BIN_DIR/quickaccent|" "$DESKTOP_SRC" > "$APP_DIR/quickaccent.desktop"
  if [[ -n "$ICON_INSTALLED" ]]; then
    sed -i "s|^Icon=.*|Icon=$ICON_INSTALLED|" "$APP_DIR/quickaccent.desktop"
  fi
  gtk-update-icon-cache -f -t "$ICON_BASE" 2>/dev/null || true
  update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

RULE_SRC=""
if [[ -f "$PKG/60-quickaccent.rules" ]]; then
  RULE_SRC="$PKG/60-quickaccent.rules"
elif [[ -n "${ROOT:-}" && -f "$ROOT/dist/linux/60-quickaccent.rules" ]]; then
  RULE_SRC="$ROOT/dist/linux/60-quickaccent.rules"
fi

if [[ -n "$RULE_SRC" ]] && command -v sudo >/dev/null 2>&1; then
  echo "==> udev + uinput module + input group (sudo)"
  sudo install -m 644 "$RULE_SRC" /etc/udev/rules.d/60-quickaccent.rules
  sudo udevadm control --reload-rules || true
  sudo udevadm trigger || true

  # uinput must be loaded at boot: QuickAccent types through a virtual
  # keyboard (no desktop portal needed).
  echo uinput | sudo tee /etc/modules-load.d/uinput.conf >/dev/null
  sudo modprobe uinput || true

  NEED_RELOGIN=0
  if ! id -nG | tr ' ' '\n' | grep -qx input; then
    sudo usermod -aG input "$USER"
    NEED_RELOGIN=1
  fi
else
  NEED_RELOGIN=0
  echo "warning: skipped udev/input group (no rule file or no sudo)" >&2
fi

if command -v systemctl >/dev/null 2>&1; then
  systemctl --user daemon-reload || true
  systemctl --user enable --now quickaccent.service || true
fi

echo
echo "Installed $BIN_DIR/quickaccent"
echo "  • group 'input' can read all keyboards — trusted users only"
echo "  • accents your layout lacks are added to the keymap automatically"
echo "    (xkb option quickaccent:accents in ~/.config/xkb) — no prompts"
if [[ "${NEED_RELOGIN:-0}" -eq 1 ]]; then
  echo "  • reboot so group 'input' applies to systemd --user (logout is not enough)"
else
  echo "  • start: systemctl --user restart quickaccent.service"
fi
