pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var app
    property var status: ({ sandbox: "native", flatpakId: "", permissions: "", todaySeconds: 0 })
    property string message: ""

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    readonly property bool flatpak: status.sandbox === "flatpak"
    readonly property bool networkAllowed: flatpak && status.permissions.includes("shared=network")
    readonly property bool homeAllowed: flatpak && (status.permissions.includes("filesystems=home") || status.permissions.includes(";home;") || status.permissions.includes(";home:"))

    function duration(seconds) {
        const minutes = Math.floor((seconds || 0) / 60);
        if (minutes < 60)
            return qsTr("%1 min").arg(minutes);
        return qsTr("%1 h %2 min").arg(Math.floor(minutes / 60)).arg(minutes % 60);
    }

    function refresh() {
        if (root.app && !appStatus.running) {
            appStatus.command = ["@vesperControl@", "app-status", root.app.id];
            appStatus.running = true;
        }
    }

    onAppChanged: refresh()
    Component.onCompleted: refresh()

    Process {
        id: appStatus
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.status = JSON.parse(text);
                } catch (e) {
                    root.message = qsTr("Could not read app controls");
                }
            }
        }
    }

    Process {
        id: permissionChange
        stderr: StdioCollector { id: permissionError }
        onExited: (code, status) => {
            root.message = code === 0 ? "" : permissionError.text.trim();
            root.refresh();
        }
    }

    Process {
        id: iconRequest
        stderr: StdioCollector { id: iconError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Adaptive icon job queued") : iconError.text.trim();
        }
    }

    SectionHeader {
        text: qsTr("Privacy & permissions")
    }

    InfoRow {
        icon: root.flatpak ? "deployed_code" : "warning"
        label: qsTr("Sandbox")
        subtext: root.flatpak ? root.status.flatpakId : qsTr("native Nix app · Flatpak overrides do not apply")
        value: root.flatpak ? qsTr("Flatpak") : qsTr("native")
        iconColour: root.flatpak ? Colours.palette.m3primary : Colours.palette.m3tertiary
    }

    ToggleRow {
        visible: root.flatpak
        text: qsTr("Network access")
        subtext: qsTr("Flatpak network share override")
        checked: root.networkAllowed
        onToggled: {
            permissionChange.command = ["@vesperControl@", "app-permission", root.app.id, "network", checked ? "on" : "off"];
            permissionChange.running = true;
        }
    }

    ToggleRow {
        visible: root.flatpak
        text: qsTr("Home folder access")
        subtext: qsTr("Flatpak home filesystem override")
        checked: root.homeAllowed
        onToggled: {
            permissionChange.command = ["@vesperControl@", "app-permission", root.app.id, "home", checked ? "on" : "off"];
            permissionChange.running = true;
        }
    }

    RowButton {
        visible: root.flatpak
        icon: "restart_alt"
        text: qsTr("Reset Flatpak overrides")
        subtext: qsTr("return this app to its packaged permissions")
        onClicked: {
            permissionChange.command = ["@vesperControl@", "app-reset-permissions", root.app.id];
            permissionChange.running = true;
        }
    }

    SectionHeader {
        text: qsTr("Wellbeing")
    }

    InfoRow {
        icon: "timer"
        label: qsTr("Foreground time today")
        subtext: qsTr("local Hyprland activity sample")
        value: root.duration(root.status.todaySeconds)
    }

    SectionHeader {
        text: qsTr("Experimental")
    }

    RowButton {
        icon: "auto_awesome"
        text: qsTr("Queue adaptive icon")
        subtext: qsTr("hand this app icon to the Vesper AI icon workflow for review")
        disabled: !root.app || iconRequest.running
        onClicked: {
            iconRequest.command = ["@vesperControl@", "icon", "request", root.app.id, root.app.icon || ""];
            iconRequest.running = true;
        }
    }

    StyledText {
        Layout.fillWidth: true
        Layout.leftMargin: Tokens.padding.largeIncreased
        visible: root.message
        text: root.message
        color: root.message.includes(qsTr("queued")) ? Colours.palette.m3primary : Colours.palette.m3error
        font: Tokens.font.label.small
        wrapMode: Text.WordWrap
    }
}
