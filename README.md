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

### Linux (Wayland or X11; GNOME and Hyprland/Omarchy) — prebuilt, no Rust

Every release ships `quickaccent-linux-x86_64.tar.gz`, a `SHA256SUMS` file and
a Sigstore build-provenance attestation, so the binary you install can be
checked against what CI built from the tagged commit:

```bash
gh attestation verify quickaccent-linux-x86_64.tar.gz --repo victormasson/QuickAccent
```

**Arch / Omarchy** — AUR package [`quickaccent-bin`](dist/arch/) (tarball
pinned by sha256 in the PKGBUILD):

```bash
yay -S quickaccent-bin           # or: omarchy pkg add quickaccent-bin
sudo usermod -aG input "$USER"   # then reboot once
systemctl --user enable --now quickaccent
```

**Fedora, Debian, others** — clone the release tag and run the installer. It
downloads the tarball for that same tag, verifies it against `SHA256SUMS`
(and the attestation when `gh` is logged in), and refuses to continue on a
mismatch:

```bash
git clone --branch v1.2.0 --depth 1 https://github.com/victormasson/QuickAccent
QuickAccent/dist/linux/install.sh
sudo reboot
```

The installer sets up:

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
  `~/.config/xkb/symbols/quickaccent`, and `systemctl --user disable --now quickaccent`
  (Arch: `sudo pacman -Rns quickaccent-bin`).

Details: [dist/linux/README.md](dist/linux/README.md).

#### Omarchy / Hyprland

Install the AUR package as above. QuickAccent enables its keymap option with
`hyprctl keyword input:kb_options` (applied instantly, `hyprland.conf` is not
touched) and follows the focused window with `hyprctl activewindow`, so the
picker opens on the monitor you are typing on. There is an optional bar widget
for the Omarchy shell in [dist/omarchy/](dist/omarchy/).

### macOS (universal, no Rust)

```bash
git clone --branch v1.2.0 --depth 1 https://github.com/victormasson/QuickAccent
QuickAccent/dist/macos/install.sh    # verifies the asset against SHA256SUMS
open ~/Applications/QuickAccent.app
```

Grant **Accessibility** (System Settings → Privacy & Security).  
The menu-bar icon has *Settings…* (⌘,) to pick languages and symbol sets; the
picker draws on Liquid Glass (macOS 26+; a blur on older releases).  
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
# input_time_ms = 200
# activation_key = "Both"   # Space | LeftRightArrow | Both
# theme = "system"   # system | light | dark | dracula
                     # catppuccin-latte | catppuccin-frappe | catppuccin-macchiato | catppuccin-mocha
                     # rose-pine | rose-pine-moon | rose-pine-dawn
# overlay_opacity = 0.88
# overlay_radius = 16
# chip_radius = 8
```

On macOS the *Settings…* window in the menu-bar menu (and on GNOME the top-bar
*QuickAccent* menu) edits every key above for you (only those lines are
rewritten; comments and other keys are kept).

**Languages:** Catalan, CrimeanTatar, Croatian, Czech, Danish, Dutch, Esperanto, Estonian, Finnish, French, German, Greek, Hungarian, IPA, Iceland, Irish, Italian, Kurdish, Lithuanian, Maltese, Maori, Norwegian, Pinyin, Polish, Portuguese, ProtoIndoEuropean, Romanian, Romanization, ScottishGaelic, Serbian, Slovak, Slovenian, Spanish, Swedish, Turkish, Vietnamese, Welsh

**Additional sets:** `Special` (PowerToys symbols, punctuation, fractions and
superscripts) and `Currency`.
Optional extensions: `Typography`, `Arrows`, `Math`, and `CurrencyExtended`.
Add these names to `languages` to enable them. Hold a punctuation or number key
and trigger the picker just as you would for a letter. See
[character bindings](docs/CHARACTERS.md) for configuration and symbol keys.

`Hebrew` and `Yiddish` are also available as optional language sets, using
phonetic Latin keys. See [their bindings](docs/CHARACTERS.md#phonetic-hebrew-and-yiddish)
for letters, final forms, vowel marks, and Yiddish combinations.

`Cherokee`, `Osage`, `CanadianAboriginalSyllabics`,
`CanadianAboriginalSyllabicsExtended`, and `CanadianAboriginalSyllabicsExtendedA`
provide complete Unicode 17.0 repertoires through Latin-key lookup. See
[script bindings and casing](docs/CHARACTERS.md#cherokee-osage-and-canadian-syllabics).

## Usage

```bash
quickaccent
RUST_LOG=debug quickaccent
journalctl --user -u quickaccent -f
QUICKACCENT_DEMO=overlay quickaccent   # or =settings: open that window at
                                        # startup, no keyboard grab (UI work)
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

Assets: `quickaccent-linux-x86_64.tar.gz`, `QuickAccent-macos-universal.tar.gz`,
`SHA256SUMS`, plus a [build-provenance attestation](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations)
per archive (`gh attestation verify <asset> --repo victormasson/QuickAccent`).

Cutting a release: update `CHANGELOG.md`, bump the version in `Cargo.toml`,
`dist/macos/QuickAccent.app/Contents/Info.plist`, `dist/omarchy/manifest.json`
and `dist/arch/PKGBUILD`, then `git tag vX.Y.Z && git push origin vX.Y.Z`.
Once the release is up, pin its sha256 in the PKGBUILD and push to the AUR
([dist/arch/README.md](dist/arch/README.md)).

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

MIT. Third-party notices: [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
