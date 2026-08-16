import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

StyledRect {
    id: root

    property int percentage: -1
    property string usageState: "stale"
    property string details: "AI usage is loading"

    readonly property color accent: usageState === "critical"
        ? Colours.palette.m3error
        : usageState === "warning"
            ? Colours.palette.m3tertiary
            : Colours.palette.m3primary

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.52 : 0.28)
    border.width: 1
    border.color: Qt.alpha(Colours.palette.m3outline, mouse.containsMouse ? 0.28 : 0.16)

    function refresh(): void {
        if (!usage.running)
            usage.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 30000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Process {
        id: usage
        command: ["@codexbarStatus@"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const value = JSON.parse(text);
                    root.percentage = Number.isFinite(Number(value.percentage))
                        ? Math.round(Number(value.percentage))
                        : -1;
                    root.usageState = value.class || "stale";
                    root.details = value.tooltip || "AI provider usage";
                } catch (e) {
                    root.percentage = -1;
                    root.usageState = "stale";
                    root.details = "CodexBar data unavailable";
                }
            }
        }
    }

    Process {
        id: popup
        command: ["@codexbarPopup@"]
    }

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: "smart_toy"
            color: root.accent
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: root.percentage >= 0 ? `${root.percentage}%` : "--"
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
