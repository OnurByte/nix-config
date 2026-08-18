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

    property var providers: []
    property var credentials: []
    property string message: ""
    property bool loading: false

    title: qsTr("Credentials")
    isSubPage: true

    function refresh() {
        if (!providerStatus.running)
            providerStatus.running = true;
        if (!credentialStatus.running)
            credentialStatus.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: providerStatus
        command: ["@vesperControl@", "ai-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.providers = JSON.parse(text).credentials || [];
                } catch (e) {
                    root.providers = [];
                    root.message = qsTr("Could not read provider status");
                }
            }
        }
    }

    Process {
        id: credentialStatus
        command: ["@vesperControl@", "credential", "list"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.credentials = JSON.parse(text).credentials || [];
                } catch (e) {
                    root.credentials = [];
                    root.message = qsTr("Could not read credential aliases");
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
            text: qsTr("API keys stay in Secret Service. Vesper stores only non-secret aliases and provider metadata, then injects a selected key into the child process that needs it.")
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.credentials.length > 0
            text: qsTr("Stored aliases")
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.title.small
        }

        Repeater {
            model: root.credentials

            delegate: ConnectedRect {
                id: storedCredential

                required property var modelData
                property string errorText: ""

                Layout.fillWidth: true
                implicitHeight: storedContent.implicitHeight + Tokens.padding.large * 2

                Process {
                    id: removeCredential
                    command: ["@vesperControl@", "credential", "clear", storedCredential.modelData.id]
                    stderr: StdioCollector {
                        id: removeError
                    }
                    onExited: (code, status) => {
                        if (code === 0) {
                            root.message = qsTr("Credential removed");
                            root.refresh();
                        } else {
                            storedCredential.errorText = removeError.text.trim() || qsTr("Secret Service operation failed");
                        }
                    }
                }

                RowLayout {
                    id: storedContent
                    anchors.fill: parent
                    anchors.margins: Tokens.padding.large
                    spacing: Tokens.spacing.medium

                    MaterialIcon {
                        text: storedCredential.modelData.configured ? "key" : "key_off"
                        color: storedCredential.modelData.configured ? Colours.palette.m3primary : Colours.palette.m3error
                        fill: storedCredential.modelData.configured ? 1 : 0
                        fontStyle: Tokens.font.icon.medium
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        StyledText {
                            Layout.fillWidth: true
                            text: storedCredential.modelData.id
                            color: Colours.palette.m3onSurface
                            font: Tokens.font.body.medium
                            elide: Text.ElideRight
                        }

                        StyledText {
                            Layout.fillWidth: true
                            text: qsTr("%1 · %2 · Managed by Vesper").arg(storedCredential.modelData.providerName || storedCredential.modelData.provider).arg(storedCredential.modelData.env || "")
                            color: Colours.palette.m3onSurfaceVariant
                            font: Tokens.font.label.small
                            elide: Text.ElideRight
                        }

                        StyledText {
                            Layout.fillWidth: true
                            visible: storedCredential.errorText
                            text: storedCredential.errorText
                            color: Colours.palette.m3error
                            font: Tokens.font.label.small
                            wrapMode: Text.WordWrap
                        }
                    }

                    IconTextButton {
                        isRound: true
                        icon: "delete"
                        text: qsTr("Remove")
                        disabled: removeCredential.running
                        onClicked: removeCredential.running = true
                    }
                }
            }
        }

        StyledText {
            Layout.fillWidth: true
            text: qsTr("Providers")
            color: Colours.palette.m3onSurfaceVariant
            font: Tokens.font.title.small
        }

        Repeater {
            model: root.providers

            delegate: ConnectedRect {
                id: providerCard

                required property var modelData
                property string errorText: ""
                property string action: ""

                Layout.fillWidth: true
                implicitHeight: providerForm.implicitHeight + Tokens.padding.large * 2

                function save() {
                    if (!keyField.text.trim())
                        return;
                    action = "set";
                    errorText = "";
                    const alias = aliasField.text.trim();
                    change.command = alias.length > 0
                        ? ["@vesperControl@", "credential", "set", modelData.id, alias]
                        : ["@vesperControl@", "credential", "set", modelData.id];
                    change.running = true;
                }

                function clearDefault() {
                    action = "clear";
                    errorText = "";
                    change.command = ["@vesperControl@", "credential", "clear", modelData.id];
                    change.running = true;
                }

                Process {
                    id: change
                    stdinEnabled: true
                    stderr: StdioCollector {
                        id: changeError
                    }
                    onStarted: {
                        if (providerCard.action === "set")
                            write(keyField.text + "\n");
                    }
                    onExited: (code, status) => {
                        if (code === 0) {
                            keyField.text = "";
                            aliasField.text = "";
                            root.message = providerCard.action === "set" ? qsTr("Credential saved") : qsTr("Default credential cleared");
                            root.refresh();
                        } else {
                            providerCard.errorText = changeError.text.trim() || qsTr("Secret Service operation failed");
                        }
                    }
                }

                ColumnLayout {
                    id: providerForm
                    anchors.fill: parent
                    anchors.margins: Tokens.padding.large
                    spacing: Tokens.spacing.small

                    RowLayout {
                        Layout.fillWidth: true

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0

                            StyledText {
                                text: providerCard.modelData.name
                                font: Tokens.font.body.medium
                            }

                            StyledText {
                                text: providerCard.modelData.configured
                                    ? qsTr("default configured · %1").arg(providerCard.modelData.env)
                                    : qsTr("no default key · %1").arg(providerCard.modelData.env)
                                color: providerCard.modelData.configured ? Colours.palette.m3primary : Colours.palette.m3outline
                                font: Tokens.font.label.small
                            }
                        }

                        MaterialIcon {
                            text: providerCard.modelData.configured ? "check_circle" : "key_off"
                            color: providerCard.modelData.configured ? Colours.palette.m3primary : Colours.palette.m3outline
                            fill: providerCard.modelData.configured ? 1 : 0
                            fontStyle: Tokens.font.icon.medium
                        }
                    }

                    StyledTextField {
                        id: aliasField
                        Layout.fillWidth: true
                        placeholderText: qsTr("Alias (optional, e.g. openrouter-main)")
                        leadingIcon: "label"
                        inputMethodHints: Qt.ImhNoPredictiveText
                    }

                    StyledTextField {
                        id: keyField
                        Layout.fillWidth: true
                        placeholderText: qsTr("API key")
                        leadingIcon: "key"
                        echoMode: TextInput.Password
                        inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhSensitiveData
                        onAccepted: providerCard.save()
                    }

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Tokens.spacing.small

                        StyledText {
                            Layout.fillWidth: true
                            visible: providerCard.errorText
                            text: providerCard.errorText
                            color: Colours.palette.m3error
                            font: Tokens.font.label.small
                            wrapMode: Text.WordWrap
                        }

                        IconTextButton {
                            visible: providerCard.modelData.configured
                            isRound: true
                            icon: "delete"
                            text: qsTr("Clear default")
                            disabled: change.running
                            onClicked: providerCard.clearDefault()
                        }

                        IconTextButton {
                            isRound: true
                            icon: "save"
                            text: qsTr("Save")
                            disabled: !keyField.text.trim() || change.running
                            onClicked: providerCard.save()
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
            wrapMode: Text.WordWrap
        }
    }
}
