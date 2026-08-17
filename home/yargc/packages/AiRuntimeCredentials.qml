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

    property var consumers: []
    property string message: ""

    title: qsTr("Runtime Credentials")
    isSubPage: true

    function consumer(name) {
        return root.consumers.find(value => value.consumer === name) || ({});
    }

    function selected(name) {
        const item = root.consumer(name);
        return item.credential || item.defaultCredential || "";
    }

    function save(consumer, credential) {
        if (change.running)
            return;
        const item = root.consumer(consumer);
        const selectedCredential = credential.trim() || item.defaultCredential || "";
        if (!selectedCredential)
            return;
        root.message = "";
        change.command = ["@vesperControl@", "consumer", "set", consumer, selectedCredential];
        change.running = true;
    }

    Component.onCompleted: refresh()

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Process {
        id: status
        command: ["@vesperControl@", "consumer", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.consumers = JSON.parse(text).consumers || [];
                    if (!openCodeCredential.activeFocus)
                        openCodeCredential.text = root.selected("opencode");
                    if (!hermesCredential.activeFocus)
                        hermesCredential.text = root.selected("hermes");
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Could not read runtime credential mappings");
                }
            }
        }
    }

    Process {
        id: change
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Runtime credential mapping updated") : changeError.text.trim();
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader {
            first: true
            text: qsTr("Consumer adapters")
        }

        InfoRow {
            icon: "shield_lock"
            label: qsTr("Injection model")
            subtext: qsTr("only credential aliases are stored here; secret values stay in Secret Service")
            value: qsTr("child-process scoped")
        }

        SectionHeader { text: qsTr("OpenCode") }

        StyledTextField {
            id: openCodeCredential
            Layout.fillWidth: true
            placeholderText: root.consumer("opencode").defaultCredential || "native"
            leadingIcon: "terminal"
            supportingText: qsTr("native keeps OpenCode's own auth; otherwise enter a Vesper credential alias")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "save"
                text: qsTr("Save OpenCode")
                disabled: change.running
                onClicked: root.save("opencode", openCodeCredential.text)
            }
        }

        SectionHeader { text: qsTr("Hermes") }

        StyledTextField {
            id: hermesCredential
            Layout.fillWidth: true
            placeholderText: root.consumer("hermes").defaultCredential || "xai"
            leadingIcon: "travel_explore"
            supportingText: qsTr("API-key credential alias only; Hermes cannot opt out into unmanaged native auth")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "save"
                text: qsTr("Save Hermes")
                disabled: change.running
                onClicked: root.save("hermes", hermesCredential.text)
            }
        }

        SectionHeader { text: qsTr("Fixed adapters") }

        InfoRow {
            icon: "palette"
            label: qsTr("App Icons")
            subtext: qsTr("provider/model live in App Icons; credential selection still resolves to a Vesper API-key alias")
            value: root.selected("icon-curator")
        }

        InfoRow {
            icon: "hub"
            label: qsTr("GitHub MCP")
            subtext: qsTr("GitHub token injected only into the MCP child process")
            value: root.selected("github-mcp")
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("updated") ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
