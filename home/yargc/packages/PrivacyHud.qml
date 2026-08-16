import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

StyledRect {
    id: root

    property string privacyState: "idle"
    property string label: "LOC"
    property string details: "Privacy state unavailable"

    readonly property color accent: privacyState === "alert"
        ? Colours.palette.m3error
        : privacyState === "attention"
            ? Colours.palette.m3tertiary
            : privacyState === "private"
                ? Colours.palette.m3primary
                : Colours.palette.m3outline

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.52 : 0.28)
    border.width: 1
    border.color: Qt.alpha(Colours.palette.m3outline, mouse.containsMouse ? 0.28 : 0.16)

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
        command: ["@privacyHud@", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const value = JSON.parse(text);
                    root.privacyState = value.class || "idle";
                    root.label = value.label || "LOC";
                    root.details = value.tooltip || "Privacy status unavailable";
                } catch (e) {
                    root.privacyState = "idle";
                    root.label = "--";
                    root.details = "Privacy HUD data unavailable";
                }
            }
        }
    }

    Process {
        id: popup
        command: ["@privacyHud@", "popup"]
    }

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: "shield_lock"
            color: root.accent
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: root.label
            color: root.accent
            font: Tokens.font.body.builders.small.scale(0.72).build()
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
