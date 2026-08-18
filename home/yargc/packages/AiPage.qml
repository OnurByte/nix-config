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
    property var ai: ({ summary: {}, agents: {}, hermes: {}, providers: [], stale: true })
    property string loadError: ""
    readonly property var credentials: control.credentials || []
    readonly property var skills: control.skills || ({ count: 0, items: [] })
    readonly property var mcp: control.mcp || ({ count: 0, items: [] })
    readonly property int configuredKeys: credentials.filter(item => item.configured).length
    title: qsTr("AI")

    function refresh() {
        if (!controlStatus.running) controlStatus.running = true;
        if (!aiStatus.running) aiStatus.running = true;
    }
    Component.onCompleted: refresh()
    Timer { interval: 30000; repeat: true; running: root.visible; onTriggered: root.refresh() }
    Process {
        id: controlStatus
        command: ["@vesperControl@", "ai-status"]
        stdout: StdioCollector { onStreamFinished: { try { root.control = JSON.parse(text); root.loadError = ""; } catch (e) { root.loadError = qsTr("AI settings returned invalid data"); } } }
    }
    Process {
        id: aiStatus
        command: ["@ai@", "status"]
        stdout: StdioCollector { onStreamFinished: { try { root.ai = JSON.parse(text); } catch (e) {} } }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Overview") }
        InfoRow { icon: "key"; label: qsTr("API keys"); subtext: qsTr("Secret Service · API key only"); value: qsTr("%1 / %2").arg(root.configuredKeys).arg(root.credentials.length) }
        InfoRow { icon: "smart_toy"; label: qsTr("Providers"); subtext: root.ai.stale ? qsTr("usage snapshot is stale") : qsTr("usage and limits"); value: String(root.ai.summary?.providerCount ?? root.ai.providers?.length ?? 0) }
        InfoRow { icon: "robot_2"; label: qsTr("Agents"); subtext: qsTr("live agent processes"); value: String(root.ai.agents?.count ?? 0) }
        InfoRow { icon: "extension"; label: qsTr("Skills"); subtext: qsTr("canonical ~/.agents/skills"); value: String(root.skills.count || 0) }
        InfoRow { icon: "hub"; label: qsTr("MCP"); subtext: qsTr("shared Home Manager registry"); value: String(root.mcp.count || 0) }
        InfoRow { icon: "schedule"; label: qsTr("Hermes"); subtext: root.control.hermesRegistry ? qsTr("job registry available") : qsTr("job registry unavailable"); value: qsTr("%1 unread").arg(root.ai.hermes?.unread ?? 0) }

        SectionHeader { text: qsTr("Control plane") }
        NavRow { icon: "data_usage"; text: qsTr("Usage & Quotas"); subtext: qsTr("provider health · quota windows · resets · credits · costs"); onClicked: root.nState.openSubPageRoute("usageQuotas") }
        NavRow { icon: "password"; text: qsTr("Manage API keys"); subtext: qsTr("desktop keyring · no plaintext files"); onClicked: root.nState.openSubPageRoute("apiKeys") }
        NavRow { icon: "palette"; text: qsTr("App Icons"); subtext: qsTr("semantic SVG curation · Original / Light / Dark / Tinted / Clear"); onClicked: root.nState.openSubPageRoute("appIcons") }
        NavRow { icon: "route"; text: qsTr("Runtime Credentials"); subtext: qsTr("map OpenCode and Hermes to credential aliases"); onClicked: root.nState.openSubPageRoute("runtimeCredentials") }
        NavRow { icon: "dns"; text: qsTr("Providers"); subtext: qsTr("built-in/custom endpoints · model · budget · default/fallback routing"); onClicked: root.nState.openSubPageRoute("providers") }
        NavRow { icon: "extension"; text: qsTr("Skills & MCP"); subtext: qsTr("runtime skill promotion with explicit ownership; Nix-owned MCP inventory"); onClicked: root.nState.openSubPageRoute("skillsMcp") }
        NavRow { icon: "schedule"; text: qsTr("Hermes Jobs"); subtext: qsTr("inspect, sync and run declarative research jobs"); onClicked: root.nState.openSubPageRoute("hermes") }

        StyledText { Layout.fillWidth: true; Layout.topMargin: Tokens.spacing.medium; visible: root.loadError; text: root.loadError; color: Colours.palette.m3error; font: Tokens.font.body.small; wrapMode: Text.WordWrap }
    }
}
