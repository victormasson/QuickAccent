// QuickAccent helper: only the compositor knows where windows are on
// Wayland, so this tiny extension exposes the focused window's frame
// rectangle over D-Bus (destination org.gnome.Shell). QuickAccent uses it to
// place the accent picker on the monitor being typed on.

import Gio from 'gi://Gio';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const NODE = `<node>
<interface name="io.github.victormasson.QuickAccent.FocusedWindow">
  <method name="Get">
    <arg type="s" direction="out" name="rect"/>
  </method>
</interface>
</node>`;

export default class QuickAccentFocusExtension extends Extension {
    enable() {
        this._dbus = Gio.DBusExportedObject.wrapJSObject(NODE, this);
        this._dbus.export(
            Gio.DBus.session,
            '/io/github/victormasson/QuickAccent/FocusedWindow'
        );
    }

    disable() {
        this._dbus?.unexport();
        this._dbus = null;
    }

    // Returns "x y width height" of the focused window's frame in global
    // logical coordinates, or "" when nothing is focused.
    Get() {
        const win = global.display.focus_window;
        if (!win)
            return '';
        const r = win.get_frame_rect();
        return `${r.x} ${r.y} ${r.width} ${r.height}`;
    }
}
