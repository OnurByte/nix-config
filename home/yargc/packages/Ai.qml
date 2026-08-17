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
            root.loadError = qsTr("AI returned invalid data");
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

    function openAiSettings() {
        if (!openSettings.running)
            openSettings.running = true;
    }

    function privacyDetail() {
        return [
            `tor ${root.privacy.tor || "unknown"}`,
            `mic ${root.privacy.mic || "unknown"}`,
            `cam ${root.privacy.camera || "unknown"}`,
            `xmr ${root.privacy.node || "off"}`
        ].join(" · ");
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
        command: ["@ai@", "status"]
        stdout: StdioCollector { onStreamFinished: root.applyPayload(text) }
    }

    Process {
        id: forceRefresh
        command: ["@ai@", "refresh"]
        stdout: StdioCollector { onStreamFinished: root.applyPayload(text) }
    }

    Process {
        id: openSettings
        command: ["caelestia-shell", "ipc", "call", "settings", "openPage", "AI"]
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Tokens.spacing.large

        RowLayout {
            Layout.fillWidth: true
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
                    text: qsTr("AI")
                    font: Tokens.font.title.large
                    color: Colours.palette.m3onSurface
                }

                StyledText {
                    Layout.fillWidth: true
                    text: {
                        if (root.payload.stale)
                            return root.loadError ? qsTr("stale · %1").arg(root.loadError) : qsTr("stale snapshot");
                        if ((root.summary.maxUsedPercent ?? -1) >= 0)
                            return qsTr("%1 is most constrained · %2% used")
                                .arg(root.summary.maxProvider || qsTr("Provider"))
                                .arg(Math.round(root.summary.maxUsedPercent));
                        return qsTr("%1 enabled providers").arg(root.summary.providerCount || 0);
                    }
                    elide: Text.ElideRight
                    color: root.payload.stale ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                }
            }

            ActionButton {
                iconName: "settings"
                tooltip: qsTr("Open AI Settings")
                busy: openSettings.running
                onTriggered: root.openAiSettings()
            }

            ActionButton {
                iconName: "refresh"
                tooltip: qsTr("Refresh AI status")
                busy: forceRefresh.running
                onTriggered: root.refresh(true)
            }
        }

        GridLayout {
            Layout.fillWidth: true
            columns: 2
            columnSpacing: Tokens.spacing.medium
            rowSpacing: Tokens.spacing.medium

            SummaryCard {
                Layout.fillWidth: true
                iconName: "toll"
                title: qsTr("Providers")
                value: `${root.summary.providerCount || 0}`
                detail: qsTr("%1 critical · %2 warning")
                    .arg(root.summary.criticalCount || 0)
                    .arg(root.summary.warningCount || 0)
                accent: root.stateColour
            }

            SummaryCard {
                Layout.fillWidth: true
                iconName: "terminal"
                title: qsTr("Agents")
                value: `${root.agents.count || 0}`
                detail: {
                    const items = root.agents.agents || [];
                    return items.length
                        ? items.slice(0, 2).map(a => `${a.agent} · ${a.project}`).join("  •  ")
                        : qsTr("no active coding agents");
                }
                accent: (root.agents.count || 0) > 0 ? Colours.palette.m3secondary : Colours.palette.m3outline
            }

            SummaryCard {
                Layout.fillWidth: true
                iconName: "auto_awesome"
                title: qsTr("Hermes")
                value: `${root.hermes.unread || 0}`
                detail: root.hermes.latestTitle || qsTr("no briefings yet")
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

        StyledText {
            Layout.fillWidth: true
            text: qsTr("Provider pressure")
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.title.small
        }

        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: width
            contentHeight: providerColumn.implicitHeight
            flickableDirection: Flickable.VerticalFlick
            boundsBehavior: Flickable.StopAtBounds

            ColumnLayout {
                id: providerColumn
                width: parent.width
                spacing: Tokens.spacing.small

                StyledText {
                    Layout.fillWidth: true
                    visible: root.providers.length === 0
                    text: root.loadError || qsTr("No enabled provider usage data")
                    color: Colours.palette.m3onSurfaceVariant
                    horizontalAlignment: Text.AlignHCenter
                    font: Tokens.font.body.medium
                }

                Repeater {
                    model: root.providers

                    delegate: ProviderRow {
                        required property var modelData
                        Layout.fillWidth: true
                        providerData: modelData
                    }
                }
            }
        }

        StyledText {
            Layout.fillWidth: true
            text: qsTr("Detailed quota windows, reset times, credits and cost breakdowns live in Settings → AI → Usage & Quotas")
            color: Colours.palette.m3outline
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }

    component ActionButton: StyledRect {
        id: action
        required property string iconName
        required property string tooltip
        required property bool busy
        signal triggered

        implicitWidth: actionIcon.implicitWidth + Tokens.padding.medium * 2
        implicitHeight: actionIcon.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.full
        color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, actionMouse.containsMouse ? 0.72 : 0.38)
        opacity: busy ? 0.55 : 1

        MaterialIcon {
            id: actionIcon
            anchors.centerIn: parent
            text: action.busy ? "hourglass_top" : action.iconName
            color: Colours.palette.m3primary
            fontStyle: Tokens.font.icon.medium
        }

        MouseArea {
            id: actionMouse
            anchors.fill: parent
            enabled: !action.busy
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: action.triggered()
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

    component ProviderRow: StyledRect {
        id: providerRow
        required property var providerData

        readonly property color accent: providerData.health === "critical"
            ? Colours.palette.m3error
            : providerData.health === "warning"
                ? Colours.palette.m3tertiary
                : Colours.palette.m3primary

        implicitHeight: providerContent.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer
        border.width: 1
        border.color: Qt.alpha(accent, 0.16)

        RowLayout {
            id: providerContent
            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: providerRow.providerData.health === "critical" ? "error" : "neurology"
                color: providerRow.accent
                fill: providerRow.providerData.health === "critical" ? 1 : 0
                fontStyle: Tokens.font.icon.medium
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                StyledText {
                    Layout.fillWidth: true
                    text: providerRow.providerData.name || providerRow.providerData.id || qsTr("Provider")
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.title.small
                }

                StyledText {
                    Layout.fillWidth: true
                    text: {
                        const parts = [];
                        if (providerRow.providerData.plan)
                            parts.push(providerRow.providerData.plan);
                        if (providerRow.providerData.statusLabel)
                            parts.push(providerRow.providerData.statusLabel);
                        if (providerRow.providerData.error)
                            parts.push(providerRow.providerData.error);
                        return parts.join(" · ") || qsTr("usage available in Settings");
                    }
                    elide: Text.ElideRight
                    color: providerRow.providerData.error ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                }
            }

            StyledText {
                text: providerRow.providerData.maxUsedPercent === null || providerRow.providerData.maxUsedPercent === undefined
                    ? "--"
                    : `${Math.round(providerRow.providerData.maxUsedPercent)}%`
                color: providerRow.accent
                font: Tokens.font.headline.small
            }
        }
    }
}
