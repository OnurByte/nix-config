pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.services
import qs.modules.nexus.common

PageBase {
    id: root

    property bool configured: false
    property string message: ""

    title: qsTr("Proxy")
    isSubPage: true

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function saveProxy() {
        if (!proxyField.text.trim() || change.running)
            return;
        change.action = "set";
        change.command = ["@vesperControl@", "proxy", "set"];
        change.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status
        command: ["@vesperControl@", "proxy", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.configured = text.trim() === "configured"
        }
    }

    Process {
        id: change
        property string action: ""
        stdinEnabled: true
        stderr: StdioCollector { id: changeError }
        onStarted: {
            if (action === "set")
                write(proxyField.text + "\n");
        }
        onExited: (code, status) => {
            root.message = code === 0 ? (action === "set" ? qsTr("Proxy saved") : qsTr("Proxy cleared")) : changeError.text.trim();
            if (code === 0 && action === "set")
                proxyField.text = "";
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.large

        InfoRow {
            icon: "language"
            label: qsTr("Process proxy")
            subtext: qsTr("writes ~/.config/environment.d/90-vesper-proxy.conf")
            value: root.configured ? qsTr("configured") : qsTr("off")
        }

        StyledTextField {
            id: proxyField
            Layout.fillWidth: true
            placeholderText: qsTr("http://, https:// or socks5:// URL")
            leadingIcon: "link"
            supportingText: qsTr("applies to newly started processes; restart the session for a clean global handoff")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
            onAccepted: root.saveProxy()
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.small

            StyledText {
                Layout.fillWidth: true
                visible: root.message
                text: root.message
                color: root.message.toLowerCase().includes("saved") || root.message.toLowerCase().includes("cleared") ? Colours.palette.m3primary : Colours.palette.m3error
                font: Tokens.font.label.small
                wrapMode: Text.WordWrap
            }

            IconTextButton {
                visible: root.configured
                isRound: true
                icon: "delete"
                text: qsTr("Clear")
                disabled: change.running
                onClicked: {
                    change.action = "clear";
                    change.command = ["@vesperControl@", "proxy", "clear"];
                    change.running = true;
                }
            }

            IconTextButton {
                id: saveButton
                isRound: true
                icon: "save"
                text: qsTr("Save")
                disabled: !proxyField.text.trim() || change.running
                onClicked: root.saveProxy()
            }
        }
    }
}
