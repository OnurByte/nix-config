pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var settings: ({ followTheme: true, followIcons: true, useGlass: true, closeOnFocusLoss: true, popToRootOnClose: true, layerShell: true, accent: "" })
    property string message: ""
    property bool messageOk: false

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    function refresh(): void {
        if (!status.running)
            status.running = true;
    }

    function setSetting(key: string, value: bool): void {
        if (settingChange.running)
            return;
        root.message = "";
        settingChange.command = ["@vesperControl@", "vicinae-setting", key, value ? "on" : "off"];
        settingChange.running = true;
    }

    function syncTheme(): void {
        if (syncAction.running)
            return;
        root.message = "";
        syncAction.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status

        command: ["@vesperControl@", "vicinae-status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.settings = JSON.parse(text);
                } catch (e) {
                    root.message = qsTr("Could not read Vicinae settings");
                    root.messageOk = false;
                }
            }
        }
    }

    Process {
        id: settingChange

        stderr: StdioCollector { id: settingError }
        onExited: (code, status) => {
            if (code === 0) {
                root.message = qsTr("Vicinae setting saved");
                root.messageOk = true;
                root.refresh();
            } else {
                root.message = settingError.text.trim() || qsTr("Could not save Vicinae setting");
                root.messageOk = false;
            }
        }
    }

    Process {
        id: syncAction

        command: ["@vesperControl@", "vicinae-sync-theme"]
        stderr: StdioCollector { id: syncError }
        onExited: (code, status) => {
            if (code === 0) {
                root.message = qsTr("Vicinae is using the current Vesper palette");
                root.messageOk = true;
                root.refresh();
            } else {
                root.message = syncError.text.trim() || qsTr("Could not sync Vicinae theme");
                root.messageOk = false;
            }
        }
    }

    SectionHeader {
        first: true
        text: qsTr("Vicinae")
    }

    StyledText {
        Layout.fillWidth: true
        text: qsTr("Vicinae is Vesper's Spotlight-style launcher. The Windows key opens it, while Super + Space remains an alternate shortcut.")
        color: Colours.palette.m3onSurfaceVariant
        font: Tokens.font.body.small
        wrapMode: Text.WordWrap
    }

    InfoRow {
        icon: "keyboard"
        label: qsTr("Launcher shortcut")
        subtext: qsTr("Primary Vesper launcher entry point")
        value: qsTr("Super")
    }

    InfoRow {
        icon: "palette"
        label: qsTr("System appearance")
        subtext: qsTr("Read from the active Caelestia scheme")
        value: `${Colours.scheme} · ${Colours.flavour}`
    }

    InfoRow {
        icon: "color_lens"
        label: qsTr("Current accent")
        subtext: qsTr("The generated Vesper accent used by the launcher theme")
        value: root.settings.accent || qsTr("pending")
    }

    RowButton {
        icon: "open_in_new"
        text: qsTr("Open Vicinae")
        subtext: qsTr("Open the launcher and its built-in commands")
        onClicked: Quickshell.execDetached(["vicinae", "toggle"])
    }

    SectionHeader {
        text: qsTr("Vesper integration")
    }

    ToggleRow {
        text: qsTr("Follow Vesper theme")
        subtext: qsTr("Use the current light/dark scheme and Caelestia accent")
        checked: root.settings.followTheme
        disabled: settingChange.running
        onToggled: root.setSetting("follow-theme", checked)
    }

    ToggleRow {
        text: qsTr("Use Vesper adaptive icons")
        subtext: qsTr("Use Vesper-Adaptive for Vicinae application results")
        checked: root.settings.followIcons
        disabled: settingChange.running
        onToggled: root.setSetting("follow-icons", checked)
    }

    ToggleRow {
        text: qsTr("Use Vesper glass surface")
        subtext: qsTr("Apply Vesper's controlled opacity to the launcher window")
        checked: root.settings.useGlass
        disabled: settingChange.running
        onToggled: root.setSetting("use-glass", checked)
    }

    RowButton {
        icon: "sync"
        text: qsTr("Sync Vesper theme now")
        subtext: qsTr("Regenerate the local Vicinae theme from the active accent")
        trailingIcon: "refresh"
        disabled: syncAction.running
        onClicked: root.syncTheme()
    }

    SectionHeader {
        text: qsTr("Launcher behavior")
    }

    ToggleRow {
        text: qsTr("Close on focus loss")
        subtext: qsTr("Hide the launcher when another window receives focus")
        checked: root.settings.closeOnFocusLoss
        disabled: settingChange.running
        onToggled: root.setSetting("close-on-focus-loss", checked)
    }

    ToggleRow {
        text: qsTr("Return to root on close")
        subtext: qsTr("Start the next invocation at the root search")
        checked: root.settings.popToRootOnClose
        disabled: settingChange.running
        onToggled: root.setSetting("pop-to-root-on-close", checked)
    }

    ToggleRow {
        last: true
        text: qsTr("Layer-shell window")
        subtext: qsTr("Use the Wayland layer-shell launcher surface")
        checked: root.settings.layerShell
        disabled: settingChange.running
        onToggled: root.setSetting("layer-shell", checked)
    }

    StyledText {
        Layout.fillWidth: true
        visible: root.message
        text: root.message
        color: root.messageOk ? Colours.palette.m3primary : Colours.palette.m3error
        font: Tokens.font.body.small
        wrapMode: Text.WordWrap
    }
}
