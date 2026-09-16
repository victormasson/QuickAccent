// QuickAccent: focused-window geometry over D-Bus (so the picker opens on
// the monitor being typed on) and a top-bar menu like the macOS status item.

import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GObject from 'gi://GObject';
import St from 'gi://St';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import * as PanelMenu from 'resource:///org/gnome/shell/ui/panelMenu.js';
import * as PopupMenu from 'resource:///org/gnome/shell/ui/popupMenu.js';

const NODE = `<node>
<interface name="io.github.victormasson.QuickAccent.FocusedWindow">
  <method name="Get">
    <arg type="s" direction="out" name="rect"/>
  </method>
</interface>
</node>`;

const APP_BUS = 'io.github.victormasson.QuickAccent';
const APP_PATH = '/io/github/victormasson/QuickAccent';
const APP_IFACE = 'io.github.victormasson.QuickAccent';

const Indicator = GObject.registerClass(
class QuickAccentIndicator extends PanelMenu.Button {
    _init() {
        super._init(0.5, 'QuickAccent');

        this.add_child(new St.Label({
            text: 'Á',
            y_align: Clutter.ActorAlign.CENTER,
            style_class: 'quickaccent-panel-label',
        }));

        const settings = new PopupMenu.PopupMenuItem('Settings\u2026');
        settings.connect('activate', () => this._call('OpenSettings'));
        this.menu.addMenuItem(settings);

        this.menu.addMenuItem(new PopupMenu.PopupSeparatorMenuItem());

        const quit = new PopupMenu.PopupMenuItem('Quit QuickAccent');
        quit.connect('activate', () => this._call('Quit'));
        this.menu.addMenuItem(quit);
    }

    _call(method) {
        Gio.DBus.session.call(
            APP_BUS,
            APP_PATH,
            APP_IFACE,
            method,
            null,
            null,
            Gio.DBusCallFlags.NONE,
            -1,
            null,
            null
        );
    }
});

export default class QuickAccentFocusExtension extends Extension {
    enable() {
        this._dbus = Gio.DBusExportedObject.wrapJSObject(NODE, this);
        this._dbus.export(
            Gio.DBus.session,
            '/io/github/victormasson/QuickAccent/FocusedWindow'
        );
        this._indicator = new Indicator();
        Main.panel.addToStatusArea(this.uuid, this._indicator);
    }

    disable() {
        this._indicator?.destroy();
        this._indicator = null;
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
