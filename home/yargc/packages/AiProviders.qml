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
    property var state: ({ defaultProvider: "openai", defaultModel: "", fallbacks: "", providers: [] })
    property string message: ""
    title: qsTr("Providers")
    isSubPage: true

    function refresh(test) {
        if (status.running) return;
        status.command = ["@vesperControl@", "provider", "status"].concat(test ? ["test"] : []);
        status.running = true;
    }
    function run(command) {
        if (change.running) return;
        root.message = "";
        change.command = command;
        change.running = true;
    }
    Component.onCompleted: refresh(false)

    Process {
        id: status
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.state = JSON.parse(text);
                    if (!defaultProvider.activeFocus) defaultProvider.text = root.state.defaultProvider || "openai";
                    if (!defaultModel.activeFocus) defaultModel.text = root.state.defaultModel || "";
                    if (!fallbacks.activeFocus) fallbacks.text = root.state.fallbacks || "";
                } catch (e) { root.message = qsTr("Could not read provider registry"); }
            }
        }
    }
    Process {
        id: change
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => { root.message = code === 0 ? qsTr("Provider settings updated") : changeError.text.trim(); root.refresh(false); }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Routing") }
        StyledTextField { id: defaultProvider; Layout.fillWidth: true; leadingIcon: "route"; placeholderText: "openai"; supportingText: qsTr("default provider id") }
        StyledTextField { id: defaultModel; Layout.fillWidth: true; leadingIcon: "model_training"; placeholderText: qsTr("default model (optional)"); inputMethodHints: Qt.ImhNoPredictiveText }
        StyledTextField { id: fallbacks; Layout.fillWidth: true; leadingIcon: "alt_route"; placeholderText: "openrouter,xai"; supportingText: qsTr("ordered comma-separated fallback provider ids") }
        RowLayout {
            Layout.fillWidth: true; Item { Layout.fillWidth: true }
            IconTextButton { isRound: true; icon: "save"; text: qsTr("Save routing"); disabled: change.running; onClicked: root.run(["@vesperControl@", "provider", "routing", defaultProvider.text.trim(), defaultModel.text.trim(), fallbacks.text.trim()]) }
        }

        SectionHeader { text: qsTr("Custom OpenAI-compatible endpoint") }
        StyledTextField { id: customId; Layout.fillWidth: true; leadingIcon: "badge"; placeholderText: "my-provider" }
        StyledTextField { id: customName; Layout.fillWidth: true; leadingIcon: "label"; placeholderText: qsTr("Display name") }
        StyledTextField { id: customUrl; Layout.fillWidth: true; leadingIcon: "link"; placeholderText: "https://host.example/v1"; supportingText: qsTr("HTTPS required except localhost") }
        StyledTextField { id: customCredential; Layout.fillWidth: true; leadingIcon: "key"; placeholderText: qsTr("Vesper credential alias") }
        RowLayout {
            Layout.fillWidth: true; Item { Layout.fillWidth: true }
            IconTextButton { isRound: true; icon: "add"; text: qsTr("Add provider"); disabled: change.running || !customId.text.trim() || !customUrl.text.trim() || !customCredential.text.trim(); onClicked: root.run(["@vesperControl@", "provider", "add", customId.text.trim(), customName.text.trim() || customId.text.trim(), customUrl.text.trim(), customCredential.text.trim()]) }
        }

        SectionHeader { text: qsTr("Registry") }
        RowButton { icon: "network_check"; text: qsTr("Test endpoint reachability"); subtext: qsTr("transport only; auth/quota stay unknown unless an adapter can prove them"); disabled: status.running; onClicked: root.refresh(true) }
        Repeater {
            model: root.state.providers || []
            delegate: InfoRow {
                required property var modelData
                icon: modelData.endpointReachable === true ? "check_circle" : (modelData.endpointReachable === false ? "warning" : "cloud")
                label: modelData.name
                subtext: qsTr("%1 · credential %2 · budget %3¢ · %4").arg(modelData.baseUrl).arg(modelData.credential).arg(modelData.budgetCents).arg(modelData.custom ? qsTr("custom") : qsTr("built-in"))
                value: modelData.enabled ? (modelData.endpointReachable === null ? qsTr("enabled") : (modelData.endpointReachable ? qsTr("reachable") : qsTr("unreachable"))) : qsTr("disabled")
            }
        }
        StyledText { Layout.fillWidth: true; visible: root.message; text: root.message; color: root.message.toLowerCase().includes("updated") ? Colours.palette.m3primary : Colours.palette.m3error; font: Tokens.font.body.small; wrapMode: Text.WordWrap }
    }
}
