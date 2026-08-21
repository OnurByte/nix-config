import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services

StyledRect {
    id: root

    required property ScreenState screenState

    property int percentage: -1
    property string provider: ""
    property string usageState: "stale"
    property string details: "AI usage is loading"

    readonly property int aiTab: (Config.dashboard.showDashboard ? 1 : 0)
        + (Config.dashboard.showMedia ? 1 : 0)
        + (Config.dashboard.showPerformance ? 1 : 0)
    readonly property color accent: usageState === "critical"
        ? Colours.palette.m3error
        : usageState === "warning"
            ? Colours.palette.m3tertiary
            : Colours.palette.m3primary

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.8 : 0.35)

    function refresh(force) {
        if (force) {
            if (!forceRefresh.running)
                forceRefresh.running = true;
        } else if (!usage.running) {
            usage.running = true;
        }
    }

    function applyPayload(text) {
        try {
            const value = JSON.parse(text);
            const summary = value.summary || {};
            root.percentage = Number.isFinite(Number(summary.maxUsedPercent))
                ? Math.round(Number(summary.maxUsedPercent))
                : -1;
            root.provider = summary.maxProvider || "";
            root.usageState = value.stale ? "stale" : (summary.class || "ok");
            root.details = root.provider
                ? `${root.provider} · ${root.percentage}% used`
                : `${summary.providerCount || 0} AI providers`;
        } catch (e) {
            root.percentage = -1;
            root.provider = "";
            root.usageState = "stale";
            root.details = "AI Hub data unavailable";
        }
    }

    Component.onCompleted: refresh(false)

    Timer {
        interval: 30000
        repeat: true
        running: true
        onTriggered: root.refresh(false)
    }

    Process {
        id: usage
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

    ColumnLayout {
        id: layout
        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: root.usageState === "critical" ? "robot_2" : "smart_toy"
            color: root.accent
            fill: root.usageState === "critical" ? 1 : 0
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
            if (event.button === Qt.RightButton) {
                root.refresh(true);
            } else {
                root.screenState.dashboardTab = root.aiTab;
                root.screenState.dashboard = true;
            }
        }
    }
}
