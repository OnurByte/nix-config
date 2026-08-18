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

    property var dpi: ({ active: false, service: "", managedBy: "nix", mutableProfile: false, profile: {} })
    property var testResult: ({})
    property string message: ""

    title: qsTr("DPI / Zapret")
    isSubPage: true

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status
        command: ["@vesperControl@", "network", "dpi-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.dpi = JSON.parse(text);
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Could not read Zapret profile state");
                }
            }
        }
    }

    Process {
        id: test
        stderr: StdioCollector { id: testError }
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.testResult = JSON.parse(text);
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Zapret reachability test returned invalid data");
                }
            }
        }
        onExited: (code, status) => {
            if (code !== 0)
                root.message = testError.text.trim();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Profile") }

        InfoRow {
            icon: root.dpi.active ? "check_circle" : "pause_circle"
            label: qsTr("Zapret worker")
            subtext: root.dpi.service || "nfqws2@default.service"
            value: root.dpi.active ? qsTr("active") : qsTr("inactive")
        }
        InfoRow {
            icon: "deployed_code"
            label: qsTr("Ownership")
            subtext: qsTr("profile parameters come directly from the NixOS Zapret definition; Settings does not rewrite them")
            value: root.dpi.managedBy || qsTr("nix")
        }
        InfoRow {
            icon: "tune"
            label: qsTr("Active profile")
            subtext: qsTr("host autodetect: %1 · max packets: %2")
                .arg(root.dpi.profile?.hostAutodetect ? qsTr("on") : qsTr("off"))
                .arg(root.dpi.profile?.maxPackets ?? "--")
            value: root.dpi.profile?.name || qsTr("default")
        }
        InfoRow {
            icon: "lan"
            label: qsTr("Scope")
            subtext: qsTr("TCP %1 · UDP %2 · payload %3")
                .arg((root.dpi.profile?.tcpPorts || []).join(",") || qsTr("none"))
                .arg((root.dpi.profile?.udpPorts || []).join(",") || qsTr("none"))
                .arg(root.dpi.profile?.payload || qsTr("unknown"))
            value: root.dpi.mutableProfile ? qsTr("runtime") : qsTr("declarative")
        }

        Repeater {
            model: root.dpi.profile?.parameters || []
            delegate: InfoRow {
                required property string modelData
                Layout.fillWidth: true
                icon: "code"
                label: qsTr("Parameter")
                subtext: modelData
                value: qsTr("Nix")
            }
        }

        SectionHeader { text: qsTr("Reachability test") }

        StyledTextField {
            id: domainField
            Layout.fillWidth: true
            placeholderText: qsTr("example.com")
            leadingIcon: "language"
            supportingText: qsTr("HTTPS reachability only; a successful result does not claim Zapret caused the success")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }

        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "network_check"
                text: qsTr("Test")
                disabled: test.running || !domainField.text.trim()
                onClicked: {
                    root.testResult = ({});
                    test.command = ["@vesperControl@", "network", "dpi-test", domainField.text.trim()];
                    test.running = true;
                }
            }
        }

        InfoRow {
            visible: !!root.testResult.domain
            icon: root.testResult.reachable ? "check_circle" : "error"
            label: root.testResult.domain || ""
            subtext: root.testResult.error || qsTr("HTTPS request completed")
            value: root.testResult.reachable
                ? qsTr("HTTP %1").arg(root.testResult.httpCode || "--")
                : qsTr("unreachable")
        }

        InfoRow {
            icon: "info"
            label: qsTr("Tuning policy")
            subtext: qsTr("No fake desync/host-list sliders are exposed because this installed profile is Nix-owned. Change the declarative profile to change those parameters.")
            value: qsTr("read-only")
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.message
            text: root.message
            color: Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
