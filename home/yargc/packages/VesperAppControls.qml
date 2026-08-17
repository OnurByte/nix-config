pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.services
import qs.modules.nexus.common

ColumnLayout {
    id: root

    property var app
    property var status: ({ sandbox: "native", flatpakId: "", permissionsManageable: false, enforcementBackend: "native/unrestricted", permissionItems: [], portalPermissions: "", todaySeconds: 0 })
    property var appWellbeing: ({ todaySeconds: 0, excluded: false, dailyLimitSeconds: 0, overLimit: false, category: "", limitBehavior: "advisory", history: [] })
    property string notificationPolicy: "inherit"
    property string message: ""

    Layout.fillWidth: true
    spacing: Tokens.spacing.extraSmall / 2

    readonly property bool flatpak: status.sandbox === "flatpak"

    function duration(seconds) {
        const minutes = Math.floor((seconds || 0) / 60);
        if (minutes < 60)
            return qsTr("%1 min").arg(minutes);
        return qsTr("%1 h %2 min").arg(Math.floor(minutes / 60)).arg(minutes % 60);
    }

    function packagedLabel(value) {
        if (value === true) return qsTr("allowed");
        if (value === false) return qsTr("not requested");
        return qsTr("unknown");
    }

    function runPermission(command) {
        if (permissionChange.running) return;
        root.message = "";
        permissionChange.command = command;
        permissionChange.running = true;
    }

    function setNotificationPolicy(policy) {
        if (!root.app || notificationChange.running) return;
        root.message = "";
        notificationChange.command = ["@vesperControl@", "notifications", "set", root.app.id, root.app.name || root.app.id, policy];
        notificationChange.running = true;
    }

    function setWellbeing(field, value) {
        if (!root.app || wellbeingChange.running) return;
        root.message = "";
        wellbeingChange.command = ["@vesperControl@", "wellbeing", "app-set", root.app.id, field, value];
        wellbeingChange.running = true;
    }

    function refresh() {
        if (!root.app) return;
        if (!appStatus.running) {
            appStatus.command = ["@vesperControl@", "app-status", root.app.id];
            appStatus.running = true;
        }
        if (!notificationStatus.running) {
            notificationStatus.command = ["@vesperControl@", "notifications", "get", root.app.id];
            notificationStatus.running = true;
        }
        if (!wellbeingStatus.running) {
            wellbeingStatus.command = ["@vesperControl@", "wellbeing", "app", root.app.id];
            wellbeingStatus.running = true;
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
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Could not read app controls");
                }
            }
        }
    }

    Process {
        id: notificationStatus
        stdout: StdioCollector { onStreamFinished: root.notificationPolicy = text.trim() || "inherit" }
    }

    Process {
        id: wellbeingStatus
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.appWellbeing = JSON.parse(text);
                    if (!limitMinutes.activeFocus)
                        limitMinutes.text = root.appWellbeing.dailyLimitSeconds > 0 ? String(Math.round(root.appWellbeing.dailyLimitSeconds / 60)) : "";
                    if (!categoryField.activeFocus)
                        categoryField.text = root.appWellbeing.category || "";
                } catch (e) {
                    root.message = qsTr("Could not read per-app Wellbeing state");
                }
            }
        }
    }

    Process {
        id: notificationChange
        stderr: StdioCollector { id: notificationError }
        onExited: (code, status) => {
            root.message = code === 0 ? "" : notificationError.text.trim();
            root.refresh();
        }
    }

    Process {
        id: wellbeingChange
        stderr: StdioCollector { id: wellbeingError }
        onExited: (code, status) => {
            root.message = code === 0 ? "" : wellbeingError.text.trim();
            root.refresh();
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

    SectionHeader { text: qsTr("Permissions") }

    InfoRow {
        icon: root.flatpak ? "deployed_code" : "warning"
        label: qsTr("Sandbox")
        subtext: root.flatpak ? root.status.flatpakId : qsTr("native Nix app · capability restrictions shown here are informational")
        value: root.flatpak ? qsTr("Flatpak") : qsTr("unrestricted")
        iconColour: root.flatpak ? Colours.palette.m3primary : Colours.palette.m3tertiary
    }

    InfoRow {
        visible: !root.flatpak
        icon: "info"
        label: qsTr("Enforcement backend")
        subtext: qsTr("Vesper does not pretend Flatpak overrides exist for native applications")
        value: root.status.enforcementBackend || qsTr("informational")
    }

    Repeater {
        model: root.flatpak ? (root.status.permissionItems || []) : []
        delegate: ToggleRow {
            required property var modelData
            text: modelData.label
            subtext: qsTr("packaged: %1 · override: %2 · %3")
                .arg(root.packagedLabel(modelData.packaged))
                .arg(modelData.userOverride || qsTr("inherit"))
                .arg(modelData.backend || qsTr("Flatpak-enforced"))
            checked: modelData.effective === true
            disabled: permissionChange.running
            onToggled: root.runPermission(["@vesperControl@", "app-permission", root.app.id, modelData.id, checked ? "on" : "off"])
        }
    }

    SectionHeader { visible: root.flatpak; text: qsTr("Custom filesystem") }

    StyledTextField {
        id: filesystemField
        visible: root.flatpak
        Layout.fillWidth: true
        placeholderText: qsTr("/path, ~/path or xdg-download/subdir")
        leadingIcon: "folder"
        supportingText: qsTr("adds or denies an explicit Flatpak filesystem path")
        inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
    }

    RowLayout {
        visible: root.flatpak
        Layout.fillWidth: true
        spacing: Tokens.spacing.small
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "block"; text: qsTr("Deny")
            disabled: !filesystemField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-filesystem", root.app.id, filesystemField.text.trim(), "off"])
        }
        IconTextButton {
            isRound: true; icon: "check"; text: qsTr("Allow")
            disabled: !filesystemField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-filesystem", root.app.id, filesystemField.text.trim(), "on"])
        }
    }

    SectionHeader { visible: root.flatpak; text: qsTr("D-Bus") }

    StyledTextField {
        id: dbusField
        visible: root.flatpak
        Layout.fillWidth: true
        placeholderText: qsTr("org.example.Service")
        leadingIcon: "cable"
        supportingText: qsTr("well-known name · explicit Flatpak talk/deny override")
        inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
    }

    RowLayout {
        visible: root.flatpak
        Layout.fillWidth: true
        spacing: Tokens.spacing.small
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "block"; text: qsTr("Deny session")
            disabled: !dbusField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-dbus", root.app.id, "session", dbusField.text.trim(), "deny"])
        }
        IconTextButton {
            isRound: true; icon: "check"; text: qsTr("Allow session")
            disabled: !dbusField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-dbus", root.app.id, "session", dbusField.text.trim(), "talk"])
        }
    }

    RowLayout {
        visible: root.flatpak
        Layout.fillWidth: true
        spacing: Tokens.spacing.small
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "block"; text: qsTr("Deny system")
            disabled: !dbusField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-dbus", root.app.id, "system", dbusField.text.trim(), "deny"])
        }
        IconTextButton {
            isRound: true; icon: "check"; text: qsTr("Allow system")
            disabled: !dbusField.text.trim() || permissionChange.running
            onClicked: root.runPermission(["@vesperControl@", "app-dbus", root.app.id, "system", dbusField.text.trim(), "talk"])
        }
    }

    InfoRow {
        visible: root.flatpak
        icon: "shield"
        label: qsTr("Portal permission store")
        subtext: qsTr("camera, location and other portal-mediated decisions stay owned by the portal permission store")
        value: root.status.portalPermissions ? qsTr("entries present") : qsTr("no entries")
    }

    RowButton {
        visible: root.flatpak && root.status.permissionsManageable !== false
        icon: "restart_alt"
        text: qsTr("Reset all Flatpak overrides")
        subtext: qsTr("return this app to its packaged permission defaults")
        disabled: permissionChange.running
        onClicked: root.runPermission(["@vesperControl@", "app-reset-permissions", root.app.id])
    }

    SectionHeader { text: qsTr("Notifications") }

    InfoRow {
        icon: root.notificationPolicy === "block" ? "notifications_off" : "notifications"
        label: qsTr("Notification policy")
        subtext: qsTr("packaging-independent · enforced before Caelestia stores or shows the notification")
        value: root.notificationPolicy
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Tokens.spacing.small
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "notifications_off"; text: qsTr("Block")
            disabled: notificationChange.running || root.notificationPolicy === "block"
            onClicked: root.setNotificationPolicy("block")
        }
        IconTextButton {
            isRound: true; icon: "notifications_active"; text: qsTr("Allow")
            disabled: notificationChange.running || root.notificationPolicy === "allow"
            onClicked: root.setNotificationPolicy("allow")
        }
        IconTextButton {
            isRound: true; icon: "restart_alt"; text: qsTr("Inherit")
            disabled: notificationChange.running || root.notificationPolicy === "inherit"
            onClicked: root.setNotificationPolicy("inherit")
        }
    }

    SectionHeader { text: qsTr("Wellbeing") }

    InfoRow {
        icon: root.appWellbeing.overLimit ? "warning" : "timer"
        label: qsTr("Foreground time today")
        subtext: root.appWellbeing.overLimit
            ? qsTr("advisory daily limit reached; Vesper is not claiming a hard block")
            : qsTr("local only · paused while idle or locked")
        value: root.duration(root.appWellbeing.todaySeconds)
    }

    ToggleRow {
        text: qsTr("Exclude from screen time")
        subtext: qsTr("future foreground samples for this app are not recorded")
        checked: root.appWellbeing.excluded === true
        disabled: wellbeingChange.running
        onToggled: root.setWellbeing("excluded", checked ? "on" : "off")
    }

    StyledTextField {
        id: limitMinutes
        Layout.fillWidth: true
        placeholderText: qsTr("daily limit in minutes; empty = none")
        leadingIcon: "hourglass_top"
        supportingText: qsTr("advisory limit only")
        inputMethodHints: Qt.ImhDigitsOnly
    }

    RowLayout {
        Layout.fillWidth: true
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "save"; text: qsTr("Save limit")
            disabled: wellbeingChange.running
            onClicked: {
                const minutes = Math.max(0, Number(limitMinutes.text || 0));
                root.setWellbeing("limit", String(Math.round(minutes * 60)));
            }
        }
    }

    StyledTextField {
        id: categoryField
        Layout.fillWidth: true
        placeholderText: qsTr("category, e.g. Work or Social")
        leadingIcon: "category"
        supportingText: qsTr("local Wellbeing grouping")
        inputMethodHints: Qt.ImhNoPredictiveText
    }

    RowLayout {
        Layout.fillWidth: true
        Item { Layout.fillWidth: true }
        IconTextButton {
            isRound: true; icon: "save"; text: qsTr("Save category")
            disabled: wellbeingChange.running
            onClicked: root.setWellbeing("category", categoryField.text.trim())
        }
    }

    StyledText {
        Layout.fillWidth: true
        Layout.leftMargin: Tokens.padding.largeIncreased
        visible: root.message
        text: root.message
        color: Colours.palette.m3error
        font: Tokens.font.label.small
        wrapMode: Text.WordWrap
    }
}
