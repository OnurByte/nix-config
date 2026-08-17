pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

Item {
    id: root

    property var payload: ({ count: 0, agents: [], class: "idle" })
    property string loadError: ""

    readonly property var agents: payload.agents || []
    readonly property color accent: (payload.count || 0) > 0
        ? Colours.palette.m3secondary
        : Colours.palette.m3outline

    implicitWidth: 470
    implicitHeight: 430

    function applyPayload(text) {
        try {
            root.payload = JSON.parse(text);
            root.loadError = "";
        } catch (e) {
            root.loadError = qsTr("Agent monitor returned invalid data");
        }
    }

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function focusPid(pid) {
        if (!pid || focusAgent.running)
            return;
        focusAgent.command = ["@agentCockpit@", "focus", String(pid)];
        focusAgent.running = true;
    }

    function formatAge(seconds) {
        const value = Number(seconds || 0);
        if (value < 60)
            return `${value}s`;
        if (value < 3600)
            return `${Math.floor(value / 60)}m`;
        if (value < 86400)
            return `${Math.floor(value / 3600)}h ${Math.floor((value % 3600) / 60)}m`;
        return `${Math.floor(value / 86400)}d ${Math.floor((value % 86400) / 3600)}h`;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 2000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Process {
        id: status
        command: ["@agentCockpit@", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.applyPayload(text)
        }
    }

    Process {
        id: focusAgent
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Tokens.spacing.medium

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: "terminal"
                color: root.accent
                fill: 1
                fontStyle: Tokens.font.icon.large
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                StyledText {
                    text: qsTr("Agent Cockpit")
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.title.large
                }

                StyledText {
                    Layout.fillWidth: true
                    text: root.loadError || ((root.payload.count || 0) === 1
                        ? qsTr("1 active coding agent")
                        : qsTr(`${root.payload.count || 0} active coding agents`))
                    color: root.loadError ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.small
                    elide: Text.ElideRight
                }
            }

            MaterialIcon {
                text: "refresh"
                color: Colours.palette.m3primary
                fontStyle: Tokens.font.icon.medium

                MouseArea {
                    anchors.fill: parent
                    anchors.margins: -Tokens.padding.medium
                    cursorShape: Qt.PointingHandCursor
                    onClicked: root.refresh()
                }
            }
        }

        StyledRect {
            Layout.fillWidth: true
            implicitHeight: summary.implicitHeight + Tokens.padding.medium * 2
            radius: Tokens.rounding.large
            color: Colours.tPalette.m3surfaceContainer

            RowLayout {
                id: summary
                anchors.fill: parent
                anchors.margins: Tokens.padding.medium

                StyledText {
                    Layout.fillWidth: true
                    text: qsTr("Live sessions")
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.medium
                }

                StyledText {
                    text: `${root.payload.count || 0}`
                    color: root.accent
                    font: Tokens.font.title.large
                }
            }
        }

        Flickable {
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            contentWidth: width
            contentHeight: agentColumn.implicitHeight
            flickableDirection: Flickable.VerticalFlick
            boundsBehavior: Flickable.StopAtBounds

            ColumnLayout {
                id: agentColumn
                width: parent.width
                spacing: Tokens.spacing.small

                StyledText {
                    Layout.fillWidth: true
                    visible: root.agents.length === 0
                    text: root.loadError || qsTr("No active coding agents")
                    horizontalAlignment: Text.AlignHCenter
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.body.medium
                }

                Repeater {
                    model: root.agents

                    delegate: StyledRect {
                        id: card
                        required property var modelData

                        Layout.fillWidth: true
                        implicitHeight: cardContent.implicitHeight + Tokens.padding.medium * 2
                        radius: Tokens.rounding.large
                        color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, cardMouse.containsMouse ? 0.78 : 0.5)
                        border.width: 1
                        border.color: Qt.alpha(card.modelData.dirty ? Colours.palette.m3tertiary : root.accent, 0.18)

                        RowLayout {
                            id: cardContent
                            anchors.fill: parent
                            anchors.margins: Tokens.padding.medium
                            spacing: Tokens.spacing.medium

                            MaterialIcon {
                                text: card.modelData.dirty ? "edit_note" : "code"
                                color: card.modelData.dirty ? Colours.palette.m3tertiary : root.accent
                                fontStyle: Tokens.font.icon.medium
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0

                                RowLayout {
                                    Layout.fillWidth: true

                                    StyledText {
                                        text: card.modelData.agent || qsTr("Agent")
                                        color: Colours.palette.m3onSurface
                                        font: Tokens.font.title.medium
                                    }

                                    StyledText {
                                        Layout.fillWidth: true
                                        text: card.modelData.project || "unknown"
                                        horizontalAlignment: Text.AlignRight
                                        elide: Text.ElideLeft
                                        color: Colours.palette.m3onSurfaceVariant
                                        font: Tokens.font.body.medium
                                    }
                                }

                                StyledText {
                                    Layout.fillWidth: true
                                    text: {
                                        const parts = [];
                                        if (card.modelData.branch)
                                            parts.push(card.modelData.branch);
                                        parts.push(root.formatAge(card.modelData.elapsedSeconds));
                                        parts.push(`pid ${card.modelData.pid}`);
                                        return parts.join(" · ");
                                    }
                                    elide: Text.ElideRight
                                    color: card.modelData.dirty ? Colours.palette.m3tertiary : Colours.palette.m3onSurfaceVariant
                                    font: Tokens.font.body.small
                                }

                                StyledText {
                                    Layout.fillWidth: true
                                    visible: !!card.modelData.cwd
                                    text: card.modelData.cwd || ""
                                    elide: Text.ElideMiddle
                                    color: Colours.palette.m3outline
                                    font: Tokens.font.body.small
                                }
                            }

                            MaterialIcon {
                                text: "center_focus_strong"
                                color: Colours.palette.m3primary
                                fontStyle: Tokens.font.icon.small
                            }
                        }

                        MouseArea {
                            id: cardMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.focusPid(card.modelData.pid)
                        }
                    }
                }
            }
        }
    }
}
