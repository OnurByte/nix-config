pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.services
import qs.modules.nexus.common

PageBase {
    id: root

    property var control: ({ credentials: [], skills: { count: 0, items: [] }, mcp: { count: 0, items: [] }, hermesRegistry: false })
    property var hub: ({ summary: {}, agents: {}, hermes: {}, providers: [], stale: true })
    property var icons: ({ enabled: false, mode: "original", provider: "openai", providerConfigured: false, remoteConsent: false, followPalette: true, discovered: 0, canonical: 0, pending: 0, failed: 0, excluded: 0, active: 0 })
    property var iconQueue: ({ total: 0, pending: 0, ready: 0, running: 0, retryWait: 0, blockedNoProvider: 0, blockedNoConsent: 0, consentGranted: false, failed: 0, succeeded: 0, superseded: 0, paused: false, transport: "active" })
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
        if (!iconQueueStatus.running)
            iconQueueStatus.running = true;
    }

    function runIcon(args) {
        if (iconChange.running)
            return;
        root.iconMessage = "";
        iconChange.command = ["@vesperControl@", "icon"].concat(args);
        iconChange.running = true;
    }

    property list<MenuItem> providerItems: [
        MenuItem {
            objectName: "openai"
            text: "OpenAI"
            icon: "smart_toy"
            onClicked: root.runIcon(["provider", "openai"])
        },
        MenuItem {
            objectName: "anthropic"
            text: "Anthropic"
            icon: "smart_toy"
            onClicked: root.runIcon(["provider", "anthropic"])
        },
        MenuItem {
            objectName: "xai"
            text: "xAI"
            icon: "smart_toy"
            onClicked: root.runIcon(["provider", "xai"])
        },
        MenuItem {
            objectName: "openrouter"
            text: "OpenRouter"
            icon: "route"
            onClicked: root.runIcon(["provider", "openrouter"])
        },
        MenuItem {
            objectName: "google"
            text: "Google AI"
            icon: "smart_toy"
            onClicked: root.runIcon(["provider", "google"])
        }
    ]

    Component.onCompleted: refresh()

    Timer {
        interval: 5000
        repeat: true
        running: root.visible && ((root.iconQueue.pending || 0) > 0 || (root.iconQueue.running || 0) > 0)
        onTriggered: root.refresh()
    }

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
        id: iconQueueStatus
        command: ["@vesperControl@", "icon", "queue-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.iconQueue = JSON.parse(text);
                } catch (e) {
                    // Inventory remains usable if queue state is temporarily unavailable.
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
            text: qsTr("Automatic adaptive icons")
            subtext: qsTr("safe vectors stay local; raster and unsuitable sources are canonicalized through the bounded conversion worker")
            checked: root.icons.enabled || false
            disabled: iconChange.running
            onToggled: root.runIcon([checked ? "enable" : "disable"])
        }

        ToggleRow {
            text: qsTr("Allow remote icon analysis")
            subtext: qsTr("send only normalized application icon artwork to the selected provider when semantic conversion is required")
            checked: root.icons.remoteConsent || false
            disabled: iconChange.running
            onToggled: root.runIcon(["remote-consent", checked ? "on" : "off"])
        }

        SelectRow {
            label: qsTr("Conversion provider")
            subtext: root.icons.providerConfigured ? qsTr("existing Secret Service credential is ready") : qsTr("selected provider needs an API key")
            fallbackIcon: "smart_toy"
            fallbackText: root.providerName(root.icons.provider || "openai")
            menuItems: root.providerItems
            active: root.providerItems.find(item => item.objectName === root.icons.provider) ?? null
        }

        InfoRow {
            icon: "tune"
            label: qsTr("Conversion model")
            subtext: qsTr("provider default · can be overridden in adaptive-icons-ai.conf")
            value: qsTr("Auto")
        }

        InfoRow {
            icon: "apps"
            label: qsTr("Discovered apps")
            subtext: qsTr("resolved from effective XDG desktop entries")
            value: String(root.icons.discovered || 0)
        }

        InfoRow {
            icon: "verified"
            label: qsTr("Canonical icons")
            subtext: qsTr("accepted canonical assets with .vicon packages")
            value: String(root.icons.canonical || 0)
        }

        InfoRow {
            icon: "hourglass_top"
            label: qsTr("Conversion queue")
            subtext: root.iconQueue.blockedNoConsent > 0
                ? qsTr("%1 source jobs are waiting for remote-analysis consent").arg(root.iconQueue.blockedNoConsent)
                : root.iconQueue.blockedNoProvider > 0
                    ? qsTr("%1 source jobs are waiting for the selected provider").arg(root.iconQueue.blockedNoProvider)
                    : root.iconQueue.paused
                        ? qsTr("queue is paused")
                        : qsTr("SQLite-backed source-hash queue")
            value: qsTr("%1 pending").arg(root.iconQueue.pending || root.icons.pending || 0)
        }

        InfoRow {
            icon: "play_arrow"
            label: qsTr("Worker")
            subtext: qsTr("%1 ready · %2 retrying").arg(root.iconQueue.ready || 0).arg(root.iconQueue.retryWait || 0)
            value: root.iconQueue.running > 0 ? qsTr("%1 running").arg(root.iconQueue.running) : qsTr("idle")
        }

        InfoRow {
            icon: "check_circle"
            label: qsTr("Completed conversions")
            subtext: qsTr("accepted semantic conversions kept in canonical cache")
            value: String(root.iconQueue.succeeded || 0)
        }

        InfoRow {
            icon: "error"
            label: qsTr("Failed")
            subtext: qsTr("original packaged icons remain as fallback")
            value: String((root.icons.failed || 0) + (root.iconQueue.failed || 0))
        }

        InfoRow {
            icon: "palette"
            label: qsTr("Active Vesper icons")
            subtext: qsTr("compiled into the generated freedesktop theme")
            value: String(root.icons.active || 0)
        }

        RowButton {
            icon: root.iconQueue.paused ? "play_arrow" : "pause"
            text: root.iconQueue.paused ? qsTr("Resume conversion queue") : qsTr("Pause conversion queue")
            subtext: root.iconQueue.paused ? qsTr("continue pending semantic conversions") : qsTr("keep current icons and stop starting new remote work")
            trailingIcon: root.iconQueue.paused ? "play_arrow" : "pause"
            disabled: iconChange.running
            onClicked: root.runIcon([root.iconQueue.paused ? "queue-resume" : "queue-pause"])
        }

        RowButton {
            icon: "sync"
            text: qsTr("Reconcile icon inventory")
            subtext: qsTr("rescan desktop entries, validate sources and rebuild the local theme")
            trailingIcon: "refresh"
            disabled: iconChange.running
            onClicked: root.runIcon(["reconcile"])
        }

        RowButton {
            icon: "restart_alt"
            text: qsTr("Rebuild canonical assets")
            subtext: qsTr("discard the local canonical cache and reconstruct installed sources")
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