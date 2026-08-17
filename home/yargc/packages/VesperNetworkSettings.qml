pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var nState
    property var net: ({ airplane: false, wifi: false, bluetooth: false, connection: "", zapret: false, proxy: false })
    property string qrPath: ""
    property string errorText: ""

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 5000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Process {
        id: status
        command: ["@vesperControl@", "network", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.net = JSON.parse(text);
                    root.errorText = "";
                } catch (e) {
                    root.errorText = qsTr("Could not read network state");
                }
            }
        }
    }

    Process {
        id: airplane
        stderr: StdioCollector { id: airplaneError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = airplaneError.text.trim();
            root.refresh();
        }
    }

    Process {
        id: dpi
        stderr: StdioCollector { id: dpiError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = dpiError.text.trim() || qsTr("Could not change DPI state");
            root.refresh();
        }
    }

    Process {
        id: qr
        stdout: StdioCollector {
            id: qrOutput
            onStreamFinished: {
                if (text.trim())
                    root.qrPath = text.trim();
            }
        }
        stderr: StdioCollector { id: qrError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = qrError.text.trim();
        }
    }

    SectionHeader {
        text: qsTr("Connectivity")
    }

    ToggleRow {
        first: true
        text: qsTr("Airplane mode")
        subtext: qsTr("turn Wi-Fi and Bluetooth radios off together")
        checked: root.net.airplane
        disabled: airplane.running
        onToggled: {
            airplane.command = ["@vesperControl@", "network", "airplane", checked ? "on" : "off"];
            airplane.running = true;
        }
    }

    RowButton {
        icon: "qr_code_2"
        text: qsTr("Share current Wi-Fi")
        subtext: root.net.connection ? root.net.connection : qsTr("connect to Wi-Fi first")
        disabled: !root.net.connection || qr.running
        onClicked: {
            root.qrPath = "";
            root.errorText = "";
            qr.command = ["@vesperControl@", "network", "wifi-qr"];
            qr.running = true;
        }
    }

    NavRow {
        last: true
        icon: "language"
        text: qsTr("Proxy")
        subtext: root.net.proxy ? qsTr("configured for new processes") : qsTr("off")
        onClicked: root.nState.openSubPageRoute("proxy")
    }

    StyledRect {
        Layout.alignment: Qt.AlignHCenter
        Layout.topMargin: root.qrPath ? Tokens.spacing.medium : 0
        visible: root.qrPath
        implicitWidth: 260
        implicitHeight: 260
        radius: Tokens.rounding.large
        color: Colours.palette.m3surfaceContainerHighest

        Image {
            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            source: root.qrPath ? `file://${root.qrPath}` : ""
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            cache: false
        }
    }

    SectionHeader {
        text: qsTr("DPI")
    }

    ToggleRow {
        text: qsTr("Zapret2 DPI bypass")
        subtext: qsTr("current boot · adaptive host detection · TLS ClientHello · TCP 443")
        checked: root.net.zapret
        disabled: dpi.running
        onToggled: {
            root.errorText = "";
            dpi.command = ["@vesperControl@", "network", "dpi", checked ? "on" : "off"];
            dpi.running = true;
        }
    }

    StyledText {
        Layout.fillWidth: true
        Layout.leftMargin: Tokens.padding.largeIncreased
        Layout.rightMargin: Tokens.padding.largeIncreased
        visible: root.errorText
        text: root.errorText
        color: Colours.palette.m3error
        font: Tokens.font.label.small
        wrapMode: Text.WordWrap
    }
}
