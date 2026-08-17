pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services
import qs.modules.nexus.common

PageBase {
    id: root

    property var zapret: ({ active: false, repeats: 1, split: "default", splitPos: "1,midsld", runtimeOverride: false, scope: "" })
    property string message: ""

    title: qsTr("Zapret2")
    isSubPage: true

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function apply(repeats, split) {
        if (change.running)
            return;
        root.message = "";
        change.command = ["@vesperNetwork@", "zapret", "set", String(repeats), split];
        change.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status
        command: ["@vesperNetwork@", "zapret", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.zapret = JSON.parse(text);
                } catch (e) {
                    root.message = qsTr("Could not read Zapret2 state");
                }
            }
        }
    }

    Process {
        id: change
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Zapret2 tuning applied") : changeError.text.trim();
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
            text: qsTr("Runtime strategy")
        }

        InfoRow {
            icon: "shield"
            label: qsTr("Service")
            subtext: root.zapret.runtimeOverride ? qsTr("runtime override active") : qsTr("Nix default")
            value: root.zapret.active ? qsTr("active") : qsTr("inactive")
            iconColour: root.zapret.active ? Colours.palette.m3primary : Colours.palette.m3error
        }

        SectionHeader {
            text: qsTr("Fake repeats")
        }

        RowButton {
            icon: root.zapret.repeats === 1 ? "check_circle" : "radio_button_unchecked"
            text: qsTr("1 repeat")
            subtext: qsTr("lowest overhead · Nix default")
            disabled: change.running
            onClicked: root.apply(1, root.zapret.split)
        }

        RowButton {
            icon: root.zapret.repeats === 2 ? "check_circle" : "radio_button_unchecked"
            text: qsTr("2 repeats")
            subtext: qsTr("moderate fake replay")
            disabled: change.running
            onClicked: root.apply(2, root.zapret.split)
        }

        RowButton {
            icon: root.zapret.repeats === 4 ? "check_circle" : "radio_button_unchecked"
            text: qsTr("4 repeats")
            subtext: qsTr("stronger fake replay")
            disabled: change.running
            onClicked: root.apply(4, root.zapret.split)
        }

        RowButton {
            icon: root.zapret.repeats === 6 ? "check_circle" : "radio_button_unchecked"
            text: qsTr("6 repeats")
            subtext: qsTr("highest selectable replay")
            disabled: change.running
            onClicked: root.apply(6, root.zapret.split)
        }

        SectionHeader {
            text: qsTr("Split pattern")
        }

        RowButton {
            icon: root.zapret.split === "default" ? "check_circle" : "radio_button_unchecked"
            text: qsTr("1 + middle SLD")
            subtext: qsTr("pos=1,midsld · Nix default")
            disabled: change.running
            onClicked: root.apply(root.zapret.repeats, "default")
        }

        RowButton {
            icon: root.zapret.split === "method" ? "check_circle" : "radio_button_unchecked"
            text: qsTr("Method + middle SLD")
            subtext: qsTr("pos=method+2,midsld")
            disabled: change.running
            onClicked: root.apply(root.zapret.repeats, "method")
        }

        RowButton {
            icon: root.zapret.split === "sni" ? "check_circle" : "radio_button_unchecked"
            text: qsTr("SNI extension + middle SLD")
            subtext: qsTr("pos=1,sniext+1,midsld")
            disabled: change.running
            onClicked: root.apply(root.zapret.repeats, "sni")
        }

        SectionHeader {
            text: qsTr("Fixed safety boundary")
        }

        InfoRow {
            icon: "filter_alt"
            label: qsTr("Interception scope")
            subtext: qsTr("runtime tuning cannot widen nftables rules")
            value: qsTr("TCP 443 · 16 packets")
        }

        InfoRow {
            icon: "travel_explore"
            label: qsTr("Host selection")
            subtext: qsTr("persisted by Zapret2 only when bypass appears necessary")
            value: qsTr("adaptive")
        }

        RowButton {
            icon: "restart_alt"
            text: qsTr("Reset to Nix default")
            subtext: qsTr("1 repeat · pos=1,midsld")
            disabled: change.running || !root.zapret.runtimeOverride
            onClicked: {
                root.message = "";
                change.command = ["@vesperNetwork@", "zapret", "reset"];
                change.running = true;
            }
        }

        StyledText {
            Layout.fillWidth: true
            Layout.topMargin: Tokens.spacing.medium
            visible: root.message
            text: root.message
            color: root.message.includes(qsTr("applied")) ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
