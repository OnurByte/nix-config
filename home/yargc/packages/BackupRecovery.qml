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
    property var backup: ({ backup: {}, repositoryCheck: {}, snapper: {}, btrfsScrub: {}, retention: {} })
    property string errorText: ""
    property string actionMessage: ""
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

    Process {
        id: safeCheck
        command: ["@vesperControl@", "recovery", "check"]
        stderr: StdioCollector { id: safeCheckError }
        onExited: (code, status) => {
            if (code === 0) {
                root.actionMessage = qsTr("Restic repository verification started");
                root.errorText = "";
            } else {
                root.actionMessage = "";
                root.errorText = safeCheckError.text.trim() || qsTr("Could not start repository verification");
            }
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Restic") }
        InfoRow {
            icon: "backup"
            label: qsTr("Backend")
            subtext: qsTr("NixOS-managed Restic job; repository credentials stay outside Settings")
            value: root.backup.backend || qsTr("unknown")
        }
        InfoRow {
            icon: "schedule"
            label: qsTr("Backup scheduler")
            subtext: root.backup.backup?.nextRun || qsTr("next run unavailable")
            value: root.backup.backup?.timerActive ? qsTr("active") : qsTr("inactive")
        }
        InfoRow {
            icon: root.backup.backup?.failed ? "error" : "verified"
            label: qsTr("Last backup")
            subtext: root.backup.backup?.lastRun || qsTr("no completed run reported")
            value: root.backup.backup?.failed ? qsTr("failed") : (root.backup.backup?.lastResult || qsTr("unknown"))
        }
        InfoRow {
            icon: root.backup.repositoryCheck?.failed ? "error" : "fact_check"
            label: qsTr("Repository check")
            subtext: root.backup.repositoryCheck?.lastRun || qsTr("no completed check reported")
            value: root.backup.repositoryCheck?.jobActive
                ? qsTr("running")
                : (root.backup.repositoryCheck?.failed ? qsTr("failed") : (root.backup.repositoryCheck?.lastResult || qsTr("unknown")))
        }
        InfoRow {
            icon: "event_repeat"
            label: qsTr("Next repository check")
            subtext: root.backup.repositoryCheck?.nextRun || qsTr("next check unavailable")
            value: root.backup.repositoryCheck?.timerActive ? qsTr("scheduled") : qsTr("inactive")
        }
        RowButton {
            icon: "fact_check"
            text: qsTr("Verify repository now")
            subtext: qsTr("starts only the read-only Restic check service; it does not create, prune or restore backups")
            visible: root.backup.safeCheckActionAvailableInSettings === true
            disabled: safeCheck.running || root.backup.repositoryCheck?.jobActive === true
            onClicked: {
                root.actionMessage = "";
                root.errorText = "";
                safeCheck.running = true;
            }
        }
        InfoRow {
            icon: "history"
            label: qsTr("Retention")
            subtext: qsTr("daily / weekly / monthly snapshots kept by Restic")
            value: qsTr("%1 / %2 / %3")
                .arg(root.backup.retention?.daily ?? 0)
                .arg(root.backup.retention?.weekly ?? 0)
                .arg(root.backup.retention?.monthly ?? 0)
        }

        SectionHeader { text: qsTr("Local snapshots") }
        InfoRow {
            icon: "restore"
            label: qsTr("Snapper · root")
            subtext: root.backup.snapper?.root || qsTr("snapshot inventory unavailable")
            value: root.backup.snapper?.root ? qsTr("available") : qsTr("unknown")
        }
        InfoRow {
            icon: "home_storage"
            label: qsTr("Snapper · home")
            subtext: root.backup.snapper?.home || qsTr("snapshot inventory unavailable")
            value: root.backup.snapper?.home ? qsTr("available") : qsTr("unknown")
        }
        InfoRow {
            icon: "storage"
            label: qsTr("Btrfs scrub")
            subtext: root.backup.btrfsScrub?.next || qsTr("scrub timer unavailable")
            value: root.backup.btrfsScrub?.result ? qsTr("reported") : qsTr("unknown")
        }

        SectionHeader { text: qsTr("Recovery policy") }
        InfoRow {
            icon: root.backup.restoreReady ? "verified_user" : "warning"
            label: qsTr("Restore readiness")
            subtext: qsTr("Ready only after both the latest backup and repository verification report success")
            value: root.backup.restoreReady ? qsTr("ready") : qsTr("not verified")
        }
        InfoRow {
            icon: "lock"
            label: qsTr("Restore from Settings")
            subtext: qsTr("Destructive restore is intentionally not exposed here; recovery remains an explicit administrative operation")
            value: qsTr("disabled")
        }
        InfoRow {
            icon: "deployed_code"
            label: qsTr("Ownership")
            subtext: qsTr("Restic jobs, Snapper timelines and Btrfs scrub schedules are declared by NixOS")
            value: qsTr("NixOS")
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.actionMessage
            text: root.actionMessage
            color: Colours.palette.m3primary
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: root.actionMessage ? 0 : Tokens.spacing.medium
            visible: root.errorText
            text: root.errorText
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
