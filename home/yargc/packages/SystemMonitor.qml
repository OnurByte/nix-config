import QtQuick
import QtQuick.Layouts
import Caelestia.Config
import Caelestia.Services
import qs.components

StyledRect {
    id: root

    required property ScreenState screenState

    readonly property real cpuUsage: isNaN(Cpu.percentage) ? 0 : Cpu.percentage
    readonly property real memoryUsage: isNaN(Memory.percentage) ? 0 : Memory.percentage
    readonly property real gpuUsage: Gpu.type === Gpu.None || isNaN(Gpu.percentage) ? 0 : Gpu.percentage
    readonly property real pressure: Math.max(cpuUsage, memoryUsage, gpuUsage)
    readonly property int performanceTab: (Config.dashboard.showDashboard ? 1 : 0) + (Config.dashboard.showMedia ? 1 : 0)
    readonly property color accent: pressure >= 0.85 ? Colours.palette.m3error : Colours.palette.m3primary

    implicitWidth: Tokens.sizes.bar.innerWidth
    implicitHeight: layout.implicitHeight + Tokens.padding.small * 2
    radius: Tokens.rounding.full
    color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, mouse.containsMouse ? 0.52 : 0.28)
    border.width: 1
    border.color: Qt.alpha(root.accent, mouse.containsMouse ? 0.36 : 0.18)
    visible: Config.dashboard.enabled && Config.dashboard.showPerformance

    ServiceRef {
        service: Cpu
    }

    ServiceRef {
        service: Memory
    }

    ServiceRef {
        service: Gpu
    }

    ColumnLayout {
        id: layout

        anchors.centerIn: parent
        spacing: 0

        MaterialIcon {
            Layout.alignment: Qt.AlignHCenter
            text: "speed"
            color: root.accent
            fill: root.pressure >= 0.85 ? 1 : 0
            font.pointSize: 14
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: `C${Math.round(root.cpuUsage * 100)}`
            color: root.accent
            font: Tokens.font.body.builders.small.scale(0.68).build()
        }

        StyledText {
            Layout.alignment: Qt.AlignHCenter
            text: `R${Math.round(root.memoryUsage * 100)}`
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.body.builders.small.scale(0.68).build()
        }
    }

    MouseArea {
        id: mouse

        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor

        onClicked: {
            root.screenState.dashboardTab = root.performanceTab;
            root.screenState.dashboard = true;
        }
    }
}
