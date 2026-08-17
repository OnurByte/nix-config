pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var wellbeing: ({ schemaVersion: 1, enabled: true, agentReadable: true, totalSeconds: 0, apps: [] })
    property bool wellbeingEnabled: true
    property bool adaptiveIcons: false
    property string wellbeingError: ""

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
                    root.wellbeingEnabled = root.wellbeing.enabled !== false;
                    root.wellbeingError = "";
                } catch (e) {
                    root.wellbeing = ({ schemaVersion: 1, enabled: true, agentReadable: true, totalSeconds: 0, apps: [] });
                    root.wellbeingError = qsTr("Could not read Wellbeing status");
                }
            }
        }
    }

    Process {
        id: wellbeingChange
        stderr: StdioCollector {
            id: wellbeingChangeError
        }
        onExited: (code, status) => {
            if (code !== 0)
                root.wellbeingError = wellbeingChangeError.text.trim() || qsTr("Could not change Wellbeing state");
            root.refresh();
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

    ToggleRow {
        text: qsTr("Wellbeing")
        subtext: qsTr("On by default · local foreground app time · agents can read the summary")
        checked: root.wellbeingEnabled
        disabled: wellbeingChange.running
        onToggled: {
            root.wellbeingEnabled = checked;
            root.wellbeingError = "";
            wellbeingChange.command = ["@vesperControl@", "wellbeing", checked ? "on" : "off"];
            wellbeingChange.running = true;
        }
    }

    InfoRow {
        icon: root.wellbeingEnabled ? "timer" : "pause_circle"
        label: qsTr("Foreground app time today")
        subtext: root.wellbeingEnabled
            ? qsTr("local only · paused while idle or locked")
            : qsTr("collection paused · existing history kept locally")
        value: root.formatDuration(root.wellbeing.totalSeconds)
    }

    InfoRow {
        icon: "robot_2"
        label: qsTr("Agent access")
        subtext: qsTr("read-only JSON via vesper-control wellbeing-summary")
        value: root.wellbeing.agentReadable === false ? qsTr("off") : qsTr("available")
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

    StyledText {
        Layout.fillWidth: true
        visible: root.wellbeingError
        text: root.wellbeingError
        color: Colours.palette.m3error
        font: Tokens.font.body.small
        wrapMode: Text.WordWrap
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
