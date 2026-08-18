pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services
import qs.modules.nexus.common

PageBase {
    id: root

    property var control: ({ credentials: [], skills: { count: 0, items: [] }, mcp: { count: 0, items: [] }, hermesRegistry: false })
    property var hub: ({ summary: {}, agents: {}, hermes: {}, providers: [], stale: true })
    property var icons: ({ enabled: false, automatic: true, remoteConsent: false, provider: "openai", model: "gpt-5.6", providerConfigured: false, discovered: 0, canonical: 0, pending: 0, running: 0, retry: 0, failed: 0, blocked: 0, active: 0, queuePaused: false, progress: "" })
    property string loadError: ""
    property string iconMessage: ""

    readonly property var credentials: control.credentials || []
    readonly property var skills: control.skills || ({ count: 0, items: [] })
    readonly property var mcp: control.mcp || ({ count: 0, items: [] })
    readonly property int configuredKeys: credentials.filter(item => item.configured).length

    title: qsTr("AI")

    function providerName(id) {
        const match = root.credentials.find(item => item.id === id);
        return match ? match.name : id;
    }

    function refresh() {
        if (!controlStatus.running)
            controlStatus.running = true;
        if (!hubStatus.running)
            hubStatus.running = true;
        if (!iconStatus.running)
            iconStatus.running = true;
    }

    function runIcon(args) {
        if (iconChange.running)
            return;
        root.iconMessage = "";
        iconChange.command = ["@vesperControl@", "icon"].concat(args);
        iconChange.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 15000
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
                    // Credential and icon controls remain available if usage telemetry is stale.
                }
            }
        }
    }

    Process {
        id: iconStatus
        command: ["@vesperControl@", "icon", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.icons = JSON.parse(text);
                } catch (e) {
                    root.iconMessage = qsTr("Could not read adaptive icon status");
                }
            }
        }
    }

    Process {
        id: iconChange
        stderr: StdioCollector { id: iconError }
        onExited: (code, status) => {
            root.iconMessage = code === 0 ? "" : iconError.text.trim();
            root.refresh();
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
            text: qsTr("Adaptive icons")
        }

        ToggleRow {
            text: qsTr("Use adaptive icons")
            subtext: qsTr("serve accepted Vesper icons with packaged icons as fallback")
            checked: root.icons.enabled || false
            disabled: iconChange.running
            onToggled: root.runIcon([checked ? "enable" : "disable"])
        }

        ToggleRow {
            text: qsTr("Automatic canonicalization")
            subtext: qsTr("discover changed app icons and process the persistent conversion queue")
            checked: root.icons.automatic ?? true
            disabled: iconChange.running
            onToggled: root.runIcon(["automatic", checked ? "on" : "off"])
        }

        ToggleRow {
            text: qsTr("Allow remote icon analysis")
            subtext: qsTr("send eligible app icon artwork to the selected AI provider for semantic decomposition")
            checked: root.icons.remoteConsent || false
            disabled: iconChange.running
            onToggled: root.runIcon(["consent", checked ? "on" : "off"])
        }

        InfoRow {
            icon: "smart_toy"
            label: qsTr("Conversion provider")
            subtext: root.icons.providerConfigured ? qsTr("reuses the existing Secret Service credential") : qsTr("API key is not configured")
            value: root.providerName(root.icons.provider || "openai")
        }

        InfoRow {
            icon: "neurology"
            label: qsTr("Conversion model")
            subtext: qsTr("image input + strict structured output")
            value: root.icons.model || qsTr("Auto")
        }

        InfoRow {
            icon: "apps"
            label: qsTr("Conversion progress")
            subtext: qsTr("%1 discovered · %2 active").arg(root.icons.discovered || 0).arg(root.icons.active || 0)
            value: root.icons.progress || qsTr("0 / 0 canonicalized")
        }

        InfoRow {
            icon: "hourglass_top"
            label: qsTr("Queue")
            subtext: qsTr("%1 running · %2 retry · %3 blocked").arg(root.icons.running || 0).arg(root.icons.retry || 0).arg(root.icons.blocked || 0)
            value: qsTr("%1 pending").arg(root.icons.pending || 0)
        }

        InfoRow {
            icon: "error"
            label: qsTr("Failed")
            subtext: qsTr("last-known-good or packaged icons stay active")
            value: String(root.icons.failed || 0)
        }

        RowButton {
            icon: root.icons.queuePaused ? "play_arrow" : "pause"
            text: root.icons.queuePaused ? qsTr("Resume conversion queue") : qsTr("Pause conversion queue")
            subtext: qsTr("local appearance switching and existing icons are unaffected")
            trailingIcon: root.icons.queuePaused ? "play_arrow" : "pause"
            disabled: iconChange.running
            onClicked: root.runIcon(["queue", root.icons.queuePaused ? "resume" : "pause"])
        }

        RowButton {
            icon: "restart_alt"
            text: qsTr("Retry failed icons")
            subtext: qsTr("reset failed conversion jobs without discarding accepted canonical packages")
            trailingIcon: "refresh"
            disabled: iconChange.running
            onClicked: root.runIcon(["retry-failed"])
        }

        RowButton {
            icon: "sync"
            text: qsTr("Reconcile icon inventory")
            subtext: qsTr("rescan XDG apps, sources and exact runtime identities")
            trailingIcon: "refresh"
            disabled: iconChange.running
            onClicked: root.runIcon(["reconcile"])
        }

        RowButton {
            icon: "restart_alt"
            text: qsTr("Regenerate canonical library")
            subtext: qsTr("discard accepted .vicon packages and enqueue eligible sources again")
            trailingIcon: "refresh"
            disabled: iconChange.running
            onClicked: root.runIcon(["rebuild-canonical"])
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
            visible: root.loadError || root.iconMessage
            text: root.loadError || root.iconMessage
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
