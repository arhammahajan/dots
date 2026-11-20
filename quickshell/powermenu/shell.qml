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
    readonly property color borderMain: Qt.rgba(216/255, 218/255, 221/255, 0.16)
    readonly property color contentMain: "#e2e4e9"
    readonly property color contentMuted: "#8c93a0"
    readonly property color accent: "#b7d4ed"
    readonly property color critColor: "#e06c75"
    readonly property string fontFamily: "JetBrainsMono Nerd Font"

    property int selectedIndex: 0

    readonly property var actions: [
        { label: "lock",     icon: "󰌾", cmd: ["/home/arhammahajan/dev/dots/scripts/lock.sh"] },
        { label: "suspend",  icon: "󰤄", cmd: ["systemctl", "suspend"] },
        { label: "logout",   icon: "󰍃", cmd: ["pkill", "niri"] },
        { label: "reboot",   icon: "󰜉", cmd: ["systemctl", "reboot"] },
        { label: "shutdown", icon: "󰐥", cmd: ["systemctl", "poweroff"] }
    ]

    Process {
        id: runner
        onExited: Qt.quit()
    }

    function trigger(index) {
        if (index < 0 || index >= actions.length) return;
        runner.command = actions[index].cmd;
        runner.running = true;
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
            Keys.onLeftPressed: root.selectedIndex = (root.selectedIndex - 1 + root.actions.length) % root.actions.length
            Keys.onRightPressed: root.selectedIndex = (root.selectedIndex + 1) % root.actions.length
            Keys.onReturnPressed: root.trigger(root.selectedIndex)
            Keys.onEnterPressed: root.trigger(root.selectedIndex)

            Rectangle {
                anchors.centerIn: parent
                width: cardCol.implicitWidth + 28
                height: cardCol.implicitHeight + 28
                radius: 10
                color: root.bgCard
                border.width: 1
                border.color: root.borderMain

                MouseArea {
                    anchors.fill: parent
                    onClicked: {} // absorb clicks inside card
                }

                ColumnLayout {
                    id: cardCol
                    anchors.centerIn: parent
                    spacing: 12

                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Text {
                            text: "󰐥  Power"
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 13
                            font.weight: Font.DemiBold
                        }

                        Item { Layout.fillWidth: true }

                        Text {
                            text: "esc"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 11
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: root.borderMain
                    }

                    RowLayout {
                        id: row
                        spacing: 8

                        Repeater {
                            model: root.actions
                            delegate: Rectangle {
                                required property var modelData
                                required property int index
                                readonly property bool active: mouse.containsMouse || root.selectedIndex === index
                                readonly property bool isShutdown: modelData.label === "shutdown"

                                implicitWidth: 92
                                implicitHeight: 88
                                radius: 7
                                color: active ? root.bgActive : "transparent"
                                border.width: 1
                                border.color: active ? (isShutdown ? root.critColor : root.borderMain) : Qt.rgba(216/255, 218/255, 221/255, 0.08)

                                Behavior on color { ColorAnimation { duration: 120 } }

                                ColumnLayout {
                                    anchors.centerIn: parent
                                    spacing: 8

                                    Text {
                                        Layout.alignment: Qt.AlignHCenter
                                        text: modelData.icon
                                        color: active ? (isShutdown ? root.critColor : root.accent) : root.contentMain
                                        font.family: root.fontFamily
                                        font.pixelSize: 24
                                    }

                                    Text {
                                        Layout.alignment: Qt.AlignHCenter
                                        text: modelData.label
                                        color: active ? root.contentMain : root.contentMuted
                                        font.family: root.fontFamily
                                        font.pixelSize: 12
                                    }
                                }

                                MouseArea {
                                    id: mouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onEntered: root.selectedIndex = index
                                    onClicked: root.trigger(index)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
