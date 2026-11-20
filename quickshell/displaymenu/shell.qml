import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland

ShellRoot {
    id: root

    readonly property color bgOverlay: Qt.rgba(11/255, 11/255, 12/255, 0.55)
    readonly property color bgCard: Qt.rgba(22/255, 23/255, 26/255, 0.94)
    readonly property color bgHover: Qt.rgba(255/255, 255/255, 255/255, 0.10)
    readonly property color bgActive: Qt.rgba(255/255, 255/255, 255/255, 0.16)
    readonly property color bgTrack: Qt.rgba(255/255, 255/255, 255/255, 0.12)
    readonly property color borderMain: Qt.rgba(216/255, 218/255, 221/255, 0.16)
    readonly property color contentMain: "#e2e4e9"
    readonly property color contentMuted: "#8c93a0"
    readonly property color accent: "#b7d4ed"
    readonly property string fontFamily: "JetBrainsMono Nerd Font"

    readonly property string scriptPath: "/home/arhammahajan/dev/dots/scripts/switch-monitor-layout.py"

    property string activeLayout: ""
    property var layouts: [
        { id: "original", label: "original", desc: "AW3426DWM + GW2480 (portrait)", available: true },
        { id: "stacked",  label: "stacked",  desc: "GW2270 (180°) + ULTRAGEAR",     available: true },
        { id: "all",      label: "all",      desc: "All 4 monitors",                available: true }
    ]
    property var monitors: []
    property bool loading: true
    property int selectedIndex: 0

    Process {
        id: statusProc
        command: [root.scriptPath, "--status"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const data = JSON.parse(this.text);
                    root.activeLayout = data.active_layout || "";
                    if (data.layouts) root.layouts = data.layouts;
                    if (data.monitors) root.monitors = data.monitors;
                    for (let i = 0; i < root.layouts.length; i++) {
                        if (root.layouts[i].id === root.activeLayout) {
                            root.selectedIndex = i;
                            break;
                        }
                    }
                } catch (e) {
                    console.log("displaymenu parse error:", e);
                }
                root.loading = false;
            }
        }
    }

    Process {
        id: layoutProc
        onExited: {
            root.loading = true;
            statusProc.running = true;
        }
    }

    function applyLayout(id) {
        if (!id || layoutProc.running) return;
        layoutProc.command = [root.scriptPath, id];
        layoutProc.running = true;
    }

    Component.onCompleted: {
        statusProc.running = true;
    }

    PanelWindow {
        anchors {
            top: true
            bottom: true
            left: true
            right: true
        }
        color: root.bgOverlay
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.Exclusive
        exclusiveZone: -1

        MouseArea {
            anchors.fill: parent
            onClicked: Qt.quit()
        }

        Item {
            anchors.fill: parent
            focus: true
            Keys.onEscapePressed: Qt.quit()
            Keys.onLeftPressed: root.selectedIndex = (root.selectedIndex - 1 + root.layouts.length) % root.layouts.length
            Keys.onRightPressed: root.selectedIndex = (root.selectedIndex + 1) % root.layouts.length
            Keys.onReturnPressed: root.applyLayout(root.layouts[root.selectedIndex].id)
            Keys.onEnterPressed: root.applyLayout(root.layouts[root.selectedIndex].id)

            Rectangle {
                anchors.centerIn: parent
                width: Math.max(460, mainCol.implicitWidth + 36)
                height: mainCol.implicitHeight + 36
                radius: 10
                color: root.bgCard
                border.width: 1
                border.color: root.borderMain

                MouseArea {
                    anchors.fill: parent
                    onClicked: {}
                }

                ColumnLayout {
                    id: mainCol
                    anchors.fill: parent
                    anchors.margins: 18
                    spacing: 14

                    // Header
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: "󰍹  Displays"
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 13
                            font.weight: Font.DemiBold
                        }

                        Item { Layout.fillWidth: true }

                        Text {
                            visible: root.loading
                            text: "syncing..."
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 11
                        }

                        Rectangle {
                            implicitWidth: 24
                            implicitHeight: 24
                            radius: 6
                            color: closeDispMouse.containsMouse ? root.bgHover : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "󰅖"
                                color: root.contentMuted
                                font.family: root.fontFamily
                                font.pixelSize: 12
                            }

                            MouseArea {
                                id: closeDispMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: Qt.quit()
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: root.borderMain
                    }

                    // Layout selector row
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: "layout"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        RowLayout {
                            Layout.fillWidth: true
                            spacing: 10

                            Repeater {
                                model: root.layouts
                                delegate: Rectangle {
                                    required property var modelData
                                    required property int index
                                    readonly property bool isCurrent: modelData.id === root.activeLayout
                                    readonly property bool isFocused: layoutMouse.containsMouse || root.selectedIndex === index

                                    Layout.fillWidth: true
                                    implicitHeight: 58
                                    radius: 7
                                    color: isCurrent ? root.bgActive : (isFocused ? root.bgHover : "transparent")
                                    border.width: 1
                                    border.color: isCurrent ? root.accent : (isFocused ? root.borderMain : Qt.rgba(216/255, 218/255, 221/255, 0.08))
                                    opacity: modelData.available ? 1.0 : 0.45

                                    Behavior on color { ColorAnimation { duration: 120 } }

                                    ColumnLayout {
                                        anchors.centerIn: parent
                                        width: parent.width - 16
                                        spacing: 4

                                        Text {
                                            Layout.alignment: Qt.AlignHCenter
                                            text: isCurrent ? "● " + modelData.label : modelData.label
                                            color: isCurrent ? root.accent : root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 14
                                            font.bold: isCurrent
                                        }

                                        Text {
                                            Layout.alignment: Qt.AlignHCenter
                                            Layout.fillWidth: true
                                            horizontalAlignment: Text.AlignHCenter
                                            elide: Text.ElideRight
                                            text: modelData.desc
                                            color: root.contentMuted
                                            font.family: root.fontFamily
                                            font.pixelSize: 11
                                        }
                                    }

                                    MouseArea {
                                        id: layoutMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        enabled: modelData.available
                                        cursorShape: Qt.PointingHandCursor
                                        onEntered: root.selectedIndex = index
                                        onClicked: root.applyLayout(modelData.id)
                                    }
                                }
                            }
                        }
                    }

                    // Divider
                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: root.borderMain
                    }

                    // Brightness sliders per detected monitor
                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 10

                        Text {
                            text: "brightness"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        Text {
                            visible: root.monitors.length === 0
                            text: root.loading ? "detecting monitors..." : "no monitors detected"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        Repeater {
                            model: root.monitors
                            delegate: ColumnLayout {
                                id: monRow
                                required property var modelData
                                readonly property bool hasDdc: modelData.bus !== null && modelData.bus !== undefined
                                readonly property int maxVal: (modelData.max_brightness && modelData.max_brightness > 0) ? modelData.max_brightness : 100
                                property int currentVal: (modelData.brightness !== null && modelData.brightness !== undefined) ? modelData.brightness : 0
                                property int pendingVal: currentVal

                                Layout.fillWidth: true
                                spacing: 6

                                Process {
                                    id: setBrightProc
                                }

                                Timer {
                                    id: debounceTimer
                                    interval: 120
                                    repeat: false
                                    onTriggered: {
                                        if (!monRow.hasDdc) return;
                                        setBrightProc.command = [
                                            root.scriptPath,
                                            "--set-brightness",
                                            String(monRow.modelData.bus),
                                            String(monRow.pendingVal)
                                        ];
                                        setBrightProc.running = true;
                                    }
                                }

                                function updateBrightness(val) {
                                    const clamped = Math.max(0, Math.min(monRow.maxVal, Math.round(val)));
                                    if (clamped === monRow.currentVal) return;
                                    monRow.currentVal = clamped;
                                    monRow.pendingVal = clamped;
                                    debounceTimer.restart();
                                }

                                RowLayout {
                                    Layout.fillWidth: true
                                    spacing: 8

                                    Text {
                                        text: "󰃠"
                                        color: monRow.hasDdc ? root.accent : root.contentMuted
                                        font.family: root.fontFamily
                                        font.pixelSize: 14
                                    }

                                    Text {
                                        text: monRow.modelData.model
                                        color: monRow.modelData.active ? root.contentMain : root.contentMuted
                                        font.family: root.fontFamily
                                        font.pixelSize: 13
                                    }

                                    Text {
                                        text: monRow.modelData.connector + (monRow.modelData.active ? "" : " (off)")
                                        color: root.contentMuted
                                        font.family: root.fontFamily
                                        font.pixelSize: 11
                                    }

                                    Item { Layout.fillWidth: true }

                                    Text {
                                        text: monRow.hasDdc ? (Math.round(monRow.currentVal * 100 / monRow.maxVal) + "%") : "no ddc/ci"
                                        color: monRow.hasDdc ? root.contentMain : root.contentMuted
                                        font.family: root.fontFamily
                                        font.pixelSize: 12
                                    }
                                }

                                // Slider track
                                Item {
                                    visible: monRow.hasDdc
                                    Layout.fillWidth: true
                                    height: 18

                                    Rectangle {
                                        id: track
                                        anchors.verticalCenter: parent.verticalCenter
                                        width: parent.width
                                        height: 6
                                        radius: 3
                                        color: root.bgTrack

                                        Rectangle {
                                            width: Math.max(6, parent.width * (monRow.currentVal / monRow.maxVal))
                                            height: parent.height
                                            radius: 3
                                            color: root.accent
                                        }

                                        Rectangle {
                                            x: Math.max(0, Math.min(parent.width - width, parent.width * (monRow.currentVal / monRow.maxVal) - width / 2))
                                            anchors.verticalCenter: parent.verticalCenter
                                            width: 14
                                            height: 14
                                            radius: 7
                                            color: root.contentMain
                                            border.width: 2
                                            border.color: root.accent
                                        }
                                    }

                                    MouseArea {
                                        anchors.fill: parent
                                        cursorShape: Qt.PointingHandCursor
                                        onPressed: (mouse) => monRow.updateBrightness((mouse.x / width) * monRow.maxVal)
                                        onPositionChanged: (mouse) => {
                                            if (pressed) monRow.updateBrightness((mouse.x / width) * monRow.maxVal);
                                        }
                                        onWheel: (wheel) => {
                                            const step = wheel.angleDelta.y > 0 ? 5 : -5;
                                            monRow.updateBrightness(monRow.currentVal + step);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
