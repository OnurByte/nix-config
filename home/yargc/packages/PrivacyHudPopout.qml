pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

Item {
    id: root

    property var payload: ({ tor: "off", mic: "unknown", camera: "none", clipboard: "off", node: "off", class: "idle", label: "LOC" })
    property string loadError: ""

    readonly property color accent: payload.class === "alert"
        ? Colours.palette.m3error
        : payload.class === "attention"
            ? Colours.palette.m3tertiary
            : payload.class === "private"
                ? Colours.palette.m3primary
                : Colours.palette.m3outline

    implicitWidth: 430
    implicitHeight: 390

    function applyPayload(text) {
        try {
            root.payload = JSON.parse(text);
            root.loadError = "";
        } catch (e) {
            root.loadError = qsTr("Privacy monitor returned invalid data");
        }
    }

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function stateColour(kind, value) {
        if (kind === "camera" && value === "active")
            return Colours.palette.m3error;
        if (kind === "mic" && value === "unmuted")
            return Colours.palette.m3tertiary;
        if (kind === "tor" && value === "on")
            return Colours.palette.m3primary;
        if (kind === "node" && value !== "off")
            return Colours.palette.m3secondary;
        return Colours.palette.m3onSurfaceVariant;
    }

    function statusText() {
        if (root.loadError)
            return root.loadError;
        if (root.payload.camera === "active")
            return qsTr("Camera device is currently in use");
        if (root.payload.mic === "unmuted")
            return qsTr("Microphone is unmuted; this does not mean it is recording");
        if (root.payload.tor === "on")
            return qsTr("System Tor client is active");
        return qsTr("No privacy-sensitive activity detected");
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
        command: ["@privacyHud@", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.applyPayload(text)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: Tokens.spacing.medium

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: root.payload.class === "alert" ? "warning" : "shield_lock"
                color: root.accent
                fill: 1
                fontStyle: Tokens.font.icon.large
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                StyledText {
                    text: qsTr("Privacy HUD")
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.title.large
                }

                StyledText {
                    Layout.fillWidth: true
                    text: root.statusText()
                    color: root.loadError ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                    elide: Text.ElideRight
                    font: Tokens.font.body.small
                }
            }

            StyledRect {
                implicitWidth: badge.implicitWidth + Tokens.padding.medium * 2
                implicitHeight: badge.implicitHeight + Tokens.padding.small * 2
                radius: Tokens.rounding.full
                color: Qt.alpha(root.accent, 0.14)

                StyledText {
                    id: badge
                    anchors.centerIn: parent
                    text: root.payload.label || "LOC"
                    color: root.accent
                    font: Tokens.font.label.large
                }
            }
        }

        StyledRect {
            Layout.fillWidth: true
            Layout.fillHeight: true
            radius: Tokens.rounding.large
            color: Colours.tPalette.m3surfaceContainer

            ColumnLayout {
                anchors.fill: parent
                anchors.margins: Tokens.padding.large
                spacing: Tokens.spacing.small

                Repeater {
                    model: [
                        { key: "tor", icon: "vpn_lock", label: qsTr("Tor"), value: root.payload.tor || "off" },
                        { key: "mic", icon: "mic", label: qsTr("Microphone"), value: root.payload.mic || "unknown" },
                        { key: "camera", icon: "videocam", label: qsTr("Camera"), value: root.payload.camera || "none" },
                        { key: "clipboard", icon: "content_paste", label: qsTr("Clipboard history"), value: root.payload.clipboard || "off" },
                        { key: "node", icon: "currency_bitcoin", label: qsTr("Monero node"), value: root.payload.node || "off" }
                    ]

                    delegate: Item {
                        id: row
                        required property var modelData

                        Layout.fillWidth: true
                        implicitHeight: Math.max(rowIcon.implicitHeight, rowTexts.implicitHeight) + Tokens.padding.small * 2

                        RowLayout {
                            anchors.fill: parent
                            spacing: Tokens.spacing.medium

                            MaterialIcon {
                                id: rowIcon
                                text: row.modelData.icon
                                color: root.stateColour(row.modelData.key, row.modelData.value)
                                fill: row.modelData.key === "camera" && row.modelData.value === "active" ? 1 : 0
                                fontStyle: Tokens.font.icon.medium
                            }

                            ColumnLayout {
                                id: rowTexts
                                Layout.fillWidth: true
                                spacing: 0

                                StyledText {
                                    text: row.modelData.label
                                    color: Colours.palette.m3onSurface
                                    font: Tokens.font.body.medium
                                }

                                StyledText {
                                    text: row.modelData.key === "clipboard"
                                        ? qsTr("Caelestia / Quickshell clipboard service")
                                        : row.modelData.key === "node"
                                            ? qsTr("Local Monero daemon")
                                            : qsTr("Live local state")
                                    color: Colours.palette.m3outline
                                    font: Tokens.font.body.small
                                }
                            }

                            StyledText {
                                text: row.modelData.value
                                color: root.stateColour(row.modelData.key, row.modelData.value)
                                font: Tokens.font.title.medium
                            }
                        }
                    }
                }

                Item { Layout.fillHeight: true }

                StyledText {
                    Layout.fillWidth: true
                    text: qsTr("Local checks only · refreshes every 2 seconds")
                    horizontalAlignment: Text.AlignHCenter
                    color: Colours.palette.m3outline
                    font: Tokens.font.body.small
                }
            }
        }
    }
}
