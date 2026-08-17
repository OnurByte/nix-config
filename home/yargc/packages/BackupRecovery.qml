pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.modules.nexus.common

PageBase {
    id: root
    property var backup: ({ retention: {} })
    property string errorText: ""
    title: qsTr("Backup & Recovery")

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()
    Timer { interval: 30000; repeat: true; running: root.visible; onTriggered: root.refresh() }

    Process {
        id: status
        command: ["@vesperControl@", "recovery", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.backup = JSON.parse(text);
                    root.errorText = "";
                } catch (e) {
                    root.errorText = qsTr("Could not read backup state");
                }
            }
        }
        stderr: StdioCollector { id: statusError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = statusError.text.trim();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Backup status") }
        InfoRow {
            icon: "backup"
            label: qsTr("Backend")
            subtext: qsTr("NixOS-managed local backup job")
            value: root.backup.backend || qsTr("unknown")
        }
        InfoRow {
            icon: "schedule"
            label: qsTr("Scheduler")
            subtext: root.backup.nextRun || qsTr("next run unavailable")
            value: root.backup.timerActive ? qsTr("active") : qsTr("inactive")
        }
        InfoRow {
            icon: root.backup.failed ? "error" : "verified"
            label: qsTr("Last job")
            subtext: root.backup.lastRun || qsTr("no completed run reported")
            value: root.backup.failed ? qsTr("failed") : (root.backup.lastResult || qsTr("unknown"))
        }
        InfoRow {
            icon: "folder"
            label: qsTr("Repository")
            subtext: root.backup.repository || qsTr("repository path unavailable")
            value: root.backup.repositoryExists ? qsTr("present") : qsTr("not detected")
        }
        InfoRow {
            icon: "history"
            label: qsTr("Retention")
            subtext: qsTr("daily / weekly / monthly archives")
            value: qsTr("%1 / %2 / %3")
                .arg(root.backup.retention?.daily ?? 0)
                .arg(root.backup.retention?.weekly ?? 0)
                .arg(root.backup.retention?.monthly ?? 0)
        }

        SectionHeader { text: qsTr("Recovery policy") }
        InfoRow {
            icon: "lock"
            label: qsTr("Restore from Settings")
            subtext: qsTr("Destructive restore is intentionally not exposed here; recovery remains an explicit administrative operation")
            value: qsTr("disabled")
        }
        InfoRow {
            icon: "deployed_code"
            label: qsTr("Ownership")
            subtext: qsTr("Job, repository path and retention are declared by NixOS")
            value: qsTr("NixOS")
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.errorText
            text: root.errorText
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
