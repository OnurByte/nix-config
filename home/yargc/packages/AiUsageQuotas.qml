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

    property var ai: ({ summary: {}, providers: [], stale: true })
    property string errorText: ""

    title: qsTr("Usage & Quotas")
    isSubPage: true

    function refresh(force) {
        if (status.running)
            return;
        status.command = force
            ? ["vesper-ai", "status", "--refresh"]
            : ["vesper-ai", "status"];
        status.running = true;
    }

    function percent(value) {
        if (value === null || value === undefined || value < 0)
            return qsTr("unknown");
        return qsTr("%1%").arg(Number(value).toFixed(1));
    }

    function objectSummary(value) {
        if (!value || typeof value !== "object")
            return qsTr("unknown");
        const keys = Object.keys(value);
        if (keys.length === 0)
            return qsTr("unknown");
        return keys.map(key => `${key}: ${String(value[key])}`).join(" · ");
    }

    Component.onCompleted: refresh(false)

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refresh(false)
    }

    Process {
        id: status
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.ai = JSON.parse(text);
                    root.errorText = root.ai.backendError || "";
                } catch (e) {
                    root.errorText = qsTr("AI usage backend returned invalid data");
                }
            }
        }
        stderr: StdioCollector { id: statusError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = statusError.text.trim() || qsTr("Could not refresh AI usage");
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Normalized status") }

        InfoRow {
            icon: root.ai.stale ? "sync_problem" : "verified"
            label: qsTr("Snapshot")
            subtext: root.ai.generatedAt || qsTr("generation time unavailable")
            value: root.ai.stale ? qsTr("stale") : qsTr("fresh")
        }

        InfoRow {
            icon: "dns"
            label: qsTr("Providers")
            subtext: qsTr("critical %1 · warning %2")
                .arg(root.ai.summary?.criticalCount ?? 0)
                .arg(root.ai.summary?.warningCount ?? 0)
            value: String(root.ai.summary?.providerCount ?? (root.ai.providers || []).length)
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "refresh"
                text: qsTr("Refresh")
                disabled: status.running
                onClicked: root.refresh(true)
            }
        }

        Repeater {
            model: root.ai.providers || []

            delegate: ColumnLayout {
                id: providerRow
                required property var modelData
                Layout.fillWidth: true
                spacing: Tokens.spacing.extraSmall / 2

                SectionHeader {
                    text: providerRow.modelData.name || providerRow.modelData.id || qsTr("Provider")
                }

                InfoRow {
                    icon: providerRow.modelData.health === "critical"
                        ? "error"
                        : (providerRow.modelData.health === "warning" ? "warning" : "check_circle")
                    label: qsTr("Health")
                    subtext: providerRow.modelData.error
                        || providerRow.modelData.statusLabel
                        || qsTr("no provider error reported")
                    value: providerRow.modelData.health || qsTr("unknown")
                }

                InfoRow {
                    icon: "account_circle"
                    label: qsTr("Account / plan")
                    subtext: providerRow.modelData.account || qsTr("account identity unavailable")
                    value: providerRow.modelData.plan || qsTr("unknown")
                }

                InfoRow {
                    icon: "source"
                    label: qsTr("Source")
                    subtext: providerRow.modelData.updatedAt || qsTr("update time unavailable")
                    value: providerRow.modelData.source || qsTr("unknown")
                }

                Repeater {
                    model: providerRow.modelData.windows || []
                    delegate: InfoRow {
                        required property var modelData
                        Layout.fillWidth: true
                        icon: "data_usage"
                        label: modelData.label || modelData.kind || qsTr("Quota")
                        subtext: modelData.resetAt
                            ? qsTr("resets %1").arg(modelData.resetAt)
                            : qsTr("reset time unavailable")
                        value: qsTr("used %1 · left %2")
                            .arg(root.percent(modelData.usedPercent))
                            .arg(root.percent(modelData.remainingPercent))
                    }
                }

                InfoRow {
                    visible: providerRow.modelData.credits !== null && providerRow.modelData.credits !== undefined
                    icon: "payments"
                    label: qsTr("Credits")
                    subtext: qsTr("provider-native credit object; unknown fields are not fabricated")
                    value: root.objectSummary(providerRow.modelData.credits)
                }

                InfoRow {
                    visible: providerRow.modelData.cost !== null && providerRow.modelData.cost !== undefined
                    icon: "receipt_long"
                    label: qsTr("Cost")
                    subtext: qsTr("provider-native cost windows, including today/30-day when reported")
                    value: root.objectSummary(providerRow.modelData.cost)
                }
            }
        }

        InfoRow {
            visible: (root.ai.providers || []).length === 0
            icon: "info"
            label: qsTr("No provider usage reported")
            subtext: qsTr("The shared AI backend did not return any enabled quota providers")
            value: root.ai.stale ? qsTr("stale") : qsTr("empty")
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.errorText
            text: root.errorText
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
