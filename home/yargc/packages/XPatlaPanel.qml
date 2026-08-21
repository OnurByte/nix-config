import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import qs.components
import qs.services

Item {
    id: root

    property var payload: ({
        "summary": {
            "sourcesConfigured": 0,
            "sourcesObserved": 0,
            "postsObserved": 0,
            "postsLast24h": 0,
            "mediaAssets": 0,
            "opportunities": 0,
            "lastRun": null
        }
    })
    property string loadError: ""
    readonly property var summary: payload.summary || ({
    })
    readonly property var lastRun: summary.lastRun || null
    readonly property var recentPosts: summary.recentPosts || []
    readonly property string followersLabel: summary.followersTotal === null || summary.followersTotal === undefined ? "followers unknown" : `${summary.followersTotal} source followers (sum)`
    readonly property color stateColour: !lastRun ? Colours.palette.m3outline : lastRun.status === "ok" ? Colours.palette.m3primary : lastRun.status === "partial" ? Colours.palette.m3tertiary : Colours.palette.m3error

    function refresh() {
        if (!status.running)
            status.running = true;

    }

    function applyPayload(text) {
        try {
            root.payload = JSON.parse(text);
            root.loadError = "";
        } catch (e) {
            root.loadError = "XPatla status data unavailable";
        }
    }

    function runDetail() {
        if (root.loadError)
            return root.loadError;

        if (!root.lastRun)
            return "no FxTwitter scan has completed";

        return `${root.lastRun.status || "unknown"} · ${root.lastRun.postsNew || 0} new · ${root.lastRun.postsSeen || 0} seen`;
    }

    implicitWidth: 760
    implicitHeight: 520
    Component.onCompleted: refresh()

    Timer {
        interval: 60000
        repeat: true
        running: true
        onTriggered: root.refresh()
    }

    Process {
        id: status

        command: ["@xpatla@", "status", "--json"]

        stdout: StdioCollector {
            onStreamFinished: root.applyPayload(text)
        }

    }

    RowLayout {
        id: header

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        spacing: Tokens.spacing.medium

        MaterialIcon {
            text: "newspaper"
            color: root.stateColour
            fill: 1
            fontStyle: Tokens.font.icon.large
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0

            StyledText {
                text: qsTr("X news")
                font: Tokens.font.title.large
                color: Colours.palette.m3onSurface
            }

            StyledText {
                Layout.fillWidth: true
                text: root.runDetail()
                elide: Text.ElideRight
                color: root.loadError ? Colours.palette.m3error : Colours.palette.m3onSurfaceVariant
                font: Tokens.font.body.small
            }

        }

        StyledRect {
            implicitWidth: refreshIcon.implicitWidth + Tokens.padding.medium * 2
            implicitHeight: refreshIcon.implicitHeight + Tokens.padding.medium * 2
            radius: Tokens.rounding.full
            color: Qt.alpha(Colours.tPalette.m3surfaceContainerHigh, refreshMouse.containsMouse ? 0.72 : 0.38)

            MaterialIcon {
                id: refreshIcon

                anchors.centerIn: parent
                text: "refresh"
                color: Colours.palette.m3primary
                fontStyle: Tokens.font.icon.medium
            }

            MouseArea {
                id: refreshMouse

                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.refresh()
            }

        }

    }

    GridLayout {
        id: summaryGrid

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: header.bottom
        anchors.topMargin: Tokens.spacing.large
        columns: 2
        columnSpacing: Tokens.spacing.medium
        rowSpacing: Tokens.spacing.medium

        SummaryCard {
            Layout.fillWidth: true
            iconName: "account_balance"
            title: qsTr("Sources")
            value: `${root.summary.sourcesConfigured || 0}`
            detail: `${root.summary.sourcesObserved || 0} observed · ${root.followersLabel}`
            accent: Colours.palette.m3primary
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "dynamic_feed"
            title: qsTr("Observed")
            value: `${root.summary.postsObserved || 0}`
            detail: `${root.summary.postsLast24h || 0} in the last 24 hours`
            accent: Colours.palette.m3secondary
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "photo_library"
            title: qsTr("Media")
            value: `${root.summary.mediaAssets || 0}`
            detail: "FxTwitter photo/video provenance records"
            accent: Colours.palette.m3tertiary
        }

        SummaryCard {
            Layout.fillWidth: true
            iconName: "bolt"
            title: qsTr("Opportunities")
            value: `${root.summary.opportunities || 0}`
            detail: "ranked candidate posts"
            accent: root.stateColour
        }

    }

    StyledRect {
        id: detailPanel

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: summaryGrid.bottom
        anchors.topMargin: Tokens.spacing.large
        implicitHeight: detailColumn.implicitHeight + Tokens.padding.large * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer
        border.width: 1
        border.color: Qt.alpha(root.stateColour, 0.18)

        ColumnLayout {
            id: detailColumn

            anchors.fill: parent
            anchors.margins: Tokens.padding.large
            spacing: Tokens.spacing.small

            StyledText {
                text: qsTr("FxTwitter-only pipeline")
                color: Colours.palette.m3onSurface
                font: Tokens.font.title.small
            }

            StyledText {
                Layout.fillWidth: true
                text: "Dynamic source config · no X mirrors · media cache and provenance stay local"
                color: Colours.palette.m3onSurfaceVariant
                font: Tokens.font.body.medium
                wrapMode: Text.WordWrap
            }

            StyledText {
                Layout.fillWidth: true
                visible: !!root.lastRun && !!root.lastRun.errors
                text: root.lastRun ? (root.lastRun.errors || "") : ""
                color: Colours.palette.m3error
                font: Tokens.font.body.small
                wrapMode: Text.WordWrap
            }

        }

    }

    Flickable {
        id: streamView

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: detailPanel.bottom
        anchors.bottom: parent.bottom
        anchors.topMargin: Tokens.spacing.large
        clip: true
        contentWidth: width
        contentHeight: streamColumn.implicitHeight
        flickableDirection: Flickable.VerticalFlick
        boundsBehavior: Flickable.StopAtBounds

        ColumnLayout {
            id: streamColumn

            width: streamView.width
            spacing: Tokens.spacing.small

            StyledText {
                Layout.fillWidth: true
                text: qsTr("Recent stream")
                color: Colours.palette.m3onSurfaceVariant
                font: Tokens.font.title.small
            }

            StyledText {
                Layout.fillWidth: true
                visible: root.recentPosts.length === 0
                text: qsTr("No observed posts yet")
                color: Colours.palette.m3outline
                font: Tokens.font.body.medium
            }

            Repeater {
                model: root.recentPosts

                delegate: StreamRow {
                    required property var modelData

                    Layout.fillWidth: true
                    postData: modelData
                }

            }

        }

    }

    component SummaryCard: StyledRect {
        id: summaryCard

        required property string iconName
        required property string title
        required property string value
        required property string detail
        required property color accent

        implicitHeight: summaryContent.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.large
        color: Colours.tPalette.m3surfaceContainer

        RowLayout {
            id: summaryContent

            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: summaryCard.iconName
                color: summaryCard.accent
                fill: 1
                fontStyle: Tokens.font.icon.medium
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                RowLayout {
                    Layout.fillWidth: true

                    StyledText {
                        Layout.fillWidth: true
                        text: summaryCard.title
                        color: Colours.palette.m3onSurfaceVariant
                        font: Tokens.font.body.small
                    }

                    StyledText {
                        text: summaryCard.value
                        color: summaryCard.accent
                        font: Tokens.font.title.medium
                    }

                }

                StyledText {
                    Layout.fillWidth: true
                    text: summaryCard.detail
                    elide: Text.ElideRight
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.body.small
                }

            }

        }

    }

    component StreamRow: StyledRect {
        id: streamRow

        required property var postData

        implicitHeight: rowContent.implicitHeight + Tokens.padding.medium * 2
        radius: Tokens.rounding.medium
        color: Colours.tPalette.m3surfaceContainer
        border.width: 1
        border.color: Qt.alpha(Colours.palette.m3outline, 0.12)

        RowLayout {
            id: rowContent

            anchors.fill: parent
            anchors.margins: Tokens.padding.medium
            spacing: Tokens.spacing.medium

            MaterialIcon {
                text: streamRow.postData.media ? "photo_library" : "feed"
                color: streamRow.postData.media ? Colours.palette.m3tertiary : Colours.palette.m3outline
                fontStyle: Tokens.font.icon.small
            }

            ColumnLayout {
                Layout.fillWidth: true
                spacing: 0

                StyledText {
                    Layout.fillWidth: true
                    text: `@${streamRow.postData.source || "unknown"} · score ${Math.round(Number(streamRow.postData.score || 0))}`
                    color: Colours.palette.m3onSurfaceVariant
                    font: Tokens.font.label.medium
                }

                StyledText {
                    Layout.fillWidth: true
                    text: streamRow.postData.text || "(no text)"
                    color: Colours.palette.m3onSurface
                    font: Tokens.font.body.medium
                    elide: Text.ElideRight
                }

            }

        }

    }

}
