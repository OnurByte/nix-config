pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import qs.modules.nexus.common

PageBase {
    id: root

    title: qsTr("Clipboard")

    property var entries: []
    property string selectedId: ""

    function parseHistory(raw: string): var {
        if (!raw.trim())
            return [];

        return raw.split("\n").filter(line => line.length > 0).slice(0, 40).map(line => {
            const tab = line.indexOf("\t");
            if (tab < 0)
                return { id: "", preview: line };
            return {
                id: line.slice(0, tab),
                preview: line.slice(tab + 1).replace(/\s+/g, " ").trim()
            };
        }).filter(entry => entry.id.length > 0);
    }

    function reload(): void {
        historyProc.running = true;
    }

    function copyEntry(id: string): void {
        root.selectedId = id;
        Quickshell.execDetached(["@vesperSettings@", "clipboard", "copy", id]);
    }

    function deleteSelected(): void {
        if (!root.selectedId)
            return;
        Quickshell.execDetached(["@vesperSettings@", "clipboard", "delete", root.selectedId]);
        root.selectedId = "";
        refreshTimer.restart();
    }

    Component.onCompleted: reload()

    Process {
        id: historyProc
        command: ["@vesperSettings@", "clipboard", "list"]
        stdout: StdioCollector {
            onStreamFinished: root.entries = root.parseHistory(text)
        }
    }

    Timer {
        id: refreshTimer
        interval: 250
        repeat: false
        onTriggered: root.reload()
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader {
            first: true
            text: qsTr("Recent history")
        }

        InfoRow {
            visible: root.entries.length === 0
            icon: "content_paste_off"
            label: qsTr("Clipboard history is empty")
            subtext: qsTr("Copied text and images will appear here")
            value: ""
        }

        Repeater {
            model: root.entries

            RowButton {
                required property var modelData
                required property int index

                first: index === 0
                last: index === root.entries.length - 1
                icon: root.selectedId === modelData.id ? "check" : "content_paste"
                text: modelData.preview || qsTr("Binary or image clipboard entry")
                subtext: qsTr("Entry %1 · click to copy").arg(modelData.id)
                trailingIcon: "content_copy"
                onClicked: root.copyEntry(modelData.id)
            }
        }

        SectionHeader {
            text: qsTr("History actions")
        }

        RowButton {
            icon: "delete"
            text: qsTr("Delete selected entry")
            subtext: root.selectedId ? qsTr("Selected entry %1").arg(root.selectedId) : qsTr("Select an entry above first")
            disabled: !root.selectedId
            onClicked: root.deleteSelected()
        }

        RowButton {
            icon: "compress"
            text: qsTr("Compact history")
            subtext: qsTr("Deduplicate and shrink the cliphist database")
            onClicked: {
                Quickshell.execDetached(["@vesperSettings@", "clipboard", "compact"]);
                refreshTimer.restart();
            }
        }

        RowButton {
            icon: "delete_sweep"
            text: qsTr("Clear all history")
            subtext: qsTr("Wipe the local clipboard history database")
            onClicked: {
                Quickshell.execDetached(["@vesperSettings@", "clipboard", "wipe"]);
                root.selectedId = "";
                refreshTimer.restart();
            }
        }

        RowButton {
            last: true
            icon: "refresh"
            text: qsTr("Refresh")
            onClicked: root.reload()
        }
    }
}
