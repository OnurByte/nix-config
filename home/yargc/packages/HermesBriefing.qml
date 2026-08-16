import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

StyledRect {
    id: root

    property int unreadCount: 0
    property int highCount: 0
    property string briefingState: "idle"
    property string details: "Hermes · no briefings yet"

    readonly property color accent: highCount > 0
        ? Colours.palette.m3error
        : unreadCount > 0
            ? Colours.palette.m3primary
            : Colours.palette.m3outline

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.52 : 0.28)
    border.width: 1
    border.color: Qt.alpha(root.accent, mouse.containsMouse ? 0.34 : 0.18)

    function refresh(): void {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 15000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Process {
        id: status
        command: ["@hermesRuntime@", "status", "--json"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const value = JSON.parse(text);
                    root.unreadCount = Number(value.unread) || 0;
                    root.highCount = Number(value.high) || 0;
                    root.briefingState = value.class || "idle";
                    root.details = value.tooltip || "Hermes briefing status unavailable";
                } catch (e) {
                    root.unreadCount = 0;
                    root.highCount = 0;
                    root.briefingState = "idle";
                    root.details = "Hermes briefing data unavailable";
                }
            }
        }
    }

    Process {
        id: popup
        command: ["@hermesRuntime@", "inbox"]
    }

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: root.highCount > 0 ? "notification_important" : "auto_awesome"
            color: root.accent
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: `${root.unreadCount}`
            color: root.accent
            font: Tokens.font.body.builders.small.scale(0.82).build()
        }
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor

        onClicked: event => {
            if (event.button === Qt.RightButton)
                root.refresh();
            else if (!popup.running)
                popup.running = true;
        }
    }
}
