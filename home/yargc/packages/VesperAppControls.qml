pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.services
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var app
    property var status: ({ sandbox: "native", flatpakId: "", permissions: "", todaySeconds: 0 })
    property var iconStatus: ({ id: "", canonicalAppId: "", launchDesktopId: "", sourcePath: "", sourceKind: "", sourceResolver: "", fingerprint: "", canonicalState: "unresolved-source", queueState: "", active: false, excluded: false, error: "", workKey: "" })
    property string message: ""
    property bool messageOk: false

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
        if (!root.app)
            return;
        if (!appStatus.running) {
            appStatus.command = ["@vesperControl@", "app-status", root.app.id];
            appStatus.running = true;
        }
        if (!iconAppStatus.running) {
            iconAppStatus.command = ["@vesperControl@", "icon", "app-status", root.app.id];
            iconAppStatus.running = true;
        }
    }

    function runIcon(args, successMessage) {
        if (!root.app || iconAction.running)
            return;
        root.message = "";
        root.messageOk = false;
        iconAction.successMessage = successMessage;
        iconAction.command = ["@vesperControl@", "icon"].concat(args);
        iconAction.running = true;
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
                    root.messageOk = false;
                }
            }
        }
    }

    Process {
        id: iconAppStatus
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.iconStatus = JSON.parse(text);
                } catch (e) {
                    root.iconStatus = ({ id: root.app ? root.app.id : "", canonicalAppId: "", launchDesktopId: "", sourcePath: "", sourceKind: "", sourceResolver: "", fingerprint: "", canonicalState: "unresolved-source", queueState: "", active: false, excluded: false, error: qsTr("not discovered yet"), workKey: "" });
                }
            }
        }
    }

    Process {
        id: permissionChange
        stderr: StdioCollector { id: permissionError }
        onExited: (code, status) => {
            root.message = code === 0 ? "" : permissionError.text.trim();
            root.messageOk = code === 0;
            root.refresh();
        }
    }

    Process {
        id: iconAction
        property string successMessage: ""
        stdout: StdioCollector { id: iconOutput }
        stderr: StdioCollector { id: iconError }
        onExited: (code, status) => {
            const output = iconOutput.text.trim();
            root.message = code === 0 ? (output || successMessage) : iconError.text.trim();
            root.messageOk = code === 0;
            root.refresh();
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
        disabled: permissionChange.running
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
        disabled: permissionChange.running
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
        disabled: permissionChange.running
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
        text: qsTr("Adaptive icon")
    }

    InfoRow {
        icon: "fingerprint"
        label: qsTr("Resolved identity")
        subtext: root.iconStatus.launchDesktopId || (root.app ? root.app.id : "")
        value: root.iconStatus.canonicalAppId || qsTr("unknown")
    }

    InfoRow {
        icon: "image"
        label: qsTr("Source icon")
        subtext: root.iconStatus.sourcePath || qsTr("source not resolved")
        value: root.iconStatus.sourceResolver || root.iconStatus.sourceKind || qsTr("unknown")
    }

    InfoRow {
        icon: root.iconStatus.canonicalState === "canonical-ai" ? "verified" : "layers"
        label: qsTr("Canonical tier")
        subtext: root.iconStatus.error || qsTr("source fingerprint %1").arg((root.iconStatus.fingerprint || "").slice(0, 12))
        value: root.iconStatus.canonicalState || qsTr("unresolved-source")
        iconColour: root.iconStatus.canonicalState === "canonical-ai" ? Colours.palette.m3primary : Colours.palette.m3tertiary
    }

    InfoRow {
        icon: "schedule"
        label: qsTr("Conversion job")
        subtext: root.iconStatus.queueState ? qsTr("persistent work queue state") : qsTr("no conversion job")
        value: root.iconStatus.queueState || qsTr("none")
    }

    InfoRow {
        icon: "palette"
        label: qsTr("Active appearance")
        subtext: root.iconStatus.active ? qsTr("served by the Vesper overlay") : qsTr("using inherited packaged/original icon")
        value: root.iconStatus.active ? qsTr("Vesper") : qsTr("original")
    }

    ToggleRow {
        text: qsTr("Use original icon")
        subtext: qsTr("exclude this app from Vesper without modifying its packaged desktop entry")
        checked: root.iconStatus.excluded || false
        disabled: !root.app || iconAction.running
        onToggled: root.runIcon(["app-exclude", root.app.id, checked ? "on" : "off"], checked ? qsTr("Original icon restored") : qsTr("App returned to adaptive icons"))
    }

    RowButton {
        icon: "restart_alt"
        text: qsTr("Retry this icon")
        subtext: qsTr("reset this app's failed conversion job and preserve any last-known-good package")
        trailingIcon: "refresh"
        disabled: !root.app || iconAction.running
        onClicked: root.runIcon(["app-retry", root.app.id], qsTr("Adaptive icon retry queued"))
    }

    RowButton {
        icon: "download"
        text: qsTr("Export current icon")
        subtext: qsTr("render the accepted canonical package locally; no provider request")
        trailingIcon: "download"
        disabled: !root.app || iconAction.running || root.iconStatus.canonicalState !== "canonical-ai"
        onClicked: root.runIcon(["app-export", root.app.id], qsTr("Icon exported"))
    }

    StyledText {
        Layout.fillWidth: true
        Layout.leftMargin: Tokens.padding.largeIncreased
        visible: root.message
        text: root.message
        color: root.messageOk ? Colours.palette.m3primary : Colours.palette.m3error
        font: Tokens.font.label.small
        wrapMode: Text.WordWrap
    }
}
