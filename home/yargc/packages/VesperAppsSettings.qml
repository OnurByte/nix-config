pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var wellbeing: ({ totalSeconds: 0, apps: [] })
    property bool adaptiveIcons: false

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    function formatDuration(seconds) {
        const minutes = Math.floor((seconds || 0) / 60);
        if (minutes < 60)
            return qsTr("%1 min").arg(minutes);
        return qsTr("%1 h %2 min").arg(Math.floor(minutes / 60)).arg(minutes % 60);
    }

    function refresh() {
        if (!wellbeingStatus.running)
            wellbeingStatus.running = true;
        if (!iconStatus.running)
            iconStatus.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Process {
        id: wellbeingStatus
        command: ["@vesperControl@", "wellbeing-summary"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.wellbeing = JSON.parse(text);
                } catch (e) {
                    root.wellbeing = ({ totalSeconds: 0, apps: [] });
                }
            }
        }
    }

    Process {
        id: iconStatus
        command: ["@vesperControl@", "icon", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.adaptiveIcons = text.trim() === "on"
        }
    }

    Process {
        id: iconChange
        onExited: (code, status) => root.refresh()
    }

    SectionHeader {
        text: qsTr("Wellbeing")
    }

    InfoRow {
        icon: "timer"
        label: qsTr("Foreground app time today")
        subtext: qsTr("local only · sampled from Hyprland")
        value: root.formatDuration(root.wellbeing.totalSeconds)
    }

    Repeater {
        model: (root.wellbeing.apps || []).slice(0, 5)

        delegate: InfoRow {
            required property var modelData
            icon: "schedule"
            label: modelData.app
            value: root.formatDuration(modelData.seconds)
        }
    }

    SectionHeader {
        text: qsTr("Experimental")
    }

    ToggleRow {
        text: qsTr("AI adaptive icons")
        subtext: qsTr("enable the reviewed icon-job queue; generated assets are never applied automatically")
        checked: root.adaptiveIcons
        onToggled: {
            iconChange.command = ["@vesperControl@", "icon", checked ? "on" : "off"];
            iconChange.running = true;
        }
    }
}
