pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var wellbeing: ({
        enabled: true,
        focus: false,
        limitBehavior: "advisory",
        todaySeconds: 0,
        weekSeconds: 0,
        monthSeconds: 0,
        dailyGoalSeconds: 0,
        goalReached: false,
        days: [],
        apps: []
    })
    property string wellbeingError: ""
    property string exportedPath: ""
    property bool clearAllArmed: false

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    readonly property real weekMax: {
        let max = 1;
        const days = (root.wellbeing.days || []).slice(0, 7);
        for (const day of days)
            max = Math.max(max, day.seconds || 0);
        return max;
    }

    function formatDuration(seconds) {
        const minutes = Math.floor((seconds || 0) / 60);
        if (minutes < 60)
            return qsTr("%1 min").arg(minutes);
        return qsTr("%1 h %2 min").arg(Math.floor(minutes / 60)).arg(minutes % 60);
    }

    function refresh() {
        if (!wellbeingStatus.running)
            wellbeingStatus.running = true;
    }

    function run(command) {
        if (wellbeingChange.running)
            return;
        root.wellbeingError = "";
        wellbeingChange.command = command;
        wellbeingChange.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Timer {
        id: clearAllConfirmTimer
        interval: 6000
        repeat: false
        onTriggered: root.clearAllArmed = false
    }

    Process {
        id: wellbeingStatus
        command: ["@vesperControl@", "wellbeing", "report"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.wellbeing = JSON.parse(text);
                    root.wellbeingError = "";
                    if (!goalMinutes.activeFocus)
                        goalMinutes.text = root.wellbeing.dailyGoalSeconds > 0
                            ? String(Math.round(root.wellbeing.dailyGoalSeconds / 60))
                            : "";
                } catch (e) {
                    root.wellbeingError = qsTr("Could not read Wellbeing report");
                }
            }
        }
    }

    Process {
        id: wellbeingChange
        stderr: StdioCollector { id: wellbeingChangeError }
        onExited: (code, status) => {
            if (code !== 0)
                root.wellbeingError = wellbeingChangeError.text.trim() || qsTr("Could not change Wellbeing state");
            root.refresh();
        }
    }

    Process {
        id: wellbeingExport
        command: ["@vesperControl@", "wellbeing", "export"]
        stdout: StdioCollector {
            onStreamFinished: {
                root.exportedPath = text.trim();
                root.wellbeingError = "";
            }
        }
        stderr: StdioCollector { id: wellbeingExportError }
        onExited: (code, status) => {
            if (code !== 0)
                root.wellbeingError = wellbeingExportError.text.trim() || qsTr("Could not export Wellbeing data");
        }
    }

    SectionHeader {
        text: qsTr("Wellbeing")
    }

    ToggleRow {
        text: qsTr("Screen time")
        subtext: qsTr("local foreground time · idle/locked time excluded · history stays on device")
        checked: root.wellbeing.enabled === true
        disabled: wellbeingChange.running
        onToggled: root.run(["@vesperControl@", "wellbeing", checked ? "on" : "off"])
    }

    ToggleRow {
        text: qsTr("Focus mode")
        subtext: qsTr("uses Caelestia Do Not Disturb; this is not a hard application blocker")
        checked: root.wellbeing.focus === true
        disabled: wellbeingChange.running
        onToggled: root.run(["@vesperControl@", "wellbeing", "focus", checked ? "on" : "off"])
    }

    InfoRow {
        icon: root.wellbeing.enabled ? "timer" : "pause_circle"
        label: qsTr("Today")
        subtext: root.wellbeing.goalReached
            ? qsTr("daily goal reached")
            : qsTr("limit behavior: %1").arg(root.wellbeing.limitBehavior || "advisory")
        value: root.formatDuration(root.wellbeing.todaySeconds)
    }

    InfoRow {
        icon: "date_range"
        label: qsTr("Last 7 days")
        value: root.formatDuration(root.wellbeing.weekSeconds)
    }

    InfoRow {
        icon: "calendar_month"
        label: qsTr("Last 30 days")
        value: root.formatDuration(root.wellbeing.monthSeconds)
    }

    SectionHeader {
        text: qsTr("Daily goal")
    }

    StyledTextField {
        id: goalMinutes
        Layout.fillWidth: true
        placeholderText: qsTr("minutes; 0 or empty disables")
        leadingIcon: "flag"
        supportingText: qsTr("personal screen-time goal; advisory only")
        inputMethodHints: Qt.ImhDigitsOnly
    }

    RowLayout {
        Layout.fillWidth: true
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true
            icon: "save"
            text: qsTr("Save goal")
            disabled: wellbeingChange.running
            onClicked: {
                const minutes = Math.max(0, Number(goalMinutes.text || 0));
                root.run(["@vesperControl@", "wellbeing", "goal", String(Math.round(minutes * 60))]);
            }
        }
    }

    SectionHeader {
        text: qsTr("7-day activity")
    }

    Repeater {
        model: (root.wellbeing.days || []).slice(0, 7).reverse()

        delegate: ColumnLayout {
            required property var modelData
            Layout.fillWidth: true
            spacing: Tokens.spacing.extraSmall / 2

            RowLayout {
                Layout.fillWidth: true
                StyledText {
                    text: modelData.date
                    font: Tokens.font.label.small
                    color: Colours.palette.m3onSurfaceVariant
                }
                Item { Layout.fillWidth: true }
                StyledText {
                    text: root.formatDuration(modelData.seconds)
                    font: Tokens.font.label.small
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 8
                radius: height / 2
                color: Colours.palette.m3surfaceContainerHighest

                Rectangle {
                    width: parent.width * Math.min(1, (modelData.seconds || 0) / root.weekMax)
                    height: parent.height
                    radius: parent.radius
                    color: Colours.palette.m3primary
                }
            }
        }
    }

    SectionHeader {
        text: qsTr("Top apps today")
    }

    Repeater {
        model: (root.wellbeing.apps || []).slice(0, 8)

        delegate: InfoRow {
            required property var modelData
            icon: modelData.overLimit ? "warning" : "schedule"
            label: modelData.app
            subtext: modelData.category
                ? modelData.category
                : (modelData.overLimit ? qsTr("advisory daily limit reached") : "")
            value: root.formatDuration(modelData.seconds)
        }
    }

    SectionHeader {
        text: qsTr("Local data")
    }

    RowButton {
        icon: "download"
        text: qsTr("Export Wellbeing JSON")
        subtext: root.exportedPath
            ? root.exportedPath
            : qsTr("writes a private 0600 snapshot under Vesper's local state directory")
        disabled: wellbeingExport.running
        onClicked: {
            root.exportedPath = "";
            wellbeingExport.running = true;
        }
    }

    RowButton {
        icon: "restart_alt"
        text: qsTr("Reset today")
        subtext: qsTr("clears only today's recorded foreground samples")
        disabled: wellbeingChange.running
        onClicked: root.run(["@vesperControl@", "wellbeing", "reset", "today"])
    }

    RowButton {
        icon: root.clearAllArmed ? "warning" : "delete_sweep"
        text: root.clearAllArmed ? qsTr("Confirm clear all history") : qsTr("Clear all history")
        subtext: root.clearAllArmed
            ? qsTr("click again within 6 seconds; app policies and goals are kept")
            : qsTr("removes recorded daily TSV history only")
        disabled: wellbeingChange.running
        onClicked: {
            if (root.clearAllArmed) {
                clearAllConfirmTimer.stop();
                root.clearAllArmed = false;
                root.run(["@vesperControl@", "wellbeing", "reset", "all"]);
            } else {
                root.clearAllArmed = true;
                clearAllConfirmTimer.restart();
            }
        }
    }

    InfoRow {
        icon: "robot_2"
        label: qsTr("Agent access")
        subtext: qsTr("structured local JSON via wellbeing report / wellbeing-summary")
        value: qsTr("readable")
    }

    StyledText {
        Layout.fillWidth: true
        visible: root.wellbeingError
        text: root.wellbeingError
        color: Colours.palette.m3error
        font: Tokens.font.body.small
        wrapMode: Text.WordWrap
    }
}
