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

    property var iconState: ({
        enabled: true,
        mode: "original",
        tint: "#8aadf4",
        provider: "openai",
        credential: "openai",
        model: "gpt-5",
        counts: {},
        jobs: []
    })
    property string message: ""

    title: qsTr("App Icons")
    isSubPage: true

    readonly property var modes: [
        { id: "original", label: qsTr("Original"), detail: qsTr("use application originals; prepared semantic assets stay cached") },
        { id: "light", label: qsTr("Light"), detail: qsTr("deterministic light-surface rendering") },
        { id: "dark", label: qsTr("Dark"), detail: qsTr("deterministic dark-surface rendering") },
        { id: "tinted", label: qsTr("Tinted"), detail: qsTr("single-accent rendering from the semantic SVG") },
        { id: "clear", label: qsTr("Clear"), detail: qsTr("transparent low-fill rendering") }
    ]

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function setValue(key, value) {
        if (change.running)
            return;
        root.message = "";
        change.action = "set";
        change.command = ["@vesperControl@", "icons", "set", key, value];
        change.running = true;
    }

    function runReconcile() {
        if (change.running)
            return;
        root.message = "";
        change.action = "reconcile";
        change.command = ["@vesperControl@", "icons", "reconcile"];
        change.running = true;
    }

    function stateSummary() {
        const counts = root.iconState.counts || {};
        return qsTr("%1 prepared · %2 pending · %3 fallback")
            .arg(counts.prepared || 0)
            .arg(counts.pending || 0)
            .arg(counts.fallback || 0);
    }

    Component.onCompleted: refresh()

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refresh()
    }

    Process {
        id: status
        command: ["@vesperControl@", "icons", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.iconState = JSON.parse(text);
                    root.message = "";
                    tintField.text = root.iconState.tint || "#8aadf4";
                    providerField.text = root.iconState.provider || "openai";
                    credentialField.text = root.iconState.credential || "openai";
                    modelField.text = root.iconState.model || "gpt-5";
                } catch (e) {
                    root.message = qsTr("Could not read App Icons state");
                }
            }
        }
    }

    Process {
        id: change
        property string action: ""
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => {
            root.message = code === 0
                ? (action === "reconcile" ? qsTr("Icon inventory reconciled") : qsTr("App Icons updated"))
                : changeError.text.trim();
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
            text: qsTr("App Icons")
        }

        ToggleRow {
            text: qsTr("App Icons")
            subtext: qsTr("curated semantic SVGs + automatic AI generation for every other installed app")
            checked: root.iconState.enabled === true
            disabled: change.running
            onToggled: root.setValue("enabled", checked ? "on" : "off")
        }

        InfoRow {
            icon: "auto_awesome"
            label: qsTr("Inventory")
            subtext: qsTr("new apps and changed source icons are detected by reconciliation")
            value: root.stateSummary()
        }

        SectionHeader {
            text: qsTr("Appearance")
        }

        Repeater {
            model: root.modes

            delegate: RowButton {
                required property var modelData
                icon: root.iconState.mode === modelData.id ? "check_circle" : "circle"
                text: modelData.label
                subtext: modelData.detail
                disabled: change.running
                onClicked: root.setValue("mode", modelData.id)
            }
        }

        StyledTextField {
            id: tintField
            Layout.fillWidth: true
            placeholderText: "#8aadf4"
            leadingIcon: "palette"
            supportingText: qsTr("Tint color · #RRGGBB · changing it never calls AI")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
            onAccepted: root.setValue("tint", text.trim())
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.small
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "save"
                text: qsTr("Apply tint")
                disabled: change.running || !tintField.text.trim()
                onClicked: root.setValue("tint", tintField.text.trim())
            }
        }

        SectionHeader {
            text: qsTr("AI curator")
        }

        StyledTextField {
            id: providerField
            Layout.fillWidth: true
            placeholderText: "openai"
            leadingIcon: "smart_toy"
            supportingText: qsTr("provider adapter · OpenAI is currently production-enabled")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        StyledTextField {
            id: credentialField
            Layout.fillWidth: true
            placeholderText: "openai"
            leadingIcon: "key"
            supportingText: qsTr("logical Secret Service credential alias; plaintext is never stored here")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        StyledTextField {
            id: modelField
            Layout.fillWidth: true
            placeholderText: "gpt-5"
            leadingIcon: "model_training"
            supportingText: qsTr("vision-capable Responses API model")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: Tokens.spacing.small
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "save"
                text: qsTr("Save curator")
                disabled: change.running || !providerField.text.trim() || !credentialField.text.trim() || !modelField.text.trim()
                onClicked: {
                    root.setValue("provider", providerField.text.trim());
                    // Subsequent values are applied after the first process finishes
                    // by explicit user action if the backend rejects a provider.
                    pendingCuratorSave.running = true;
                }
            }
        }

        Timer {
            id: pendingCuratorSave
            property int step: 0
            interval: 150
            repeat: true
            onTriggered: {
                if (change.running)
                    return;
                if (step === 0) {
                    root.setValue("credential", credentialField.text.trim());
                    step = 1;
                } else if (step === 1) {
                    root.setValue("model", modelField.text.trim());
                    step = 2;
                } else {
                    step = 0;
                    running = false;
                }
            }
        }

        RowButton {
            icon: "sync"
            text: qsTr("Reconcile now")
            subtext: qsTr("discover installed apps, validate curated assets, process a bounded number of AI jobs and refresh the live registry")
            disabled: change.running
            onClicked: root.runReconcile()
        }

        SectionHeader {
            text: qsTr("Queue")
        }

        Repeater {
            model: (root.iconState.jobs || []).slice(0, 20)

            delegate: InfoRow {
                required property var modelData
                icon: modelData.state === "prepared" ? "check_circle" : (modelData.state === "fallback" ? "warning" : "schedule")
                label: modelData.name || modelData.id
                subtext: modelData.error || modelData.sourceType || ""
                value: modelData.state
            }
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.small
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("updated") || root.message.toLowerCase().includes("reconciled")
                ? Colours.palette.m3primary
                : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
