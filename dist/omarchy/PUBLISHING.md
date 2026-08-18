# Publishing this plugin to omarchyplugins.com

The marketplace requires `manifest.json` in the **repository root**, so this
directory has to be pushed as its own repository.

```bash
# from a clone of QuickAccent
cp -r dist/omarchy /tmp/omarchy-plugin-quickaccent
cd /tmp/omarchy-plugin-quickaccent
rm PUBLISHING.md
git init -b main && git add -A
git commit -m "QuickAccent plugin for Omarchy"
gh repo create victormasson/omarchy-plugin-quickaccent --public --source=. --push
```

Verify on an Omarchy machine before submitting:

```bash
cp -r . ~/.config/omarchy/plugins/io.github.victormasson.quickaccent
omarchy plugin validate ~/.config/omarchy/plugins/io.github.victormasson.quickaccent
qmllint -I "$OMARCHY_PATH/shell" BarWidget.qml
omarchy plugin list --json | jq '.[] | select(.id == "io.github.victormasson.quickaccent")'
```

Then open the submission issue:
<https://github.com/HANCORE-linux/omarchy-plugin-marketplace/issues/new?template=submit-plugin.yml>

Form values:

| Field | Value |
|-------|-------|
| Repository URL | `https://github.com/victormasson/omarchy-plugin-quickaccent` |
| Category | System |
| Tags | Hyprland, System, Quickshell |
| Maintainer notes | Front-end for the QuickAccent daemon (separate install, one curl command). Needs membership of group `input` and access to `/dev/uinput` to grab keys and inject accents; the widget itself only calls `systemctl --user`. Enables the xkb option `quickaccent:accents` via `hyprctl keyword input:kb_options` at runtime and does not modify `hyprland.conf`. MIT. |

Checklist: repository public with instructions, license and dependencies
documented, ownership confirmed, plugin respects user configuration.
