pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

Item {
    id: root

    property var payload: ({
        summary: { providerCount: 0, criticalCount: 0, warningCount: 0, maxUsedPercent: -1, maxProvider: "", class: "stale" },
        providers: [],
        agents: { count: 0, agents: [] },
        hermes: { unread: 0, high: 0, latestTitle: "" },
        privacy: { tor: "unknown", mic: "unknown", camera: "unknown", clipboard: "unknown", node: "unknown", class: "unknown", label: "--" },
        stale: true
    })
    property string loadError: ""

    readonly property var summary: payload.summary || ({})
    readonly property var agents: payload.agents || ({})
    readonly property var hermes: payload.hermes || ({})
    readonly property var privacy: payload.privacy || ({})
    readonly property var providers: payload.providers || []
    readonly property color stateColour: summary.class === "critical"
        ? Colours.palette.m3error
        : summary.class === "warning"
            ? Colours.palette.m3tertiary
            : Colours.palette.m3primary
    readonly property color privacyColour: privacy.class === "alert"
        ? Colours.palette.m3error
        : privacy.class === "attention"
            ? Colours.palette.m3tertiary
            : privacy.class === "private"
                ? Colours.palette.m3primary
                : Colours.palette.m3outline

    implicitWidth: 760
    implicitHeight: 520

    function applyPayload(text) {
        try {
            root.payload = JSON.parse(text);
            root.loadError = root.payload.backendError || "";
        } catch (e) {
            root.loadError = "AI Hub returned invalid data";
        }
    }

    function refresh(force) {
        if (force) {
            if (!forceRefresh.running)
                forceRefresh.running = true;
        } else if (!status.running) {
            status.running = true;
        }
    }

    function formatReset(value) {
        if (!value)
            return "";
        const d = new Date(value);
        if (isNaN(d.getTime()))
            return value;
        const pad = n => String(n).padStart(2, "0");
        return `${pad(d.getDate())}.${pad(d.getMonth() + 1)} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
    }

    function privacyDetail() {
        const parts = [
            `tor ${root.privacy.tor || "unknown"}`,
            `mic ${root.privacy.mic || "unknown"}`,
            `cam ${root.privacy.camera || "unknown"}`,
            `xmr ${root.privacy.node || "off"}`
        ];
        return parts.join(" · ");
    }

    Component.onCompleted: refresh(false)

    Timer {
        interval: 60000
        repeat: true
        running: true
        onTriggered: root.refresh(false)
    }

    Process {
        id: status
        command: ["@aiHub@", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.applyPayload(text)
        }
    }

    Process {
        id: forceRefresh
        command: ["@aiHub@", "refresh"]
        stdout: StdioCollector {
            onStreamFinished: root.applyPayload(text)
        }
    }

    RowLayout {
        id: header
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        spacing: Tokens.spacing.medium

        MaterialIcon {
            text: "smart_toy"
            color: root.stateColour
            fill: 1
            fontStyle: Tokens.font.icon.large
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0

            StyledText {
                text: qsTr("Vesper Hub")
                font: Tokens.font.title.large
                color: Colours.palette.m3onSurface
            }

            StyledText {
                Layout.fillWidth: true
                text: {
                    if (root.payload.stale)
                        return root.loadError ? `stale · ${root.loadError}` : "stale snapshot";
                    if ((root.summary.maxUsedPercent ?? -1) >= 0)
                        return `${root.summary.maxProvider || "Provider"} is most constrained · ${Math.round(root.summary.maxUsedPercent)}% used`;
                    return `${root.summary.providerCount || 0} enabled providers`;
                }
                elide: Text.ElideRight
                color: root.payload.stale ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                font: Tokens.font.body.small
            }
        }

        StyledRect {
            implicitWidth: refreshIcon.implicitWidth + Tokens.padding.medium * 2
            implicitHeight: refreshIcon.implicitHeight + Tokens.padding.medium * 2
            radius: Tokens.rounding.full
            color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, refreshMouse.containsMouse ? 0.72 : 0.38)

            MaterialIcon {
                id: refreshIcon
                anchors.centerIn: parent
                text: "refresh"
                color: Colours.palette.m3primary
                fontStyle: Tokens.font.icon.medium
            }

            MouseArea {
                id: refreshMouse
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.refresh(true)
            }
        }
    }

    GridLayout {
        id: summaries
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: header.bottom
        anchors.topMargin: Tokens.spacing.large
        columns: 2
        columnSpacing: Tokens.spacing.medium
        rowSpacing: Tokens.spacing.medium

        SummaryCard {
            Layout.fillWidth: true
            iconName: "toll"
            title: qsTr("Providers")
            value: `${root.summary.providerCount || 0}`
            detail: `${root.summary.criticalCount || 0} critical · ${root.summary.warningCount || 0} warning`
            accent: root.stateColour
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "terminal"
            title: qsTr("Agents")
            value: `${root.agents.count || 0}`
            detail: {
                const items = root.agents.agents || [];
                if (!items.length)
                    return "no active coding agents";
                return items.slice(0, 3).map(a => `${a.agent} · ${a.project}`).join("  •  ");
            }
            accent: (root.agents.count || 0) > 0 ? Colours.palette.m3secondary : Colours.palette.m3outline
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "auto_awesome"
            title: qsTr("Hermes")
            value: `${root.hermes.unread || 0}`
            detail: root.hermes.latestTitle || "no briefings yet"
            accent: (root.hermes.high || 0) > 0 ? Colours.palette.m3error : Colours.palette.m3tertiary
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "shield_lock"
            title: qsTr("Privacy")
            value: root.privacy.label || "--"
            detail: root.privacyDetail()
            accent: root.privacyColour
        }
    }

    Flickable {
        id: contentView
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: summaries.bottom
        anchors.bottom: parent.bottom
        anchors.topMargin: Tokens.spacing.large
        clip: true
        contentWidth: width
        contentHeight: contentColumn.implicitHeight
        flickableDirection: Flickable.VerticalFlick
        boundsBehavior: Flickable.StopAtBounds

        ColumnLayout {
            id: contentColumn
            width: contentView.width
            spacing: Tokens.spacing.medium

            StyledText {
                Layout.fillWidth: true
                visible: (root.agents.agents || []).length > 0
                text: qsTr("Active agents")
                color: Colours.palette.m3onSurfaceVariant
                font: Tokens.font.title.small
            }

            Repeater {
                model: root.agents.agents || []

                delegate: AgentCard {
                    required property var modelData
                    Layout.fillWidth: true
                    agentData: modelData
                }
            }

            StyledText {
                Layout.fillWidth: true
                text: qsTr("AI providers")
                color: Colours.palette.m3onSurfaceVariant
                font: Tokens.font.title.small
            }

            StyledText {
                Layout.fillWidth: true
                visible: root.providers.length === 0
                text: root.loadError || qsTr("No enabled CodexBar providers returned data")
                color: Colours.palette.m3onSurfaceVariant
                horizontalAlignment: Text.AlignHCenter
                font: Tokens.font.body.medium
            }

            Repeater {
                model: root.providers

                delegate: ProviderCard {
                    required property var modelData
                    Layout.fillWidth: true
                    providerData: modelData
                }
            }
        }
    }

    component SummaryCard: StyledRect {
        id: summaryCard

        required property string iconName
        required property string title
        required property string value
        required property string detail
        required property color accent

        implicitHeight: summaryContent.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer

        RowLayout {
            id: summaryContent
            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: summaryCard.iconName
                color: summaryCard.accent
                fill: 1
                fontStyle: Tokens.font.icon.medium
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                RowLayout {
                    Layout.fillWidth: true

                    StyledText {
                        Layout.fillWidth: true
                        text: summaryCard.title
                        color: Colours.palette.m3onSurfaceVariant
                        font: Tokens.font.body.small
                    }

                    StyledText {
                        text: summaryCard.value
                        color: summaryCard.accent
                        font: Tokens.font.title.medium
                    }
                }

                StyledText {
                    Layout.fillWidth: true
                    text: summaryCard.detail
                    elide: Text.ElideRight
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.body.small
                }
            }
        }
    }

    component AgentCard: StyledRect {
        id: agentCard
        required property var agentData

        implicitHeight: agentContent.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer
        border.width: 1
        border.color: Qt.alpha(Colours.palette.m3secondary, 0.16)

        RowLayout {
            id: agentContent
            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: "terminal"
                color: Colours.palette.m3secondary
                fontStyle: Tokens.font.icon.medium
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                StyledText {
                    Layout.fillWidth: true
                    text: `${agentCard.agentData.agent || "Agent"} · ${agentCard.agentData.project || "unknown"}`
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.title.small
                }

                StyledText {
                    Layout.fillWidth: true
                    text: {
                        const branch = agentCard.agentData.branch || "detached";
                        const dirty = agentCard.agentData.dirty ? "dirty" : "clean";
                        const age = Number(agentCard.agentData.elapsedSeconds || 0);
                        return `${branch} · ${dirty} · pid ${agentCard.agentData.pid || "?"} · ${age}s`;
                    }
                    elide: Text.ElideRight
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                }

                StyledText {
                    Layout.fillWidth: true
                    visible: !!agentCard.agentData.cwd
                    text: agentCard.agentData.cwd || ""
                    elide: Text.ElideMiddle
                    color: Colours.palette.m3outline
                    font: Tokens.font.body.small
                }
            }
        }
    }

    component ProviderCard: StyledRect {
        id: card
        required property var providerData

        readonly property color accent: providerData.health === "critical"
            ? Colours.palette.m3error
            : providerData.health === "warning"
                ? Colours.palette.m3tertiary
                : Colours.palette.m3primary

        implicitHeight: cardContent.implicitHeight + Tokens.padding.large * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer
        border.width: 1
        border.color: Qt.alpha(card.accent, 0.16)

        ColumnLayout {
            id: cardContent
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Tokens.padding.large
            spacing: Tokens.spacing.small

            RowLayout {
                Layout.fillWidth: true
                spacing: Tokens.spacing.medium

                MaterialIcon {
                    text: card.providerData.health === "critical" ? "error" : "neurology"
                    color: card.accent
                    fill: card.providerData.health === "critical" ? 1 : 0
                    fontStyle: Tokens.font.icon.medium
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 0

                    StyledText {
                        Layout.fillWidth: true
                        text: card.providerData.name || card.providerData.id
                        color: Colours.palette.m3onSurface
                        font: Tokens.font.title.medium
                    }

                    StyledText {
                        Layout.fillWidth: true
                        text: {
                            const parts = [];
                            if (card.providerData.plan)
                                parts.push(card.providerData.plan);
                            if (card.providerData.source)
                                parts.push(card.providerData.source);
                            if (card.providerData.account)
                                parts.push(card.providerData.account);
                            return parts.join(" · ") || (card.providerData.statusLabel || card.providerData.status || "provider");
                        }
                        elide: Text.ElideRight
                        color: Colours.palette.m3onSurfaceVariant
                        font: Tokens.font.body.small
                    }
                }

                StyledText {
                    visible: card.providerData.maxUsedPercent !== null && card.providerData.maxUsedPercent !== undefined
                    text: `${Math.round(card.providerData.maxUsedPercent || 0)}%`
                    color: card.accent
                    font: Tokens.font.headline.small
                }
            }

            Repeater {
                model: card.providerData.windows || []

                delegate: UsageRow {
                    required property var modelData
                    Layout.fillWidth: true
                    windowData: modelData
                    accent: card.accent
                }
            }

            RowLayout {
                Layout.fillWidth: true
                visible: !!card.providerData.credits || !!card.providerData.cost
                spacing: Tokens.spacing.large

                StyledText {
                    visible: !!card.providerData.credits
                    text: {
                        const c = card.providerData.credits || {};
                        return c.remaining === null || c.remaining === undefined
                            ? ""
                            : `credits ${c.remaining}${c.unit ? ` ${c.unit}` : ""}`;
                    }
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                }

                StyledText {
                    Layout.fillWidth: true
                    visible: !!card.providerData.cost
                    text: {
                        const c = card.providerData.cost || {};
                        const today = c.todayUSD === null || c.todayUSD === undefined ? "" : `$${Number(c.todayUSD).toFixed(2)} today`;
                        const month = c.last30DaysUSD === null || c.last30DaysUSD === undefined ? "" : `$${Number(c.last30DaysUSD).toFixed(2)} / 30d`;
                        return [today, month].filter(Boolean).join(" · ");
                    }
                    horizontalAlignment: Text.AlignRight
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                }
            }

            StyledText {
                Layout.fillWidth: true
                visible: !!card.providerData.error
                text: card.providerData.error || ""
                wrapMode: Text.Wrap
                color: Colours.palette.m3error
                font: Tokens.font.body.small
            }
        }
    }

    component UsageRow: ColumnLayout {
        id: usageRow
        required property var windowData
        required property color accent

        spacing: Tokens.spacing.extraSmall

        RowLayout {
            Layout.fillWidth: true

            StyledText {
                Layout.fillWidth: true
                text: usageRow.windowData.label || usageRow.windowData.kind || "Usage"
                color: Colours.palette.m3onSurface
                font: Tokens.font.body.medium
            }

            StyledText {
                text: {
                    const w = usageRow.windowData;
                    const usage = w.usedPercent === null || w.usedPercent === undefined ? "--" : `${Math.round(w.usedPercent)}% used`;
                    const reset = root.formatReset(w.resetAt);
                    return reset ? `${usage} · reset ${reset}` : usage;
                }
                color: Colours.palette.m3onSurfaceVariant
                font: Tokens.font.body.small
            }
        }

        StyledRect {
            Layout.fillWidth: true
            implicitHeight: Tokens.padding.small
            radius: Tokens.rounding.full
            color: Colours.palette.m3surfaceContainerHighest

            StyledRect {
                anchors.left: parent.left
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                width: parent.width * Math.max(0, Math.min(1, Number(usageRow.windowData.usedPercent || 0) / 100))
                radius: parent.radius
                color: usageRow.accent
            }
        }
    }
}
