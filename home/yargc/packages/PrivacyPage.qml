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
    property var privacy: ({ tor: {}, zapret: {}, firewall: {}, network: {} })
    property string errorText: ""
    title: qsTr("Privacy")

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    Component.onCompleted: refresh()
    Timer { interval: 10000; repeat: true; running: root.visible; onTriggered: root.refresh() }

    Process {
        id: status
        command: ["@vesperControl@", "privacy", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.privacy = JSON.parse(text);
                    root.errorText = "";
                } catch (e) {
                    root.errorText = qsTr("Could not read privacy state");
                }
            }
        }
        stderr: StdioCollector { id: statusError }
        onExited: (code, status) => {
            if (code !== 0)
                root.errorText = statusError.text.trim();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Anonymity") }
        InfoRow {
            icon: "security"
            label: qsTr("Tor")
            subtext: qsTr("Nix-managed system client; Settings reports state without mutating declarative policy")
            value: root.privacy.tor?.active ? qsTr("active") : qsTr("inactive")
        }
        InfoRow {
            icon: "lan"
            label: qsTr("Tor SOCKS")
            subtext: qsTr("local listener on 127.0.0.1:9050")
            value: root.privacy.tor?.socksListening ? qsTr("listening") : qsTr("not detected")
        }
        InfoRow {
            icon: "shield"
            label: qsTr("Zapret DPI bypass")
            subtext: qsTr("system policy from the privacy module")
            value: root.privacy.zapret?.active ? qsTr("active") : qsTr("inactive")
        }

        SectionHeader { text: qsTr("Network privacy") }
        InfoRow {
            icon: "local_fire_department"
            label: qsTr("Firewall")
            subtext: qsTr("NixOS firewall service")
            value: root.privacy.firewall?.active ? qsTr("active") : qsTr("inactive")
        }
        InfoRow {
            icon: "dns"
            label: qsTr("DNS")
            subtext: root.privacy.network?.dns || qsTr("resolver details unavailable")
            value: qsTr("runtime")
        }
        InfoRow {
            icon: "wifi_lock"
            label: qsTr("Wi-Fi MAC policy")
            subtext: root.privacy.network?.wifiMacPolicy || qsTr("no active Wi-Fi profile or no explicit policy")
            value: root.privacy.network?.wifiMacPolicy ? qsTr("configured") : qsTr("default")
        }

        InfoRow {
            icon: "deployed_code"
            label: qsTr("Ownership")
            subtext: qsTr("Tor, Zapret, firewall, DNS policy and MAC policy stay declarative where configured by Nix; unsupported runtime toggles are intentionally not shown")
            value: qsTr("Nix-first")
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
