pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.modules.nexus.common

PageBase {
    id: root

    property var control: ({ credentials: [], skills: { count: 0, items: [] }, mcp: { count: 0, items: [] }, hermesRegistry: false })
    property var hub: ({ summary: {}, agents: {}, hermes: {}, providers: [], stale: true })
    property string loadError: ""

    readonly property var credentials: control.credentials || []
    readonly property var skills: control.skills || ({ count: 0, items: [] })
    readonly property var mcp: control.mcp || ({ count: 0, items: [] })
    readonly property int configuredKeys: credentials.filter(item => item.configured).length

    title: qsTr("AI")

    function refresh() {
        if (!controlStatus.running)
            controlStatus.running = true;
        if (!hubStatus.running)
            hubStatus.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Process {
        id: controlStatus
        command: ["@vesperControl@", "ai-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.control = JSON.parse(text);
                    root.loadError = "";
                } catch (e) {
                    root.loadError = qsTr("AI settings returned invalid data");
                }
            }
        }
    }

    Process {
        id: hubStatus
        command: ["@aiHub@", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.hub = JSON.parse(text);
                } catch (e) {
                    // The settings inventory remains useful even if the usage backend is stale.
                }
            }
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
            icon: "key"
            label: qsTr("API keys")
            subtext: qsTr("Secret Service · API key only")
            value: qsTr("%1 / %2").arg(root.configuredKeys).arg(root.credentials.length)
        }

        InfoRow {
            icon: "smart_toy"
            label: qsTr("Providers")
            subtext: root.hub.stale ? qsTr("usage snapshot is stale") : qsTr("usage and limits")
            value: String(root.hub.summary?.providerCount ?? root.hub.providers?.length ?? 0)
        }

        InfoRow {
            icon: "robot_2"
            label: qsTr("Agents")
            subtext: qsTr("live agent processes")
            value: String(root.hub.agents?.count ?? 0)
        }

        InfoRow {
            icon: "extension"
            label: qsTr("Skills")
            subtext: qsTr("canonical ~/.agents/skills")
            value: String(root.skills.count || 0)
        }

        InfoRow {
            icon: "hub"
            label: qsTr("MCP")
            subtext: qsTr("shared Home Manager registry")
            value: String(root.mcp.count || 0)
        }

        InfoRow {
            icon: "schedule"
            label: qsTr("Hermes")
            subtext: root.control.hermesRegistry ? qsTr("job registry available") : qsTr("job registry unavailable")
            value: qsTr("%1 unread").arg(root.hub.hermes?.unread ?? 0)
        }

        SectionHeader {
            text: qsTr("Credentials")
        }

        NavRow {
            icon: "password"
            text: qsTr("Manage API keys")
            subtext: qsTr("stored in the desktop keyring and injected only into selected processes")
            onClicked: root.nState.openSubPage(1)
        }

        SectionHeader {
            text: qsTr("Skills")
        }

        Repeater {
            model: root.skills.items || []

            delegate: InfoRow {
                required property string modelData
                icon: "bolt"
                label: modelData
                value: qsTr("active")
            }
        }

        SectionHeader {
            text: qsTr("MCP")
        }

        Repeater {
            model: root.mcp.items || []

            delegate: InfoRow {
                required property string modelData
                icon: "cable"
                label: modelData
                subtext: qsTr("available to configured coding agents")
                value: qsTr("enabled")
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
