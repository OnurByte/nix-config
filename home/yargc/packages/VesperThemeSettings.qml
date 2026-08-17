pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Caelestia.Config
import qs.components.controls
import qs.services
import qs.modules.nexus.common

ColumnLayout {
    id: root

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall

    property string currentVariant: "tonalspot"

    function setScheme(name: string, flavour: string): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-n", name, "-f", flavour]);
        refreshTimer.restart();
    }

    function setVariant(variant: string): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-v", variant]);
        currentVariant = variant;
        refreshTimer.restart();
    }

    function setMode(mode: string): void {
        // Keep the native shell state immediate, then ask Caelestia CLI to
        // regenerate the shared GTK, Qt and Hyprland palettes in one pass.
        Colours.setMode(mode);
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-m", mode]);
        refreshTimer.restart();
    }

    function setGlass(enabled: bool): void {
        // Caelestia owns translucent shell surfaces; Hyprland supplies the
        // backdrop blur. Do not lower whole-window opacity because that would
        // fade text and icons together with the background.
        GlobalConfig.appearance.transparency.enabled = enabled;
        Quickshell.execDetached([
            "hyprctl",
            "keyword",
            "decoration:blur:enabled",
            enabled ? "true" : "false"
        ]);
    }

    function reapplyToolkitThemes(): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-m", Colours.light ? "light" : "dark"]);
        refreshTimer.restart();
    }

    Component.onCompleted: {
        // appearance.lua defaults blur to enabled. Reconcile it with the
        // persisted Caelestia transparency state whenever the settings page
        // is instantiated so one Glass switch remains authoritative.
        Quickshell.execDetached([
            "hyprctl",
            "keyword",
            "decoration:blur:enabled",
            Colours.transparency.enabled ? "true" : "false"
        ]);
    }

    property list<MenuItem> schemeItems: [
        MenuItem {
            objectName: "caelestia/default"
            text: qsTr("Caelestia")
            icon: "palette"
            onClicked: root.setScheme("caelestia", "default")
        },
        MenuItem {
            objectName: "catppuccin/mocha"
            text: qsTr("Catppuccin Mocha")
            icon: "palette"
            onClicked: root.setScheme("catppuccin", "mocha")
        },
        MenuItem {
            objectName: "catppuccin/frappe"
            text: qsTr("Catppuccin Frappe")
            icon: "palette"
            onClicked: root.setScheme("catppuccin", "frappe")
        },
        MenuItem {
            objectName: "dracula/medium"
            text: qsTr("Dracula")
            icon: "palette"
            onClicked: root.setScheme("dracula", "medium")
        },
        MenuItem {
            objectName: "everforest/medium"
            text: qsTr("Everforest")
            icon: "palette"
            onClicked: root.setScheme("everforest", "medium")
        },
        MenuItem {
            objectName: "gruvbox/medium"
            text: qsTr("Gruvbox")
            icon: "palette"
            onClicked: root.setScheme("gruvbox", "medium")
        },
        MenuItem {
            objectName: "nord/medium"
            text: qsTr("Nord")
            icon: "palette"
            onClicked: root.setScheme("nord", "medium")
        },
        MenuItem {
            objectName: "rosepine/main"
            text: qsTr("Rose Pine")
            icon: "palette"
            onClicked: root.setScheme("rosepine", "main")
        },
        MenuItem {
            objectName: "solarized/medium"
            text: qsTr("Solarized")
            icon: "palette"
            onClicked: root.setScheme("solarized", "medium")
        },
        MenuItem {
            objectName: "tokyonight/medium"
            text: qsTr("Tokyo Night")
            icon: "palette"
            onClicked: root.setScheme("tokyonight", "medium")
        },
        MenuItem {
            objectName: "dynamic/default"
            text: qsTr("Dynamic wallpaper")
            icon: "auto_awesome"
            onClicked: root.setScheme("dynamic", "default")
        }
    ]

    property list<MenuItem> variantItems: [
        MenuItem {
            objectName: "tonalspot"
            text: qsTr("Tonal spot")
            onClicked: root.setVariant("tonalspot")
        },
        MenuItem {
            objectName: "vibrant"
            text: qsTr("Vibrant")
            onClicked: root.setVariant("vibrant")
        },
        MenuItem {
            objectName: "expressive"
            text: qsTr("Expressive")
            onClicked: root.setVariant("expressive")
        },
        MenuItem {
            objectName: "fidelity"
            text: qsTr("Fidelity")
            onClicked: root.setVariant("fidelity")
        },
        MenuItem {
            objectName: "fruitsalad"
            text: qsTr("Fruit salad")
            onClicked: root.setVariant("fruitsalad")
        },
        MenuItem {
            objectName: "monochrome"
            text: qsTr("Monochrome")
            onClicked: root.setVariant("monochrome")
        },
        MenuItem {
            objectName: "neutral"
            text: qsTr("Neutral")
            onClicked: root.setVariant("neutral")
        },
        MenuItem {
            objectName: "rainbow"
            text: qsTr("Rainbow")
            onClicked: root.setVariant("rainbow")
        },
        MenuItem {
            objectName: "content"
            text: qsTr("Content")
            onClicked: root.setVariant("content")
        }
    ]

    Process {
        id: getVariant

        running: true
        command: ["caelestia", "scheme", "get", "-v"]
        stdout: StdioCollector {
            onStreamFinished: {
                const value = text.trim();
                if (value.length > 0)
                    root.currentVariant = value;
            }
        }
    }

    Timer {
        id: refreshTimer

        interval: 250
        repeat: false
        onTriggered: getVariant.running = true
    }

    SectionHeader {
        first: true
        text: qsTr("Vesper appearance")
    }

    SelectRow {
        label: qsTr("Colour scheme")
        subtext: qsTr("Drives Caelestia, GTK, Qt and Hyprland from one palette")
        fallbackIcon: "palette"
        fallbackText: `${Colours.scheme} ${Colours.flavour}`
        menuItems: root.schemeItems
        active: root.schemeItems.find(item => item.objectName === `${Colours.scheme}/${Colours.flavour}`) ?? null
    }

    SelectRow {
        label: qsTr("Material variant")
        subtext: qsTr("Material You palette generation strategy")
        fallbackIcon: "colors"
        fallbackText: root.currentVariant
        menuItems: root.variantItems
        active: root.variantItems.find(item => item.objectName === root.currentVariant) ?? null
    }

    ToggleRow {
        text: qsTr("Glass")
        subtext: qsTr("Caelestia translucency + Hyprland backdrop blur")
        checked: Colours.transparency.enabled
        onToggled: root.setGlass(checked)
    }

    ToggleRow {
        last: true
        text: qsTr("Dark theme")
        subtext: qsTr("One switch for Caelestia, GTK, Qt and Hyprland")
        checked: !Colours.light
        onToggled: root.setMode(checked ? "dark" : "light")
    }

    SectionHeader {
        text: qsTr("Hyprland")
    }

    InfoRow {
        icon: "blur_on"
        label: qsTr("Backdrop blur")
        subtext: qsTr("Compositor blur behind translucent toolkit surfaces")
        value: Colours.transparency.enabled ? qsTr("Enabled") : qsTr("Disabled")
    }

    InfoRow {
        icon: "tune"
        label: qsTr("Glass profile")
        subtext: qsTr("Configured declaratively in Vesper appearance")
        value: "12 px × 4"
    }

    SectionHeader {
        text: qsTr("GTK")
    }

    InfoRow {
        icon: "desktop_windows"
        label: qsTr("GTK theme")
        subtext: qsTr("GTK 3 and GTK 4 receive the active Caelestia palette")
        value: qsTr("Caelestia")
    }

    InfoRow {
        icon: "image"
        label: qsTr("Icon theme")
        subtext: qsTr("Follows the active light/dark mode")
        value: Colours.light ? "Papirus-Light" : "Papirus-Dark"
    }

    RowButton {
        icon: "sync"
        text: qsTr("Reapply GTK theme")
        subtext: qsTr("Regenerate GTK colours from the current Caelestia scheme")
        trailingIcon: "refresh"
        onClicked: root.reapplyToolkitThemes()
    }

    SectionHeader {
        text: qsTr("Qt")
    }

    InfoRow {
        icon: "widgets"
        label: qsTr("Platform theme")
        subtext: qsTr("Native Qt platform integration")
        value: "qtengine"
    }

    InfoRow {
        icon: "brush"
        label: qsTr("Widget style")
        subtext: qsTr("Palette is regenerated by Caelestia")
        value: "Darkly"
    }

    RowButton {
        icon: "sync"
        text: qsTr("Reapply Qt theme")
        subtext: qsTr("Regenerate qtengine colours from the current Caelestia scheme")
        trailingIcon: "refresh"
        onClicked: root.reapplyToolkitThemes()
    }
}
