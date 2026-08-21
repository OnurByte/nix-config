import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services

StyledRect {
    id: root

    required property ScreenState screenState

    property int count: 0
    property string activityState: "idle"
    property string details: "No active coding agents"

    readonly property int aiTab: (Config.dashboard.showDashboard ? 1 : 0)
        + (Config.dashboard.showMedia ? 1 : 0)
        + (Config.dashboard.showPerformance ? 1 : 0)
    readonly property color accent: activityState === "active"
        ? Colours.palette.m3secondary
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
                    root.count = Number(value.count) || 0;
                    root.activityState = value.class || "idle";
                    root.details = value.tooltip || "Agent status unavailable";
                } catch (e) {
                    root.count = 0;
                    root.activityState = "idle";
                    root.details = "Agent cockpit data unavailable";
                }
            }
        }
    }

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: root.count > 0 ? "terminal" : "code"
            color: root.accent
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: `${root.count}`
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
            if (event.button === Qt.RightButton) {
                root.refresh();
            } else {
                root.screenState.dashboardTab = root.aiTab;
                root.screenState.dashboard = true;
            }
        }
    }
}
