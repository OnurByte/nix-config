import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

StyledRect {
    id: root

    property int activeCount: 0
    property string cockpitState: "idle"
    property string details: "No active coding agents"

    readonly property color accent: cockpitState === "active"
        ? Colours.palette.m3primary
        : Colours.palette.m3outline

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.8 : 0.35)

    function refresh(): void {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 5000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Process {
        id: status
        command: ["@agentCockpit@", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const value = JSON.parse(text);
                    root.activeCount = Number(value.count) || 0;
                    root.cockpitState = value.class || "idle";
                    root.details = value.tooltip || "Agent status unavailable";
                } catch (e) {
                    root.activeCount = 0;
                    root.cockpitState = "idle";
                    root.details = "Agent cockpit data unavailable";
                }
            }
        }
    }

    Process {
        id: popup
        command: ["@agentCockpit@", "popup"]
    }

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: "terminal"
            color: root.accent
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: `${root.activeCount}`
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
