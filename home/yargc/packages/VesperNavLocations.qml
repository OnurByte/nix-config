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

    // Keep navigation hierarchy independent from upstream registry order. Pages
    // are resolved by their stable registry icon, so visual ordering can follow
    // how people actually look for settings without breaking page indices.
    readonly property var sections: [
        {
            label: qsTr("Network & Devices"),
            icon: "hub",
            pages: [
                {
                    pageIcon: "wifi",
                    label: qsTr("Network"),
                    icon: "wifi",
                    description: qsTr("Wi-Fi, ethernet, VPN, proxy")
                },
                {
                    pageIcon: "devices_other",
                    label: qsTr("Bluetooth"),
                    icon: "bluetooth",
                    description: qsTr("Devices, pairing, discoverability")
                },
                {
                    pageIcon: "volume_up",
                    label: qsTr("Audio"),
                    icon: "volume_up",
                    description: qsTr("Output, input, app volumes")
                }
            ]
        },
        {
            label: qsTr("Personalization"),
            icon: "palette",
            pages: [
                {
                    pageIcon: "palette",
                    label: qsTr("Appearance"),
                    icon: "palette",
                    description: qsTr("Wallpaper, colours, icons")
                },
                {
                    pageIcon: "search",
                    label: qsTr("Vicinae"),
                    icon: "search",
                    description: qsTr("Spotlight launcher, theme, Vesper controls")
                },
                {
                    pageIcon: "dock_to_bottom",
                    label: qsTr("Panels"),
                    icon: "dock_to_bottom",
                    description: qsTr("Taskbar, dashboard, launcher, sidebar")
                }
            ]
        },
        {
            label: qsTr("Apps & AI"),
            icon: "apps",
            pages: [
                {
                    pageIcon: "apps",
                    label: qsTr("Apps"),
                    icon: "apps",
                    description: qsTr("Defaults, permissions, wellbeing, icons")
                },
                {
                    pageIcon: "smart_toy",
                    label: qsTr("AI"),
                    icon: "smart_toy",
                    description: qsTr("Models, API keys, agents, skills, MCP, Hermes")
                }
            ]
        },
        {
            label: qsTr("System"),
            icon: "settings",
            pages: [
                {
                    pageIcon: "globe",
                    label: qsTr("Language & region"),
                    icon: "globe",
                    description: qsTr("Language, location, units")
                },
                {
                    pageIcon: "build",
                    label: qsTr("Services"),
                    icon: "build",
                    description: qsTr("Notifications, polling, integrations")
                },
                {
                    pageIcon: "update",
                    label: qsTr("Updates"),
                    icon: "update",
                    description: qsTr("System and component updates")
                }
            ]
        },
        {
            label: qsTr("About"),
            icon: "info",
            pages: [
                {
                    pageIcon: "info",
                    label: qsTr("About"),
                    icon: "info",
                    description: qsTr("System information and credits")
                }
            ]
        }
    ]

    function pageIndexFor(pageIcon) {
        for (let i = 0; i < PageRegistry.pages.length; ++i) {
            if (PageRegistry.pages[i].icon === pageIcon)
                return i;
        }
        return -1;
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
        spacing: Tokens.spacing.large

        Repeater {
            model: root.sections

            ColumnLayout {
                id: section

                required property var modelData
                required property int index

                Layout.fillWidth: true
                spacing: Tokens.spacing.extraSmall

                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: Tokens.padding.medium
                    Layout.rightMargin: Tokens.padding.small
                    Layout.bottomMargin: Tokens.spacing.extraSmall
                    spacing: Tokens.spacing.small

                    StyledRect {
                        implicitWidth: sectionIcon.implicitWidth + Tokens.padding.small * 2
                        implicitHeight: sectionIcon.implicitHeight + Tokens.padding.small * 2
                        radius: Tokens.rounding.full
                        color: Colours.layer(Colours.palette.m3secondaryContainer, 2)

                        MaterialIcon {
                            id: sectionIcon
                            anchors.centerIn: parent
                            text: section.modelData.icon
                            color: Colours.palette.m3primary
                            fontStyle: Tokens.font.icon.small
                            fill: 0
                            grade: 25
                        }
                    }

                    StyledText {
                        Layout.fillWidth: true
                        text: section.modelData.label
                        color: Colours.palette.m3onSurfaceVariant
                        font: Tokens.font.label.medium
                        elide: Text.ElideRight
                    }
                }

                Repeater {
                    id: pageList
                    model: section.modelData.pages

                    StyledRect {
                        id: item

                        required property var modelData
                        required property int index

                        readonly property int pageIndex: root.pageIndexFor(modelData.pageIcon)
                        readonly property bool isCurrentPage: pageIndex === root.nState.currentPageIdx
                        readonly property bool isFirst: index === 0
                        readonly property bool isLast: index === pageList.count - 1

                        visible: pageIndex >= 0
                        Layout.fillWidth: true
                        implicitHeight: {
                            const h = layout.implicitHeight + Tokens.padding.large * 2;
                            return h % 2 === 0 ? h : h + 1;
                        }

                        color: isCurrentPage ? Colours.palette.m3secondaryContainer : Colours.layer(Colours.palette.m3surfaceContainerHigh, 2)

                        topLeftRadius: stateLayer.pressed ? Tokens.rounding.medium : isCurrentPage ? Tokens.rounding.extraLargeIncreased : isFirst ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                        topRightRadius: stateLayer.pressed ? Tokens.rounding.medium : isCurrentPage ? Tokens.rounding.extraLargeIncreased : isFirst ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                        bottomLeftRadius: stateLayer.pressed ? Tokens.rounding.medium : isCurrentPage ? Tokens.rounding.extraLargeIncreased : isLast ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall
                        bottomRightRadius: stateLayer.pressed ? Tokens.rounding.medium : isCurrentPage ? Tokens.rounding.extraLargeIncreased : isLast ? Tokens.rounding.extraLarge : Tokens.rounding.extraSmall

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

                            onClicked: {
                                if (item.pageIndex >= 0)
                                    root.nState.currentPageIdx = item.pageIndex;
                            }
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
                                color: item.isCurrentPage ? Colours.palette.m3primary : Colours.palette.m3secondaryContainer

                                MaterialIcon {
                                    anchors.centerIn: parent
                                    anchors.verticalCenterOffset: 1

                                    text: item.modelData.icon
                                    color: item.isCurrentPage ? Colours.palette.m3onPrimary : Colours.palette.m3onSecondaryContainer
                                    fontStyle: Tokens.font.icon.builders.medium.weight(Font.Medium).build()
                                    grade: 25
                                    fill: item.isCurrentPage ? 1 : 0
                                }
                            }

                            ColumnLayout {
                                Layout.fillWidth: true
                                spacing: 0

                                StyledText {
                                    Layout.fillWidth: true
                                    text: item.modelData.label
                                    font: Tokens.font.body.medium
                                    elide: Text.ElideRight
                                }

                                StyledText {
                                    Layout.fillWidth: true
                                    text: item.modelData.description
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
    }

    component RadiusBehavior: Behavior {
        Anim {
            type: Anim.DefaultEffects
        }
    }
}
