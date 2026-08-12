#!/usr/bin/env bash
# Install QuickAccent for the current user.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BIN_DIR="${PREFIX:-$HOME/.local}/bin"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
AUTO_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/autostart"

command -v cargo >/dev/null || { echo "error: install Rust (https://rustup.rs/)" >&2; exit 1; }

echo "==> Build"
(cd "$ROOT" && cargo build --release)

echo "==> Install binary + desktop files"
mkdir -p "$BIN_DIR" "$UNIT_DIR" "$APP_DIR" "$AUTO_DIR"
install -m 755 "$ROOT/target/release/quickaccent" "$BIN_DIR/quickaccent"
install -m 644 "$ROOT/dist/linux/quickaccent.service" "$UNIT_DIR/quickaccent.service"
for d in "$APP_DIR" "$AUTO_DIR"; do
  install -m 644 "$ROOT/dist/linux/quickaccent.desktop" "$d/quickaccent.desktop"
  sed -i "s|^Exec=quickaccent|Exec=$BIN_DIR/quickaccent|" "$d/quickaccent.desktop" 2>/dev/null || true
done

echo "==> udev + input group (sudo)"
sudo install -m 644 "$ROOT/dist/linux/60-quickaccent.rules" /etc/udev/rules.d/60-quickaccent.rules
sudo udevadm control --reload-rules || true
sudo udevadm trigger || true

NEED_RELOGIN=0
if ! id -nG | tr ' ' '\n' | grep -qx input; then
  sudo usermod -aG input "$USER"
  NEED_RELOGIN=1
fi

if command -v systemctl >/dev/null; then
  systemctl --user daemon-reload || true
  systemctl --user enable --now quickaccent.service || true
fi

echo
echo "Installed $BIN_DIR/quickaccent"
echo "  • group 'input' can read all keyboards (keylogger-equivalent) — trusted users only"
echo "  • GNOME Wayland: approve the input / Remote Desktop portal when prompted"
if [[ "$NEED_RELOGIN" -eq 1 ]]; then
  echo "  • log out and back in so group 'input' applies"
else
  echo "  • start: systemctl --user restart quickaccent.service"
fi
