pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Widgets
import Caelestia.Config
import qs.components
import qs.components.containers
import qs.components.controls
import qs.services
import qs.utils
import qs.modules.nexus.common

PageBase {
    id: root

    property string selectedCategory: "all"

    readonly property var categoryOptions: [
        { key: "all", label: qsTr("All") },
        { key: "development", label: qsTr("Development") },
        { key: "internet", label: qsTr("Internet") },
        { key: "office", label: qsTr("Office") },
        { key: "graphics", label: qsTr("Graphics") },
        { key: "media", label: qsTr("Audio & Video") },
        { key: "games", label: qsTr("Games") },
        { key: "utilities", label: qsTr("Utilities") },
        { key: "system", label: qsTr("System") },
        { key: "other", label: qsTr("Other") }
    ]

    readonly property var filteredApps: [...DesktopEntries.applications.values]
        .filter(app => root.matchesCategory(app))
        .sort((a, b) => a.name.localeCompare(b.name))

    title: qsTr("All apps")
    isSubPage: true

    function categoryLabel(key) {
        const item = root.categoryOptions.find(category => category.key === key);
        return item?.label ?? qsTr("All");
    }

    function appCategories(app) {
        return [...(app?.categories ?? [])];
    }

    function matchesAny(categories, values) {
        return values.some(value => categories.includes(value));
    }

    function matchesCategory(app) {
        if (root.selectedCategory === "all")
            return true;

        const categories = root.appCategories(app);
        switch (root.selectedCategory) {
        case "development":
            return root.matchesAny(categories, ["Development"]);
        case "internet":
            return root.matchesAny(categories, ["Network", "WebBrowser", "Email"]);
        case "office":
            return root.matchesAny(categories, ["Office"]);
        case "graphics":
            return root.matchesAny(categories, ["Graphics"]);
        case "media":
            return root.matchesAny(categories, ["AudioVideo", "Audio", "Video"]);
        case "games":
            return root.matchesAny(categories, ["Game"]);
        case "utilities":
            return root.matchesAny(categories, ["Utility"]);
        case "system":
            return root.matchesAny(categories, ["System", "Settings"]);
        case "other":
            return !root.matchesAny(categories, [
                "Development", "Network", "WebBrowser", "Email", "Office", "Graphics",
                "AudioVideo", "Audio", "Video", "Game", "Utility", "System", "Settings"
            ]);
        default:
            return true;
        }
    }

    function commandKey(command) {
        return [...(command ?? [])].join("\u0000");
    }

    function defaultRoles(app) {
        if (!app)
            return [];

        const key = root.commandKey(app.command);
        const defaults = GlobalConfig.general.apps;
        const roles = [];
        if (root.commandKey(defaults.terminal) === key)
            roles.push(qsTr("Terminal"));
        if (root.commandKey(defaults.audio) === key)
            roles.push(qsTr("Audio"));
        if (root.commandKey(defaults.playback) === key)
            roles.push(qsTr("Media"));
        if (root.commandKey(defaults.explorer) === key)
            roles.push(qsTr("Files"));
        return roles;
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader {
            first: true
            text: qsTr("Filter")
        }

        PopupRow {
            id: categoryRow

            readonly property int popupHeight: root.flickable.height - y + root.flickable.contentY - Tokens.padding.large - Tokens.padding.extraExtraLarge

            first: true
            last: true
            icon: "category"
            label: qsTr("Category")
            status: root.categoryLabel(root.selectedCategory)

            Loader {
                anchors.centerIn: parent
                active: categoryRow.popup.animDriver > 0

                sourceComponent: VerticalFadeListView {
                    id: categoriesList

                    implicitWidth: Tokens.sizes.nexus.popupWidth
                    implicitHeight: CUtils.clamp(categoryRow.popupHeight, Tokens.sizes.nexus.minPopupHeight, Tokens.sizes.nexus.maxPopupHeight)
                    model: root.categoryOptions

                    delegate: StateLayer {
                        id: categoryItem

                        required property var modelData
                        required property int index

                        anchors.fill: undefined
                        anchors.left: categoriesList.contentItem.left
                        anchors.right: categoriesList.contentItem.right
                        implicitHeight: categoryLayout.implicitHeight + Tokens.padding.medium * 2
                        radius: Tokens.rounding.small

                        onClicked: {
                            root.selectedCategory = modelData.key;
                            categoryRow.popup.open = false;
                        }

                        RowLayout {
                            id: categoryLayout

                            anchors.fill: parent
                            anchors.margins: Tokens.padding.medium
                            spacing: Tokens.spacing.medium

                            StyledText {
                                Layout.fillWidth: true
                                text: categoryItem.modelData.label
                                font: Tokens.font.body.small
                                elide: Text.ElideRight
                            }

                            MaterialIcon {
                                visible: root.selectedCategory === categoryItem.modelData.key
                                text: "check"
                                color: Colours.palette.m3primary
                                fontStyle: Tokens.font.icon.small
                            }
                        }
                    }
                }
            }
        }

        SectionHeader {
            text: qsTr("Applications")
        }

        Repeater {
            id: list

            model: root.filteredApps

            ConnectedRect {
                id: appItem

                required property DesktopEntry modelData
                required property int index

                readonly property var roles: root.defaultRoles(modelData)

                Layout.fillWidth: true
                first: index === 0
                last: index === list.count - 1
                implicitHeight: appRow.implicitHeight + appRow.anchors.margins * 2

                StateLayer {
                    onClicked: {
                        root.nState.selectedApp = appItem.modelData;
                        root.nState.openSubPage(2);
                    }
                }

                RowLayout {
                    id: appRow

                    anchors.fill: parent
                    anchors.margins: Tokens.padding.medium
                    anchors.leftMargin: Tokens.padding.largeIncreased
                    anchors.rightMargin: Tokens.padding.largeIncreased
                    spacing: Tokens.spacing.medium

                    IconImage {
                        asynchronous: true
                        implicitSize: Math.round(Tokens.font.icon.large.pointSize * 1.8)
                        source: Quickshell.iconPath(appItem.modelData.icon, "image-missing")
                    }

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 0

                        StyledText {
                            Layout.fillWidth: true
                            text: appItem.modelData.name
                            font: Tokens.font.body.small
                            elide: Text.ElideRight
                        }

                        StyledText {
                            Layout.fillWidth: true
                            visible: text
                            text: (appItem.modelData.comment || appItem.modelData.genericName) ?? ""
                            color: Colours.palette.m3outline
                            font: Tokens.font.label.small
                            elide: Text.ElideRight
                        }

                        StyledText {
                            Layout.fillWidth: true
                            visible: appItem.roles.length > 0
                            text: qsTr("Default: %1").arg(appItem.roles.join(", "))
                            color: Colours.palette.m3primary
                            font: Tokens.font.label.small
                            elide: Text.ElideRight
                        }
                    }

                    MaterialIcon {
                        visible: Strings.testRegexList(GlobalConfig.launcher.favouriteApps, appItem.modelData.id)
                        text: "favorite"
                        fill: 1
                        color: Colours.palette.m3primary
                        fontStyle: Tokens.font.icon.small
                    }

                    IconButton {
                        icon: "open_in_new"
                        onClicked: appItem.modelData.execute()
                    }

                    MaterialIcon {
                        text: "chevron_right"
                        color: Colours.palette.m3onSurfaceVariant
                        fontStyle: Tokens.font.icon.medium
                    }
                }
            }
        }

        StyledText {
            Layout.fillWidth: true
            Layout.leftMargin: Tokens.padding.largeIncreased
            Layout.rightMargin: Tokens.padding.largeIncreased
            visible: list.count === 0
            text: qsTr("No installed applications in this category")
            color: Colours.palette.m3outline
            font: Tokens.font.body.small
            wrapMode: Text.WordWrap
        }
    }
}
