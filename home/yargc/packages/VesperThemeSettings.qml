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
    property var icons: ({ enabled: false, mode: "original", material: "standard", followPalette: true, theme: "Vesper-Adaptive", active: 0, canonical: 0, discovered: 0, schemeMode: "dark", accent: "" })
    property string iconMessage: ""

    function setScheme(name: string, flavour: string): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-n", name, "-f", flavour]);
        refreshTimer.restart();
    }

    function setVariant(variant: string): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-v", variant]);
        currentVariant = variant;
        refreshTimer.restart();
    }

    function reapplyToolkitThemes(): void {
        Quickshell.execDetached(["caelestia", "scheme", "set", "--notify", "-m", Colours.light ? "light" : "dark"]);
        refreshTimer.restart();
    }

    function refreshIcons(): void {
        if (!iconStatus.running)
            iconStatus.running = true;
    }

    function runIcon(args): void {
        if (iconChange.running)
            return;
        root.iconMessage = "";
        iconChange.command = ["@vesperControl@", "icon"].concat(args);
        iconChange.running = true;
    }

    Component.onCompleted: refreshIcons()

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

    property list<MenuItem> iconModeItems: [
        MenuItem {
            objectName: "original"
            text: qsTr("Original")
            icon: "image"
            onClicked: root.runIcon(["mode", "original"])
        },
        MenuItem {
            objectName: "light"
            text: qsTr("Light")
            icon: "light_mode"
            onClicked: root.runIcon(["mode", "light"])
        },
        MenuItem {
            objectName: "dark"
            text: qsTr("Dark")
            icon: "dark_mode"
            onClicked: root.runIcon(["mode", "dark"])
        },
        MenuItem {
            objectName: "tinted"
            text: qsTr("Tinted")
            icon: "palette"
            onClicked: root.runIcon(["mode", "tinted"])
        },
        MenuItem {
            objectName: "clear"
            text: qsTr("Clear")
            icon: "blur_on"
            onClicked: root.runIcon(["mode", "clear"])
        }
    ]

    property list<MenuItem> iconMaterialItems: [
        MenuItem {
            objectName: "standard"
            text: qsTr("Standard")
            icon: "layers"
            onClicked: root.runIcon(["material", "standard"])
        },
        MenuItem {
            objectName: "glass"
            text: qsTr("Glass")
            icon: "auto_awesome"
            onClicked: root.runIcon(["material", "glass"])
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

    Process {
        id: iconStatus
        command: ["@vesperControl@", "icon", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.icons = JSON.parse(text);
                } catch (e) {
                    root.iconMessage = qsTr("Could not read adaptive icon theme status");
                }
            }
        }
    }

    Process {
        id: iconChange
        stderr: StdioCollector { id: iconError }
        onExited: (code, status) => {
            root.iconMessage = code === 0 ? "" : iconError.text.trim();
            root.refreshIcons();
        }
    }

    Timer {
        id: refreshTimer

        interval: 250
        repeat: false
        onTriggered: {
            getVariant.running = true;
            root.refreshIcons();
        }
    }

    Timer {
        interval: 30000
        repeat: true
        running: root.visible
        onTriggered: root.refreshIcons()
    }

    SectionHeader {
        first: true
        text: qsTr("Caelestia")
    }

    SelectRow {
        label: qsTr("Colour scheme")
        subtext: qsTr("Drives Caelestia, GTK and Qt from one palette")
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
        text: qsTr("Transparency")
        subtext: qsTr("Base %1, layers %2").arg(Colours.transparency.base).arg(Colours.transparency.layers)
        checked: Colours.transparency.enabled
        onToggled: GlobalConfig.appearance.transparency.enabled = checked
    }

    ToggleRow {
        last: true
        text: qsTr("Dark theme")
        subtext: qsTr("Propagates to GTK and Qt")
        checked: !Colours.light
        onToggled: Colours.setMode(checked ? "dark" : "light")
    }

    SectionHeader {
        text: qsTr("Application icons")
    }

    SelectRow {
        label: qsTr("Appearance")
        subtext: qsTr("recompiles canonical assets locally without another AI request")
        fallbackIcon: "palette"
        fallbackText: root.icons.mode || "original"
        menuItems: root.iconModeItems
        active: root.iconModeItems.find(item => item.objectName === root.icons.mode) ?? null
    }

    SelectRow {
        label: qsTr("Material")
        subtext: qsTr("Glass changes rendering only and never re-canonicalizes the icon")
        fallbackIcon: "layers"
        fallbackText: root.icons.material || "standard"
        menuItems: root.iconMaterialItems
        active: root.iconMaterialItems.find(item => item.objectName === (root.icons.material || "standard")) ?? null
    }

    ToggleRow {
        text: qsTr("Follow Caelestia palette")
        subtext: qsTr("Tinted and Glass consume the current generated accent")
        checked: root.icons.followPalette ?? true
        disabled: iconChange.running
        onToggled: root.runIcon(["follow-palette", checked ? "on" : "off"])
    }

    InfoRow {
        icon: "apps"
        label: qsTr("Generated icon theme")
        subtext: root.icons.enabled ? qsTr("%1 canonical · %2 active").arg(root.icons.canonical || 0).arg(root.icons.active || 0) : qsTr("disabled · inherited Papirus fallback stays active")
        value: root.icons.theme || "Vesper-Adaptive"
    }

    InfoRow {
        icon: "colors"
        label: qsTr("Compiler palette")
        subtext: qsTr("scheme mode %1").arg(root.icons.schemeMode || (Colours.light ? "light" : "dark"))
        value: root.icons.accent || qsTr("pending")
    }

    RowButton {
        icon: "sync"
        text: qsTr("Rebuild application icon theme")
        subtext: qsTr("rescan sources and atomically replace the generated freedesktop theme")
        trailingIcon: "refresh"
        disabled: iconChange.running
        onClicked: root.runIcon(["reconcile"])
    }

    StyledText {
        Layout.fillWidth: true
        visible: root.iconMessage
        text: root.iconMessage
        color: Colours.palette.m3error
        font: Tokens.font.body.small
        wrapMode: Text.WordWrap
    }

    SectionHeader {
        text: qsTr("GTK")
    }

    InfoRow {
        icon: "desktop_windows"
        label: qsTr("GTK theme")
        subtext: qsTr("GTK 3 and GTK 4 receive Caelestia generated CSS")
        value: qsTr("Caelestia")
    }

    InfoRow {
        icon: "image"
        label: qsTr("Icon theme")
        subtext: qsTr("Vesper application overrides with Papirus inheritance for everything else")
        value: root.icons.theme || "Vesper-Adaptive"
    }

    RowButton {
        icon: "sync"
        text: qsTr("Reapply GTK theme")
        subtext: qsTr("Regenerate GTK 3/4 colours from the current Caelestia scheme")
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
