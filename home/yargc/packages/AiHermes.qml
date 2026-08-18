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
    property var jobs: []
    property var briefings: []
    property var sourceRegistry: ({ count: 0, sources: [] })
    property string message: ""
    title: qsTr("Hermes Jobs")
    isSubPage: true

    function refresh() {
        if (!jobsProcess.running) jobsProcess.running = true;
        if (!briefingsProcess.running) briefingsProcess.running = true;
        if (!sourcesProcess.running) sourcesProcess.running = true;
    }

    function run(command) {
        if (action.running) return;
        root.message = "";
        action.command = command;
        action.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 15000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Process {
        id: jobsProcess
        command: ["vesper-hermes-jobs-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const registry = JSON.parse(text);
                    root.jobs = Object.keys(registry).sort().map(name => {
                        const job = registry[name] || ({});
                        job._name = name;
                        return job;
                    });
                    root.message = "";
                } catch (e) { root.message = qsTr("Could not read Hermes job registry"); }
            }
        }
    }

    Process {
        id: briefingsProcess
        command: ["vesper-hermes", "list", "--json"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const parsed = JSON.parse(text);
                    root.briefings = Array.isArray(parsed) ? parsed : [];
                } catch (e) {
                    root.message = qsTr("Could not read Hermes briefing history");
                }
            }
        }
    }

    Process {
        id: sourcesProcess
        command: ["vesper-hermes-automations", "links"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.sourceRegistry = JSON.parse(text);
                } catch (e) {
                    root.message = qsTr("Could not read Hermes source registry");
                }
            }
        }
    }

    Process {
        id: action
        stderr: StdioCollector { id: actionError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Hermes action completed") : actionError.text.trim();
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Scheduler") }
        InfoRow {
            icon: "schedule"
            label: qsTr("Declarative jobs")
            subtext: qsTr("schedules and orchestration stay Nix-owned; Settings can inspect, sync and trigger them")
            value: String(root.jobs.length)
        }
        RowButton {
            icon: "sync"
            text: qsTr("Sync cron registry")
            subtext: qsTr("reconcile Hermes cron state with the declarative Vesper job registry")
            disabled: action.running
            onClicked: root.run(["@hermesAutomation@", "sync-cron", "--prune"])
        }

        SectionHeader { text: qsTr("Jobs") }
        Repeater {
            model: root.jobs
            delegate: ColumnLayout {
                id: jobRow
                required property var modelData
                readonly property var last: modelData.lastRun || ({})
                Layout.fillWidth: true

                InfoRow {
                    Layout.fillWidth: true
                    icon: jobRow.last.status === "error"
                        ? "error"
                        : (jobRow.last.status === "ok" ? "check_circle" : "event_repeat")
                    label: jobRow.modelData._name
                    subtext: qsTr("%1 · %2 · %3")
                        .arg(jobRow.modelData.schedule || qsTr("manual"))
                        .arg(jobRow.modelData.mode || "agent")
                        .arg(jobRow.modelData.description || jobRow.modelData.task || "")
                    value: jobRow.last.status || qsTr("never run")
                }

                InfoRow {
                    Layout.fillWidth: true
                    visible: !!jobRow.last.finishedAt || !!jobRow.last.startedAt
                    icon: "history"
                    label: qsTr("Last run")
                    subtext: jobRow.last.error || qsTr("no error reported")
                    value: jobRow.last.finishedAt || jobRow.last.startedAt || ""
                }

                RowLayout {
                    Layout.fillWidth: true
                    Item { Layout.fillWidth: true }
                    IconTextButton {
                        isRound: true
                        icon: "play_arrow"
                        text: qsTr("Run now")
                        disabled: action.running || jobRow.modelData.enabled === false
                        onClicked: root.run(["@hermesAutomation@", "trigger", jobRow.modelData._name])
                    }
                }
            }
        }

        InfoRow {
            visible: root.jobs.length === 0
            icon: "info"
            label: qsTr("No declarative jobs")
            subtext: qsTr("The Hermes registry is empty or unavailable")
            value: qsTr("empty")
        }

        SectionHeader { text: qsTr("Briefing history") }
        InfoRow {
            icon: "inbox"
            label: qsTr("Stored briefings")
            subtext: qsTr("durable local Hermes reports; newest first")
            value: String(root.briefings.length)
        }
        RowButton {
            visible: root.briefings.some(item => item.unread === true)
            icon: "done_all"
            text: qsTr("Mark all briefings read")
            subtext: qsTr("updates only local briefing metadata")
            disabled: action.running
            onClicked: root.run(["vesper-hermes", "mark-all-read"])
        }
        Repeater {
            model: root.briefings.slice(0, 10)
            delegate: ColumnLayout {
                id: briefingRow
                required property var modelData
                Layout.fillWidth: true

                InfoRow {
                    Layout.fillWidth: true
                    icon: briefingRow.modelData.unread ? "mark_email_unread" : "article"
                    label: briefingRow.modelData.title || briefingRow.modelData.id || qsTr("Hermes briefing")
                    subtext: briefingRow.modelData.summary || briefingRow.modelData.lane || ""
                    value: briefingRow.modelData.priority || qsTr("normal")
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: briefingRow.modelData.unread === true
                    Item { Layout.fillWidth: true }
                    IconTextButton {
                        isRound: true
                        icon: "done"
                        text: qsTr("Mark read")
                        disabled: action.running || !briefingRow.modelData.id
                        onClicked: root.run(["vesper-hermes", "mark-read", briefingRow.modelData.id])
                    }
                }
            }
        }

        SectionHeader { text: qsTr("Learned source registry") }
        InfoRow {
            icon: "travel_explore"
            label: qsTr("Adaptive sources")
            subtext: qsTr("Hermes promotes sources from probation as repeated useful evidence is observed")
            value: String(root.sourceRegistry.count || 0)
        }
        Repeater {
            model: (root.sourceRegistry.sources || []).slice(0, 10)
            delegate: InfoRow {
                required property var modelData
                Layout.fillWidth: true
                icon: modelData.tier === "promoted" ? "verified" : "link"
                label: modelData.url || modelData.id || qsTr("source")
                subtext: qsTr("hits %1 · score %2 · last useful %3")
                    .arg(modelData.hits ?? 0)
                    .arg(modelData.score ?? 0)
                    .arg(modelData.lastUseful || modelData.lastSeen || qsTr("unknown"))
                value: modelData.tier || qsTr("probation")
            }
        }

        NavRow {
            icon: "rate_review"
            text: qsTr("Review Hermes skill drafts")
            subtext: qsTr("generated drafts stay outside the canonical skill registry until explicit promotion")
            onClicked: root.nState.openSubPageRoute("skillsMcp")
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("completed") ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
