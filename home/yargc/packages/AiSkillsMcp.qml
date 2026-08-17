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
    property var skillState: ({ skills: [], drafts: [] })
    property var mcpState: ({ servers: [] })
    property string message: ""
    title: qsTr("Skills & MCP")
    isSubPage: true

    function refresh() {
        if (!skillsStatus.running) skillsStatus.running = true;
        if (!mcpStatus.running) mcpStatus.running = true;
    }

    function run(command) {
        if (action.running) return;
        root.message = "";
        action.command = command;
        action.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: skillsStatus
        command: ["@vesperControl@", "skills", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try { root.skillState = JSON.parse(text); }
                catch (e) { root.message = qsTr("Could not read skill registry"); }
            }
        }
    }

    Process {
        id: mcpStatus
        command: ["@vesperControl@", "mcp", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try { root.mcpState = JSON.parse(text); }
                catch (e) { root.message = qsTr("Could not read MCP registry"); }
            }
        }
    }

    Process {
        id: action
        stderr: StdioCollector { id: actionError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("AI control-plane action completed") : actionError.text.trim();
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Skills") }
        InfoRow {
            icon: "extension"
            label: qsTr("Canonical registry")
            subtext: qsTr("Nix-owned skills stay immutable; Vesper runtime skills can be enabled, disabled or removed")
            value: String((root.skillState.skills || []).length)
        }

        Repeater {
            model: root.skillState.skills || []
            delegate: ColumnLayout {
                id: skillRow
                required property var modelData
                Layout.fillWidth: true

                InfoRow {
                    Layout.fillWidth: true
                    icon: skillRow.modelData.ownership === "nix" ? "deployed_code" : (skillRow.modelData.ownership === "runtime" ? "edit" : "link")
                    label: skillRow.modelData.name
                    subtext: skillRow.modelData.ownership === "nix"
                        ? qsTr("Nix-managed · edit declarative source to change")
                        : (skillRow.modelData.ownership === "runtime"
                            ? qsTr("Vesper runtime-owned · reviewed/promoted locally")
                            : qsTr("External/manual · informational"))
                    value: skillRow.modelData.enabled === false ? qsTr("disabled") : qsTr("enabled")
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: skillRow.modelData.mutable === true
                    Item { Layout.fillWidth: true }
                    IconTextButton {
                        isRound: true
                        icon: skillRow.modelData.enabled === false ? "play_arrow" : "pause"
                        text: skillRow.modelData.enabled === false ? qsTr("Enable") : qsTr("Disable")
                        disabled: action.running
                        onClicked: root.run(["@vesperControl@", "skills", skillRow.modelData.enabled === false ? "enable" : "disable", skillRow.modelData.name])
                    }
                    IconTextButton {
                        isRound: true
                        icon: "delete"
                        text: qsTr("Remove")
                        disabled: action.running
                        onClicked: root.run(["@vesperControl@", "skills", "remove", skillRow.modelData.name])
                    }
                }
            }
        }

        SectionHeader { text: qsTr("Generated drafts") }
        InfoRow {
            visible: (root.skillState.drafts || []).length === 0
            icon: "verified"
            label: qsTr("No drafts awaiting promotion")
            subtext: qsTr("Generated skills remain outside the canonical registry until explicitly promoted")
            value: qsTr("reviewed flow")
        }
        Repeater {
            model: root.skillState.drafts || []
            delegate: RowButton {
                required property string modelData
                icon: "rate_review"
                text: modelData
                subtext: qsTr("Promote only after reviewing SKILL.md and bundled files")
                disabled: action.running
                onClicked: root.run(["@vesperControl@", "skills", "promote", modelData])
            }
        }

        SectionHeader { text: qsTr("MCP") }
        InfoRow {
            icon: "hub"
            label: qsTr("Configured servers")
            subtext: qsTr("Ownership is explicit; declarative Nix servers are not mutated from Settings")
            value: String((root.mcpState.servers || []).length)
        }
        Repeater {
            model: root.mcpState.servers || []
            delegate: InfoRow {
                required property var modelData
                Layout.fillWidth: true
                icon: modelData.state === "running" ? "check_circle" : "hub"
                label: modelData.name
                subtext: modelData.command
                    ? qsTr("%1 · %2").arg(modelData.ownership || "unknown").arg(modelData.command)
                    : qsTr("%1 · command metadata unavailable").arg(modelData.ownership || "unknown")
                value: modelData.mutable === true ? (modelData.state || qsTr("configured")) : qsTr("declarative")
            }
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("completed") ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
