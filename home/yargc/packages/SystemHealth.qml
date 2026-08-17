pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.modules.nexus.common

PageBase {
    id: root

    property var report: ({ healthy: false, checks: [] })
    property string loadError: ""
    property string exportedPath: ""

    title: qsTr("System Health")

    readonly property int warningCount: (report.checks || []).filter(item => item.level === "warn").length
    readonly property int okCount: (report.checks || []).filter(item => item.level === "ok").length

    function refresh() {
        if (!doctor.running)
            doctor.running = true;
    }

    function iconFor(level) {
        if (level === "ok")
            return "check_circle";
        if (level === "warn")
            return "warning";
        return "info";
    }

    Component.onCompleted: refresh()

    Process {
        id: doctor
        command: ["@vesperDoctor@", "--json"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.report = JSON.parse(text);
                    root.loadError = "";
                } catch (e) {
                    root.loadError = qsTr("vesper-doctor returned invalid JSON");
                }
            }
        }
        stderr: StdioCollector { id: doctorError }
        onExited: (code, status) => {
            if (code !== 0)
                root.loadError = doctorError.text.trim() || qsTr("vesper-doctor failed");
        }
    }

    Process {
        id: exportDoctor
        command: ["@vesperDoctor@", "--export"]
        stdout: StdioCollector {
            onStreamFinished: {
                root.exportedPath = text.trim();
                root.loadError = "";
            }
        }
        stderr: StdioCollector { id: exportError }
        onExited: (code, status) => {
            if (code !== 0)
                root.loadError = exportError.text.trim() || qsTr("Could not export diagnostic report");
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader {
            first: true
            text: qsTr("Overview")
        }

        InfoRow {
            icon: root.report.healthy ? "check_circle" : "warning"
            label: qsTr("Workstation health")
            subtext: qsTr("structured directly from vesper-doctor --json")
            value: root.report.healthy ? qsTr("healthy") : qsTr("attention needed")
        }

        InfoRow {
            icon: "fact_check"
            label: qsTr("Checks")
            subtext: qsTr("%1 passing · %2 warnings").arg(root.okCount).arg(root.warningCount)
            value: String((root.report.checks || []).length)
        }

        RowButton {
            icon: "refresh"
            text: qsTr("Refresh")
            subtext: qsTr("run vesper-doctor again; Settings does not duplicate its health logic")
            disabled: doctor.running
            onClicked: root.refresh()
        }

        RowButton {
            icon: "download"
            text: qsTr("Export diagnostic JSON")
            subtext: root.exportedPath
                ? root.exportedPath
                : qsTr("writes the same structured report to a private 0600 file under local Vesper state")
            disabled: exportDoctor.running
            onClicked: {
                root.exportedPath = "";
                exportDoctor.running = true;
            }
        }

        SectionHeader {
            text: qsTr("Checks")
        }

        Repeater {
            model: root.report.checks || []

            delegate: InfoRow {
                required property var modelData
                icon: root.iconFor(modelData.level)
                label: modelData.key || qsTr("check")
                subtext: modelData.message || ""
                value: modelData.level || qsTr("info")
            }
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.loadError
            text: root.loadError
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
