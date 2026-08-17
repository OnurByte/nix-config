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
    property var net: ({ airplane: false, wifi: false, wwan: false, bluetooth: false, connection: "", zapret: false, proxy: false })
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
        command: ["@vesperNetwork@", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.net = JSON.parse(text);
                    root.errorText = "";
                } catch (e) {
                    root.errorText = qsTr("Could not read Vesper network state");
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
        id: qr
        stdout: StdioCollector {
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
        text: qsTr("Vesper connectivity")
    }

    ToggleRow {
        first: true
        text: qsTr("Airplane mode")
        subtext: qsTr("turn radios off together and restore their previous states later")
        checked: root.net.airplane
        disabled: airplane.running
        onToggled: {
            airplane.command = ["@vesperNetwork@", "airplane", checked ? "on" : "off"];
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
            qr.command = ["@vesperNetwork@", "wifi-qr"];
            qr.running = true;
        }
    }

    NavRow {
        last: true
        icon: "language"
        text: qsTr("Proxy")
        subtext: root.net.proxy ? qsTr("configured for new processes") : qsTr("off")
        onClicked: root.nState.openSubPage(7)
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

    NavRow {
        icon: "shield"
        text: qsTr("Zapret2")
        subtext: root.net.zapret ? qsTr("active · adaptive TLS bypass · tune strategy") : qsTr("inactive · open tuning")
        onClicked: root.nState.openSubPage(8)
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
