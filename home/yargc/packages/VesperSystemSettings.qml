pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Caelestia.Config
import qs.services
import qs.modules.nexus.common

PageBase {
    id: root

    title: qsTr("System controls")

    property string displayInfo: qsTr("Detecting display…")
    property real brightness: 50
    property string clipboardCount: "0"
    property string browserDefault: qsTr("Detecting…")
    property string fileDefault: qsTr("Detecting…")
    property string mediaDefault: qsTr("Detecting…")
    property string batteryInfo: qsTr("Detecting…")
    property string powerProfile: qsTr("Detecting…")
    property real pointerSensitivity: 0
    property bool naturalScroll: false
    property string wellbeingStatus: "paused"
    property string wellbeingToday: qsTr("No data yet")
    property string wellbeingTopApps: qsTr("No activity yet")

    function exec(parts: list<string>): void {
        Quickshell.execDetached(["@vesperSettings@", ...parts]);
        refreshTimer.restart();
    }

    function formatDuration(milliseconds: real): string {
        const minutes = Math.max(0, Math.round(milliseconds / 60000));
        const hours = Math.floor(minutes / 60);
        const remainder = minutes % 60;
        if (hours > 0)
            return `${hours}h ${remainder}m`;
        return `${remainder}m`;
    }

    function refresh(): void {
        displayProc.running = true;
        brightnessProc.running = true;
        clipboardProc.running = true;
        browserProc.running = true;
        fileProc.running = true;
        mediaProc.running = true;
        batteryProc.running = true;
        powerProc.running = true;
        sensitivityProc.running = true;
        naturalScrollProc.running = true;
        wellbeingProc.running = true;
        wellbeingReportProc.running = true;
    }

    Component.onCompleted: refresh()

    Timer {
        id: refreshTimer
        interval: 350
        repeat: false
        onTriggered: root.refresh()
    }

    Process {
        id: displayProc
        command: ["@vesperSettings@", "display", "info"]
        stdout: StdioCollector {
            onStreamFinished: root.displayInfo = text.trim() || qsTr("Unavailable")
        }
    }

    Process {
        id: brightnessProc
        command: ["@vesperSettings@", "brightness", "get"]
        stdout: StdioCollector {
            onStreamFinished: {
                const value = Number(text.trim());
                if (!Number.isNaN(value))
                    root.brightness = value;
            }
        }
    }

    Process {
        id: clipboardProc
        command: ["@vesperSettings@", "clipboard", "count"]
        stdout: StdioCollector {
            onStreamFinished: root.clipboardCount = text.trim() || "0"
        }
    }

    Process {
        id: browserProc
        command: ["@vesperSettings@", "defaults", "get", "web"]
        stdout: StdioCollector {
            onStreamFinished: root.browserDefault = text.trim() || qsTr("Not set")
        }
    }

    Process {
        id: fileProc
        command: ["@vesperSettings@", "defaults", "get", "file"]
        stdout: StdioCollector {
            onStreamFinished: root.fileDefault = text.trim() || qsTr("Not set")
        }
    }

    Process {
        id: mediaProc
        command: ["@vesperSettings@", "defaults", "get", "video"]
        stdout: StdioCollector {
            onStreamFinished: root.mediaDefault = text.trim() || qsTr("Not set")
        }
    }

    Process {
        id: batteryProc
        command: ["@vesperSettings@", "battery", "info"]
        stdout: StdioCollector {
            onStreamFinished: root.batteryInfo = text.trim() || qsTr("No battery")
        }
    }

    Process {
        id: powerProc
        command: ["@vesperSettings@", "power", "get"]
        stdout: StdioCollector {
            onStreamFinished: root.powerProfile = text.trim() || qsTr("Unavailable")
        }
    }

    Process {
        id: sensitivityProc
        command: ["@vesperSettings@", "input", "sensitivity"]
        stdout: StdioCollector {
            onStreamFinished: {
                const value = Number(text.trim());
                if (!Number.isNaN(value))
                    root.pointerSensitivity = value;
            }
        }
    }

    Process {
        id: naturalScrollProc
        command: ["@vesperSettings@", "input", "get-natural-scroll"]
        stdout: StdioCollector {
            onStreamFinished: root.naturalScroll = text.trim() === "true"
        }
    }

    Process {
        id: wellbeingProc
        command: ["@vesperSettings@", "wellbeing", "status"]
        stdout: StdioCollector {
            onStreamFinished: root.wellbeingStatus = text.trim() || "paused"
        }
    }

    Process {
        id: wellbeingReportProc
        command: ["@vesperSettings@", "wellbeing", "report"]
        stdout: StdioCollector {
            onStreamFinished: {
                const raw = text.trim();
                if (!raw) {
                    root.wellbeingToday = qsTr("No data yet");
                    root.wellbeingTopApps = qsTr("No activity yet");
                    return;
                }

                try {
                    const entries = JSON.parse(raw);
                    if (!Array.isArray(entries) || entries.length === 0) {
                        root.wellbeingToday = qsTr("No data yet");
                        root.wellbeingTopApps = qsTr("No activity yet");
                        return;
                    }

                    let total = 0;
                    for (const entry of entries)
                        total += Number(entry.time_ms ?? 0);

                    root.wellbeingToday = root.formatDuration(total);
                    root.wellbeingTopApps = entries.slice(0, 3).map(entry =>
                        `${entry.name}: ${root.formatDuration(Number(entry.time_ms ?? 0))}`
                    ).join(" · ");
                } catch (error) {
                    root.wellbeingToday = qsTr("Unavailable");
                    root.wellbeingTopApps = qsTr("Could not parse local report");
                }
            }
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader {
            first: true
            text: qsTr("Display & brightness")
        }

        InfoRow {
            icon: "monitor"
            label: qsTr("Focused display")
            subtext: qsTr("Use the Display page for arrangement, modes, scale, rotation and mirroring")
            value: root.displayInfo
        }

        SliderRow {
            icon: "brightness_6"
            label: qsTr("Brightness")
            valueLabel: Math.round(root.brightness) + "%"
            value: root.brightness / 100
            onMoved: value => root.exec(["brightness", "set", String(Math.max(1, Math.round(value * 100)))])
        }

        SectionHeader {
            text: qsTr("Clipboard")
        }

        InfoRow {
            icon: "content_paste"
            label: qsTr("History")
            subtext: qsTr("Local text and image history from cliphist")
            value: qsTr("%1 entries").arg(root.clipboardCount)
        }

        RowButton {
            icon: "content_paste_search"
            text: qsTr("Open clipboard manager")
            subtext: qsTr("Caelestia clipboard picker")
            onClicked: Quickshell.execDetached(["caelestia", "clipboard"])
        }

        RowButton {
            icon: "compress"
            text: qsTr("Compact history")
            subtext: qsTr("Deduplicate and shrink the cliphist database")
            onClicked: root.exec(["clipboard", "compact"])
        }

        RowButton {
            last: true
            icon: "delete_sweep"
            text: qsTr("Clear clipboard history")
            onClicked: root.exec(["clipboard", "wipe"])
        }

        SectionHeader {
            text: qsTr("System default apps")
        }

        InfoRow {
            icon: "language"
            label: qsTr("Browser")
            subtext: qsTr("Freedesktop HTTP/HTML handlers")
            value: root.browserDefault
        }

        InfoRow {
            icon: "folder"
            label: qsTr("File manager")
            subtext: qsTr("inode/directory handler")
            value: root.fileDefault
        }

        InfoRow {
            icon: "movie"
            label: qsTr("Video")
            subtext: qsTr("Freedesktop MIME handler")
            value: root.mediaDefault
        }

        InfoRow {
            icon: "apps"
            label: qsTr("Change defaults")
            subtext: qsTr("Use the Apps page; browser, media and file-manager choices write real XDG defaults")
            value: qsTr("Apps")
        }

        SectionHeader {
            text: qsTr("Power & battery")
        }

        InfoRow {
            icon: "battery_full"
            label: qsTr("Battery")
            subtext: qsTr("Charge, state, remaining time and health from UPower")
            value: root.batteryInfo
        }

        InfoRow {
            icon: "speed"
            label: qsTr("Power profile")
            subtext: qsTr("power-profiles-daemon")
            value: root.powerProfile
        }

        RowButton {
            icon: "battery_saver"
            text: qsTr("Power saver")
            onClicked: root.exec(["power", "set", "power-saver"])
        }

        RowButton {
            icon: "balance"
            text: qsTr("Balanced")
            onClicked: root.exec(["power", "set", "balanced"])
        }

        RowButton {
            last: true
            icon: "bolt"
            text: qsTr("Performance")
            onClicked: root.exec(["power", "set", "performance"])
        }

        SectionHeader {
            text: qsTr("Input")
        }

        SliderRow {
            icon: "mouse"
            label: qsTr("Pointer sensitivity")
            valueLabel: root.pointerSensitivity.toFixed(2)
            value: (root.pointerSensitivity + 1) / 2
            onMoved: value => root.exec(["input", "set-sensitivity", String((value * 2 - 1).toFixed(2))])
        }

        ToggleRow {
            last: true
            text: qsTr("Natural scrolling")
            subtext: qsTr("Touchpad setting applied live through Hyprland")
            checked: root.naturalScroll
            onToggled: root.exec(["input", "set-natural-scroll", checked ? "true" : "false"])
        }

        SectionHeader {
            text: qsTr("Digital wellbeing")
        }

        ToggleRow {
            text: qsTr("Activity tracking")
            subtext: qsTr("Local Hyprland app-time tracker; no cloud or browser dashboard")
            checked: root.wellbeingStatus === "active"
            onToggled: root.exec(["wellbeing", checked ? "enable" : "disable"])
        }

        InfoRow {
            icon: "schedule"
            label: qsTr("Screen time today")
            subtext: qsTr("Active application time recorded locally")
            value: root.wellbeingToday
        }

        InfoRow {
            icon: "leaderboard"
            label: qsTr("Top apps today")
            subtext: qsTr("Top three by active-window time")
            value: root.wellbeingTopApps
        }

        RowButton {
            last: true
            icon: "refresh"
            text: qsTr("Refresh system state")
            onClicked: root.refresh()
        }
    }
}
