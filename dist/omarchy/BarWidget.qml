// QuickAccent bar widget for the Omarchy shell.
//
// Self-contained on purpose: it only shells out to `systemctl --user`, so it
// makes no assumptions about the shell's internal service API. QuickAccent
// itself is a standalone daemon (evdev grab + uinput injection) — this widget
// only shows and toggles it.

import QtQuick
import Quickshell
import Quickshell.Io

Item {
  id: root

  // Injected by the shell; unused here but part of the plugin contract.
  property var shell: null

  // "active" while the daemon runs, anything else (inactive/failed/unknown)
  // means accents are not armed.
  property string status: "unknown"
  readonly property bool running: status === "active"
  readonly property bool installed: status !== "missing"

  implicitWidth: label.implicitWidth
  implicitHeight: label.implicitHeight

  Text {
    id: label
    anchors.centerIn: parent
    // The app's own mark: a Q with an accent bar.
    text: "Q̄"
    font.pixelSize: 14
    color: root.running ? "#F0C400" : "#6E6E6E"
    opacity: root.installed ? 1.0 : 0.4
  }

  MouseArea {
    anchors.fill: parent
    hoverEnabled: true
    acceptedButtons: Qt.LeftButton
    onClicked: root.toggle()
  }

  function refresh() {
    if (!statusProc.running)
      statusProc.running = true
  }

  function toggle() {
    if (controlProc.running)
      return
    controlProc.command = ["systemctl", "--user",
                           root.running ? "stop" : "start",
                           "quickaccent.service"]
    controlProc.running = true
  }

  Process {
    id: statusProc
    command: ["systemctl", "--user", "is-active", "quickaccent.service"]
    stdout: StdioCollector {
      onStreamFinished: {
        var value = (text || "").trim()
        root.status = value.length > 0 ? value : "missing"
      }
    }
  }

  Process {
    id: controlProc
    onExited: settle.restart()
  }

  // systemd needs a moment before is-active reflects the change.
  Timer {
    id: settle
    interval: 400
    onTriggered: root.refresh()
  }

  Timer {
    interval: 5000
    running: true
    repeat: true
    onTriggered: root.refresh()
  }

  Component.onCompleted: refresh()
}
