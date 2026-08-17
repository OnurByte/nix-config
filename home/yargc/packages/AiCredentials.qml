pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.modules.nexus.common

PageBase {
    id: root

    property var items: []
    property string message: ""

    title: qsTr("API keys")
    isSubPage: true

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status
        command: ["@vesperControl@", "ai-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.items = JSON.parse(text).credentials || [];
                } catch (e) {
                    root.items = [];
                    root.message = qsTr("Could not read key status");
                }
            }
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.medium

        StyledText {
            Layout.fillWidth: true
            text: qsTr("Keys never go into Nix, shell history or process arguments. Vesper stores them in Secret Service and can inject a provider key only into the child process that needs it.")
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }

        Repeater {
            model: root.items

            delegate: ConnectedRect {
                id: credential

                required property var modelData
                property string action: ""
                property string errorText: ""

                Layout.fillWidth: true
                implicitHeight: form.implicitHeight + Tokens.padding.large * 2

                function save() {
                    if (!keyField.text.trim())
                        return;
                    action = "set";
                    errorText = "";
                    change.command = ["@vesperControl@", "credential", "set", modelData.id];
                    change.running = true;
                }

                function clearKey() {
                    action = "clear";
                    errorText = "";
                    change.command = ["@vesperControl@", "credential", "clear", modelData.id];
                    change.running = true;
                }

                Process {
                    id: change
                    stdinEnabled: true
                    stderr: StdioCollector {
                        id: stderrCollector
                    }
                    onStarted: {
                        if (credential.action === "set")
                            write(keyField.text + "\n");
                    }
                    onExited: (code, status) => {
                        if (code === 0) {
                            keyField.text = "";
                            root.message = credential.action === "set" ? qsTr("Key saved") : qsTr("Key cleared");
                            root.refresh();
                        } else {
                            credential.errorText = stderrCollector.text.trim() || qsTr("Secret Service operation failed");
                        }
                    }
                }

                ColumnLayout {
                    id: form
                    anchors.fill: parent
                    anchors.margins: Tokens.padding.large
                    spacing: Tokens.spacing.small

                    RowLayout {
                        Layout.fillWidth: true

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0

                            StyledText {
                                text: credential.modelData.name
                                font: Tokens.font.body.medium
                            }

                            StyledText {
                                text: credential.modelData.configured ? qsTr("configured · %1").arg(credential.modelData.env) : qsTr("not configured · %1").arg(credential.modelData.env)
                                color: credential.modelData.configured ? Colours.palette.m3primary : Colours.palette.m3outline
                                font: Tokens.font.label.small
                            }
                        }

                        MaterialIcon {
                            text: credential.modelData.configured ? "check_circle" : "key_off"
                            color: credential.modelData.configured ? Colours.palette.m3primary : Colours.palette.m3outline
                            fill: credential.modelData.configured ? 1 : 0
                            fontStyle: Tokens.font.icon.medium
                        }
                    }

                    StyledTextField {
                        id: keyField
                        Layout.fillWidth: true
                        placeholderText: qsTr("API key")
                        leadingIcon: "key"
                        echoMode: TextInput.Password
                        inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhSensitiveData
                        onAccepted: credential.save()
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Tokens.spacing.small

                        StyledText {
                            Layout.fillWidth: true
                            visible: credential.errorText
                            text: credential.errorText
                            color: Colours.palette.m3error
                            font: Tokens.font.label.small
                            wrapMode: Text.WordWrap
                        }

                        IconTextButton {
                            visible: credential.modelData.configured
                            isRound: true
                            icon: "delete"
                            text: qsTr("Clear")
                            onClicked: credential.clearKey()
                        }

                        IconTextButton {
                            isRound: true
                            icon: "save"
                            text: qsTr("Save")
                            disabled: !keyField.text.trim() || change.running
                            onClicked: credential.save()
                        }
                    }
                }
            }
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.message
            text: root.message
            color: Colours.palette.m3primary
            font: Tokens.font.body.small
        }
    }
}
