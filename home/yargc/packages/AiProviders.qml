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
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Could not read provider registry");
                }
            }
        }
    }

    Process {
        id: change
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Provider settings updated") : changeError.text.trim();
            root.refresh(false);
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Routing") }
        StyledTextField {
            id: defaultProvider
            Layout.fillWidth: true
            leadingIcon: "route"
            placeholderText: "openai"
            supportingText: qsTr("default provider id; provider must exist and be enabled")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        StyledTextField {
            id: defaultModel
            Layout.fillWidth: true
            leadingIcon: "model_training"
            placeholderText: qsTr("default model (optional)")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        StyledTextField {
            id: fallbacks
            Layout.fillWidth: true
            leadingIcon: "alt_route"
            placeholderText: "openrouter,xai"
            supportingText: qsTr("ordered comma-separated fallback provider ids")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "save"
                text: qsTr("Save routing")
                disabled: change.running
                onClicked: root.run(["@vesperControl@", "provider", "routing", defaultProvider.text.trim(), defaultModel.text.trim(), fallbacks.text.trim()])
            }
        }

        SectionHeader { text: qsTr("Custom OpenAI-compatible endpoint") }
        StyledTextField {
            id: customId
            Layout.fillWidth: true
            leadingIcon: "badge"
            placeholderText: "my-provider"
            supportingText: qsTr("stable provider id")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        StyledTextField {
            id: customName
            Layout.fillWidth: true
            leadingIcon: "label"
            placeholderText: qsTr("Display name")
        }
        StyledTextField {
            id: customUrl
            Layout.fillWidth: true
            leadingIcon: "link"
            placeholderText: "https://host.example/v1"
            supportingText: qsTr("HTTPS required except localhost")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        StyledTextField {
            id: customCredential
            Layout.fillWidth: true
            leadingIcon: "key"
            placeholderText: qsTr("Vesper credential alias")
            supportingText: qsTr("logical alias only; secret value remains in Secret Service")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "add"
                text: qsTr("Add provider")
                disabled: change.running || !customId.text.trim() || !customUrl.text.trim() || !customCredential.text.trim()
                onClicked: root.run(["@vesperControl@", "provider", "add", customId.text.trim(), customName.text.trim() || customId.text.trim(), customUrl.text.trim(), customCredential.text.trim()])
            }
        }

        SectionHeader { text: qsTr("Registry") }
        RowButton {
            icon: "network_check"
            text: qsTr("Test endpoint transport")
            subtext: qsTr("measures /models reachability and latency only; auth, quota and model entitlement remain unknown unless a real adapter proves them")
            disabled: status.running
            onClicked: root.refresh(true)
        }

        Repeater {
            model: root.state.providers || []

            delegate: ColumnLayout {
                id: providerRow
                required property var modelData
                Layout.fillWidth: true
                spacing: Tokens.spacing.extraSmall / 2

                InfoRow {
                    Layout.fillWidth: true
                    icon: providerRow.modelData.endpointReachable === true
                        ? "check_circle"
                        : (providerRow.modelData.endpointReachable === false ? "warning" : "cloud")
                    label: providerRow.modelData.name
                    subtext: qsTr("%1 · %2")
                        .arg(providerRow.modelData.baseUrl)
                        .arg(providerRow.modelData.custom ? qsTr("custom") : qsTr("built-in"))
                    value: providerRow.modelData.enabled
                        ? (providerRow.modelData.endpointReachable === null
                            ? qsTr("enabled")
                            : (providerRow.modelData.endpointReachable ? qsTr("reachable") : qsTr("unreachable")))
                        : qsTr("disabled")
                }

                InfoRow {
                    Layout.fillWidth: true
                    visible: providerRow.modelData.latencyMs !== null && providerRow.modelData.latencyMs !== undefined
                    icon: "speed"
                    label: qsTr("Transport latency")
                    subtext: qsTr("curl request time to the provider's /models endpoint; not model inference latency")
                    value: qsTr("%1 ms").arg(providerRow.modelData.latencyMs)
                }

                ToggleRow {
                    Layout.fillWidth: true
                    text: qsTr("Enabled")
                    subtext: qsTr("disabled providers cannot be selected as default or fallback")
                    checked: providerRow.modelData.enabled === true
                    disabled: change.running
                    onToggled: root.run(["@vesperControl@", "provider", "set", providerRow.modelData.id, "enabled", checked ? "true" : "false"])
                }

                StyledTextField {
                    id: credentialField
                    Layout.fillWidth: true
                    text: providerRow.modelData.credential || ""
                    leadingIcon: "key"
                    placeholderText: qsTr("credential alias")
                    supportingText: qsTr("Secret Service alias; this field never contains the secret itself")
                    inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
                }

                StyledTextField {
                    id: modelField
                    Layout.fillWidth: true
                    text: providerRow.modelData.model || ""
                    leadingIcon: "model_training"
                    placeholderText: qsTr("preferred model (optional)")
                    supportingText: qsTr("explicit model id; automatic inventory is not fabricated")
                    inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
                }

                StyledTextField {
                    id: budgetField
                    Layout.fillWidth: true
                    text: String(providerRow.modelData.budgetCents || 0)
                    leadingIcon: "payments"
                    placeholderText: "0"
                    supportingText: qsTr("local policy budget in cents; 0 disables the budget marker")
                    inputMethodHints: Qt.ImhDigitsOnly
                }

                StyledTextField {
                    id: baseUrlField
                    visible: providerRow.modelData.custom === true
                    Layout.fillWidth: true
                    text: providerRow.modelData.baseUrl || ""
                    leadingIcon: "link"
                    placeholderText: "https://host.example/v1"
                    supportingText: qsTr("custom providers only")
                    inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
                }

                RowLayout {
                    Layout.fillWidth: true
                    spacing: Tokens.spacing.small
                    Item { Layout.fillWidth: true }
                    IconTextButton {
                        isRound: true
                        icon: "key"
                        text: qsTr("Save credential")
                        disabled: change.running || !credentialField.text.trim()
                        onClicked: root.run(["@vesperControl@", "provider", "set", providerRow.modelData.id, "credential", credentialField.text.trim()])
                    }
                    IconTextButton {
                        isRound: true
                        icon: "model_training"
                        text: qsTr("Save model")
                        disabled: change.running
                        onClicked: root.run(["@vesperControl@", "provider", "set", providerRow.modelData.id, "model", modelField.text.trim()])
                    }
                    IconTextButton {
                        isRound: true
                        icon: "payments"
                        text: qsTr("Save budget")
                        disabled: change.running || !budgetField.text.trim()
                        onClicked: root.run(["@vesperControl@", "provider", "set", providerRow.modelData.id, "budget", budgetField.text.trim()])
                    }
                }

                RowLayout {
                    Layout.fillWidth: true
                    visible: providerRow.modelData.custom === true
                    spacing: Tokens.spacing.small
                    Item { Layout.fillWidth: true }
                    IconTextButton {
                        isRound: true
                        icon: "save"
                        text: qsTr("Save base URL")
                        disabled: change.running || !baseUrlField.text.trim()
                        onClicked: root.run(["@vesperControl@", "provider", "set", providerRow.modelData.id, "base-url", baseUrlField.text.trim()])
                    }
                    IconTextButton {
                        isRound: true
                        icon: "delete"
                        text: qsTr("Remove custom provider")
                        disabled: change.running
                        onClicked: root.run(["@vesperControl@", "provider", "remove", providerRow.modelData.id])
                    }
                }

                InfoRow {
                    Layout.fillWidth: true
                    icon: "info"
                    label: qsTr("Unknown capabilities stay unknown")
                    subtext: qsTr("authValid, quota and model inventory are intentionally not inferred from a transport-only endpoint probe")
                    value: qsTr("honest")
                }
            }
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("updated") ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
