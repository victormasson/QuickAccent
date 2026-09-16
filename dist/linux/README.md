# QuickAccent on Linux

## Install (prebuilt, no Rust)

Release assets are verifiable: each release carries `SHA256SUMS` and a
Sigstore build-provenance attestation
(`gh attestation verify quickaccent-linux-x86_64.tar.gz --repo victormasson/QuickAccent`).

**Arch / Omarchy:** AUR package `quickaccent-bin`, see [../arch/](../arch/README.md).

**Fedora / Debian / others:** clone the release tag and run the installer from
it. The installer downloads the asset for the same tag, checks it against
`SHA256SUMS` (and the attestation when `gh` is logged in), and stops on any
mismatch. Nothing downloaded is ever executed before verification.

```bash
# Fedora / GNOME Wayland (Debian / Ubuntu: apt install wl-clipboard)
sudo dnf install -y wl-clipboard

git clone --branch v1.3.0 --depth 1 https://github.com/victormasson/QuickAccent
QuickAccent/dist/linux/install.sh
sudo reboot
```

| Env | Default | Meaning |
|-----|---------|---------|
| `GITHUB_REPO` | `victormasson/QuickAccent` | Repo for assets |
| `QUICKACCENT_VERSION` | version of the checkout | Release tag; `latest` = newest stable, `continuous` = rolling build |
| `QUICKACCENT_SKIP_VERIFY` | `0` | Set `1` only for releases before v1.2.0, which have no `SHA256SUMS` |
| `PREFIX` | `~/.local` | Install prefix (`bin/`) |
| `INSTALL_FROM_SOURCE` | `0` | Set `1` to `cargo build` instead |

## Checklist (all desktops)

1. Group `input` + udev rule (`/dev/input/event*` and `/dev/uinput`) — the
   installer does this with sudo.
2. **Reboot** after being added to `input`. A GNOME logout does not restart
   `systemd --user`, so grab stays `Permission denied`.

Injection is always direct — no permission prompt, no clipboard. Accents your
layout lacks (`é` on US, `É` on AZERTY…) are **added to the keymap** at
startup: QuickAccent generates the xkb option `quickaccent:accents` in
`~/.config/xkb/` (mapping them onto spare keycodes: F13–F23 plus a few
multimedia codes, F24 as the level-3 shift; 80 characters fit) and enables it in
GNOME's `xkb-options`; the compositor reloads the keymap live and the
characters are typed as ordinary keystrokes through uinput. Remove with
`gsettings reset org.gnome.desktop.input-sources xkb-options` (restores your
previous options minus ours) and delete `~/.config/xkb/symbols/quickaccent`.

Only one instance runs at a time: a second launch (autostart entry, app-grid
click) exits with "already running", and the systemd service always wins over
a manually started copy.

```bash
journalctl --user -u quickaccent -f
systemctl --user restart quickaccent.service
```

## What you need

| Need | Why |
|------|-----|
| Group `input` + udev rule | Evdev grab + uinput virtual keyboard |
| `uinput` module loaded | The installer adds `/etc/modules-load.d/uinput.conf` |
| Reboot after `usermod` | User systemd session picks up `input` |
| `wl-clipboard` (optional) | Emergency fallback only |
| x86_64 | Prebuilt asset (else build from source) |

**Security:** `input` can read all keystrokes. Trusted accounts only.

## Settings without a panel icon (Hyprland and other desktops)

There is no top-bar menu outside GNOME. Instead, launching QuickAccent while
the daemon runs opens its Settings window over D-Bus (click the app in your
launcher or dock), the desktop entry has *Settings* / *Quit QuickAccent*
actions for launchers that show them, `quickaccent --settings` and
`quickaccent --quit` do the same from a terminal, and a right-click on the
picker opens Settings too. On Hyprland the Settings window (class
`quickaccent-settings`) is floated and centred by a runtime window rule that
QuickAccent applies at startup and after each config reload; the persistent
form for `~/.config/hypr/windows.lua` is
`hl.window_rule({ match = { class = "quickaccent-settings" }, float = true, center = true })`.

## Multi-monitor overlay (GNOME)

Wayland hides window positions from apps, so QuickAccent self-installs a
GNOME Shell extension (`quickaccent-focus@victormasson.github.io`, sources
in `dist/linux/gnome-extension/`) that reports the focused window's
rectangle over D-Bus and adds a top-bar *QuickAccent* menu (*Settings…* /
*Quit*). The picker then opens centered on the window you are typing in —
i.e. on the right monitor. **Log out/in once** after the first run so GNOME
loads the extension; until then (and on other desktops) the overlay is
centered on the primary monitor and there is no panel button. Remove it with
`gnome-extensions disable quickaccent-focus@victormasson.github.io` and by
deleting its directory under `~/.local/share/gnome-shell/extensions/`.
Known limitation: with fractional display scaling the position can be offset.

## How typing works (macOS-style)

- A letter with accent variants is held back until you release the key — it
  appears on key **release** (~a keystroke later when typing normally), and
  holding it does **not** auto-repeat, exactly like macOS press-and-hold.
- Hold the letter, press Space → the picker opens and the plain letter is
  never typed. Cycle with Space/arrows, release the letter to insert the
  accent — no backspace, no cursor jump.
- Escape closes the picker and types the plain letter.
- Shift makes the picker uppercase and the choice sticks — releasing Shift
  before the letter still commits the capital. Pressing Shift while the
  picker is open toggles the case.

## Injection backend (all sessions: Wayland, X11)

Most direct mechanism first — the clipboard is a last resort only:

| Case | Mechanism |
|------|-----------|
| Accent in the keymap — natively, or added by the auto-installed `quickaccent:accents` xkb option (GNOME) | uinput key combo — instant, no authorization, all apps incl. terminals |
| Keymap extension unavailable (non-GNOME desktops, slot overflow) | Portal keysym injection — one-time authorization, persisted via restore token. Note: mutter only types keysyms already in the keymap |
| Portal denied/unavailable | `wl-copy` + virtual Ctrl+V, loudly logged. Terminals treat Ctrl+V literally |

## From source

```bash
INSTALL_FROM_SOURCE=1 ./dist/linux/install.sh
# or
cargo build --release
```

Fedora build deps:

```bash
sudo dnf install -y gcc pkgconf-pkg-config libxkbcommon-devel libxkbcommon-x11-devel \
  libevdev-devel wayland-devel fontconfig-devel alsa-lib-devel \
  libX11-devel libXi-devel libXtst-devel libXcursor-devel libXrandr-devel \
  libXinerama-devel mesa-libEGL-devel mesa-libGL-devel vulkan-loader-devel
```
