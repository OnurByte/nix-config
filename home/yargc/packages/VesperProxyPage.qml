pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import Quickshell.Io
import Caelestia.Config
import qs.components
import qs.components.controls
import qs.modules.nexus.common

PageBase {
    id: root

    property var proxy: ({ configured: false, http: "", https: "", socks: "", noProxy: "", authSupported: false, pacSupported: false, appliesTo: "new-processes" })
    property string message: ""

    title: qsTr("Proxy")
    isSubPage: true

    function refresh() {
        if (!status.running)
            status.running = true;
    }

    function setField(kind, value) {
        if (change.running)
            return;
        root.message = "";
        change.command = ["@vesperControl@", "proxy", "set", kind, value.trim()];
        change.running = true;
    }

    function clearField(kind) {
        if (change.running)
            return;
        root.message = "";
        change.command = ["@vesperControl@", "proxy", "clear", kind];
        change.running = true;
    }

    Component.onCompleted: refresh()

    Process {
        id: status
        command: ["@vesperControl@", "proxy", "status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    root.proxy = JSON.parse(text);
                    if (!httpField.activeFocus) httpField.text = root.proxy.http || "";
                    if (!httpsField.activeFocus) httpsField.text = root.proxy.https || "";
                    if (!socksField.activeFocus) socksField.text = root.proxy.socks || "";
                    if (!noProxyField.activeFocus) noProxyField.text = root.proxy.noProxy || "";
                    root.message = "";
                } catch (e) {
                    root.message = qsTr("Could not read proxy state");
                }
            }
        }
    }

    Process {
        id: change
        stderr: StdioCollector { id: changeError }
        onExited: (code, status) => {
            root.message = code === 0 ? qsTr("Proxy configuration updated") : changeError.text.trim();
            root.refresh();
        }
    }

    ColumnLayout {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: root.cappedWidth
        spacing: Tokens.spacing.extraSmall / 2

        SectionHeader { first: true; text: qsTr("Process proxy") }

        InfoRow {
            icon: "language"
            label: qsTr("Environment proxy")
            subtext: qsTr("HTTP, HTTPS, SOCKS and NO_PROXY are independent; changes apply to newly launched processes")
            value: root.proxy.configured ? qsTr("configured") : qsTr("off")
        }

        InfoRow {
            icon: "lock"
            label: qsTr("Credential policy")
            subtext: qsTr("credential-bearing URLs are rejected; Vesper will not persist proxy passwords in environment.d")
            value: qsTr("no plaintext auth")
        }

        SectionHeader { text: qsTr("HTTP") }
        StyledTextField {
            id: httpField
            Layout.fillWidth: true
            placeholderText: qsTr("http://host:port")
            leadingIcon: "http"
            supportingText: qsTr("used as HTTP_PROXY/http_proxy")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton { isRound: true; icon: "delete"; text: qsTr("Clear"); disabled: change.running || !root.proxy.http; onClicked: root.clearField("http") }
            IconTextButton { isRound: true; icon: "save"; text: qsTr("Save"); disabled: change.running || !httpField.text.trim(); onClicked: root.setField("http", httpField.text) }
        }

        SectionHeader { text: qsTr("HTTPS") }
        StyledTextField {
            id: httpsField
            Layout.fillWidth: true
            placeholderText: qsTr("http://host:port or https://host:port")
            leadingIcon: "https"
            supportingText: qsTr("used as HTTPS_PROXY/https_proxy")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton { isRound: true; icon: "delete"; text: qsTr("Clear"); disabled: change.running || !root.proxy.https; onClicked: root.clearField("https") }
            IconTextButton { isRound: true; icon: "save"; text: qsTr("Save"); disabled: change.running || !httpsField.text.trim(); onClicked: root.setField("https", httpsField.text) }
        }

        SectionHeader { text: qsTr("SOCKS") }
        StyledTextField {
            id: socksField
            Layout.fillWidth: true
            placeholderText: qsTr("socks5h://127.0.0.1:9050")
            leadingIcon: "route"
            supportingText: qsTr("used as ALL_PROXY/all_proxy; socks5h keeps DNS resolution on the proxy side")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton {
                isRound: true
                icon: "security"
                text: qsTr("Tor")
                disabled: change.running
                onClicked: root.setField("socks", "socks5h://127.0.0.1:9050")
            }
            IconTextButton { isRound: true; icon: "delete"; text: qsTr("Clear"); disabled: change.running || !root.proxy.socks; onClicked: root.clearField("socks") }
            IconTextButton { isRound: true; icon: "save"; text: qsTr("Save"); disabled: change.running || !socksField.text.trim(); onClicked: root.setField("socks", socksField.text) }
        }

        SectionHeader { text: qsTr("Bypass") }
        StyledTextField {
            id: noProxyField
            Layout.fillWidth: true
            placeholderText: qsTr("localhost,127.0.0.1,.example.com,10.0.0.0/8")
            leadingIcon: "block"
            supportingText: qsTr("comma-separated NO_PROXY/no_proxy entries")
            inputMethodHints: Qt.ImhNoPredictiveText | Qt.ImhNoAutoUppercase
        }
        RowLayout {
            Layout.fillWidth: true
            Item { Layout.fillWidth: true }
            IconTextButton { isRound: true; icon: "delete"; text: qsTr("Clear"); disabled: change.running || !root.proxy.noProxy; onClicked: root.clearField("no-proxy") }
            IconTextButton { isRound: true; icon: "save"; text: qsTr("Save"); disabled: change.running || !noProxyField.text.trim(); onClicked: root.setField("no-proxy", noProxyField.text) }
        }

        SectionHeader { text: qsTr("Policy") }
        InfoRow {
            icon: "key_off"
            label: qsTr("Authenticated proxy")
            subtext: qsTr("not exposed until credentials can be injected without persisting them in process-global environment files")
            value: root.proxy.authSupported ? qsTr("available") : qsTr("unsupported")
        }
        InfoRow {
            icon: "description"
            label: qsTr("PAC")
            subtext: qsTr("not exposed because this backend currently manages environment variables, not browser/system PAC engines")
            value: root.proxy.pacSupported ? qsTr("available") : qsTr("unsupported")
        }

        RowButton {
            icon: "delete_sweep"
            text: qsTr("Clear all proxy settings")
            subtext: qsTr("removes Vesper proxy state and environment.d entries")
            disabled: change.running || !root.proxy.configured
            onClicked: root.clearField("all")
        }

        StyledText {
            Layout.fillWidth: true
            visible: root.message
            text: root.message
            color: root.message.toLowerCase().includes("updated") ? Colours.palette.m3primary : Colours.palette.m3error
            font: Tokens.font.label.small
            wrapMode: Text.WordWrap
        }
    }
}
