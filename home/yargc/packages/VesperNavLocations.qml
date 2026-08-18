pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Caelestia.Config
import qs.components
import qs.components.containers
import qs.services
import qs.modules.nexus

VerticalFadeFlickable {
    id: root

    required property NexusState nState

    function categoryLabel(category) {
        switch (category) {
        case "appearance":
            return qsTr("Personalization");
        case "connectivity":
            return qsTr("Connectivity");
        case "system":
            return qsTr("System");
        case "shell":
            return qsTr("Desktop");
        case "about":
            return qsTr("Information");
        default:
            return qsTr("Settings");
        }
    }

    function categoryIcon(category) {
        switch (category) {
        case "appearance":
            return "palette";
        case "connectivity":
            return "hub";
        case "system":
            return "settings";
        case "shell":
            return "desktop_windows";
        case "about":
            return "info";
        default:
            return "tune";
        }
    }

    topMargin: Tokens.padding.large
    bottomMargin: Tokens.padding.large
    contentHeight: content.implicitHeight

    TapHandler {
        onTapped: root.focus = true
    }

    ColumnLayout {
        id: content

        anchors.left: parent.left
        anchors.right: parent.right
        spacing: Tokens.spacing.extraSmall

        Repeater {
            id: list

            model: PageRegistry.pages

            ColumnLayout {
                id: entry

                required property var modelData
                required property int index

                readonly property bool isCurrentPage: index === root.nState.currentPageIdx
                readonly property bool isCategoryStart: index === 0 || PageRegistry.pages[index - 1].category !== modelData.category
                readonly property bool isCategoryEnd: index === list.model.length - 1 || PageRegistry.pages[index + 1].category !== modelData.category

                Layout.fillWidth: true
                Layout.topMargin: index !== 0 && isCategoryStart ? Tokens.spacing.large : 0
                spacing: Tokens.spacing.extraSmall

                RowLayout {
                    visible: entry.isCategoryStart
                    Layout.fillWidth: true
                    Layout.leftMargin: Tokens.padding.medium
                    Layout.rightMargin: Tokens.padding.small
                    Layout.bottomMargin: Tokens.spacing.extraSmall
                    spacing: Tokens.spacing.small

                    MaterialIcon {
                        text: root.categoryIcon(entry.modelData.category)
                        color: Colours.palette.m3primary
                        fontStyle: Tokens.font.icon.medium
                        fill: 0
                        grade: 25
                    }

                    StyledText {
                        Layout.fillWidth: true
                        text: root.categoryLabel(entry.modelData.category)
                        color: Colours.palette.m3onSurfaceVariant
                        font: Tokens.font.label.medium
                        elide: Text.ElideRight
                    }
                }

                StyledRect {
                    id: item

                    Layout.fillWidth: true
                    implicitHeight: {
                        const h = layout.implicitHeight + Tokens.padding.large * 2;
                        return h % 2 === 0 ? h : h + 1;
                    }

                    color: entry.isCurrentPage ? Colours.palette.m3secondaryContainer : Colours.layer(Colours.palette.m3surfaceContainerHigh, 2)

                    topLeftRadius: stateLayer.pressed ? Tokens.rounding.medium : entry.isCurrentPage ? Tokens.rounding.extraLargeIncreased : entry.isCategoryStart ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                    topRightRadius: stateLayer.pressed ? Tokens.rounding.medium : entry.isCurrentPage ? Tokens.rounding.extraLargeIncreased : entry.isCategoryStart ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                    bottomLeftRadius: stateLayer.pressed ? Tokens.rounding.medium : entry.isCurrentPage ? Tokens.rounding.extraLargeIncreased : entry.isCategoryEnd ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                    bottomRightRadius: stateLayer.pressed ? Tokens.rounding.medium : entry.isCurrentPage ? Tokens.rounding.extraLargeIncreased : entry.isCategoryEnd ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall

                    RadiusBehavior on topLeftRadius {}
                    RadiusBehavior on topRightRadius {}
                    RadiusBehavior on bottomLeftRadius {}
                    RadiusBehavior on bottomRightRadius {}

                    StateLayer {
                        id: stateLayer

                        anchors.fill: parent
                        topLeftRadius: parent.topLeftRadius
                        topRightRadius: parent.topRightRadius
                        bottomLeftRadius: parent.bottomLeftRadius
                        bottomRightRadius: parent.bottomRightRadius

                        onClicked: root.nState.currentPageIdx = entry.index
                    }

                    RowLayout {
                        id: layout

                        anchors.fill: parent
                        anchors.margins: Tokens.padding.large
                        anchors.leftMargin: Tokens.padding.large + Tokens.padding.medium
                        spacing: Tokens.spacing.medium

                        StyledRect {
                            Layout.fillHeight: true
                            Layout.topMargin: -1
                            Layout.bottomMargin: -1
                            implicitWidth: height

                            radius: Tokens.rounding.full
                            color: entry.isCurrentPage ? Colours.palette.m3primary : Colours.palette.m3secondaryContainer

                            MaterialIcon {
                                anchors.centerIn: parent
                                anchors.verticalCenterOffset: 1

                                text: entry.modelData.icon
                                color: entry.isCurrentPage ? Colours.palette.m3onPrimary : Colours.palette.m3onSecondaryContainer
                                fontStyle: Tokens.font.icon.builders.medium.weight(Font.Medium).build()
                                grade: 25
                                fill: entry.modelData.noFill ? 0 : 1
                            }
                        }

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 0

                            StyledText {
                                Layout.fillWidth: true
                                text: entry.modelData.label
                                font: Tokens.font.body.medium
                                elide: Text.ElideRight
                            }

                            StyledText {
                                Layout.fillWidth: true
                                text: entry.modelData.description
                                color: Colours.palette.m3onSurfaceVariant
                                font: Tokens.font.label.small
                                elide: Text.ElideRight
                            }
                        }
                    }
                }
            }
        }
    }

    component RadiusBehavior: Behavior {
        Anim {
            type: Anim.DefaultEffects
        }
    }
}
