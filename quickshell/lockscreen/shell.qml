import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Services.Pam

ShellRoot {
    id: root

    WlSessionLock {
        id: sessionLock
        locked: true

        surface: WlSessionLockSurface {
            id: lockSurface
            color: "#0b0b0c"

            property string pendingPassword: ""
            property string statusMessage: ""
            property bool statusIsError: false
            property bool verifying: false

            function submitPassword() {
                if (verifying) return;
                if (passwordInput.text.length === 0) return;

                statusMessage = "";
                statusIsError = false;
                verifying = true;

                if (pam.active && pam.responseRequired) {
                    const pw = passwordInput.text;
                    passwordInput.text = "";
                    pam.respond(pw);
                } else {
                    pendingPassword = passwordInput.text;
                    passwordInput.text = "";
                    if (!pam.active) {
                        pam.start();
                    }
                }
            }

            PamContext {
                id: pam
                config: "login"

                onPamMessage: {
                    if (pam.responseRequired && lockSurface.pendingPassword.length > 0) {
                        const pw = lockSurface.pendingPassword;
                        lockSurface.pendingPassword = "";
                        pam.respond(pw);
                    } else if (pam.message.length > 0 && !pam.responseRequired) {
                        lockSurface.statusMessage = pam.message;
                        lockSurface.statusIsError = pam.messageIsError;
                    }
                }

                onCompleted: result => {
                    lockSurface.verifying = false;
                    lockSurface.pendingPassword = "";

                    if (result === PamResult.Success) {
                        sessionLock.locked = false;
                        Qt.quit();
                    } else {
                        lockSurface.statusIsError = true;
                        lockSurface.statusMessage = pam.message.length > 0
                            ? pam.message
                            : "Incorrect password";
                        passwordInput.text = "";
                        passwordInput.forceActiveFocus();
                    }
                }

                onError: {
                    lockSurface.verifying = false;
                    lockSurface.pendingPassword = "";
                    lockSurface.statusIsError = true;
                    lockSurface.statusMessage = pam.message.length > 0
                        ? pam.message
                        : "Authentication error";
                    passwordInput.forceActiveFocus();
                }
            }

            // Blurred rice wallpaper backdrop
            Image {
                anchors.fill: parent
                source: "file:///home/arhammahajan/dev/dots/wallpapers/backdrop.jpg"
                fillMode: Image.PreserveAspectCrop
                asynchronous: true
            }

            // Frosted Midnight dark scrim
            Rectangle {
                anchors.fill: parent
                color: "#660b0b0c"
            }

            // Centered standard lockscreen stack
            ColumnLayout {
                anchors.centerIn: parent
                spacing: 8

                // Clock
                Text {
                    id: clockText
                    Layout.alignment: Qt.AlignHCenter
                    color: "#d8dadd"
                    font.family: "Ubuntu"
                    font.pixelSize: 64
                    font.weight: Font.Light
                    text: Qt.formatDateTime(new Date(), "HH:mm")
                }

                // Date
                Text {
                    id: dateText
                    Layout.alignment: Qt.AlignHCenter
                    color: "#b2bcc4"
                    font.family: "Ubuntu"
                    font.pixelSize: 15
                    text: Qt.formatDateTime(new Date(), "dddd, MMMM d")
                }

                Timer {
                    interval: 1000
                    running: true
                    repeat: true
                    onTriggered: {
                        const now = new Date();
                        clockText.text = Qt.formatDateTime(now, "HH:mm");
                        dateText.text = Qt.formatDateTime(now, "dddd, MMMM d");
                    }
                }

                Item {
                    Layout.preferredHeight: 20
                }

                // Compact password box
                Rectangle {
                    Layout.alignment: Qt.AlignHCenter
                    implicitWidth: 230
                    implicitHeight: 36
                    radius: 4
                    color: "#bf0b0b0c"
                    border.width: 1
                    border.color: lockSurface.statusIsError
                        ? "#e06c75"
                        : (passwordInput.activeFocus ? "#80b7d4ed" : "#33d8dadd")

                    RowLayout {
                        anchors.fill: parent
                        anchors.leftMargin: 14
                        anchors.rightMargin: 8
                        spacing: 6

                        Item {
                            Layout.fillWidth: true
                            Layout.fillHeight: true

                            Text {
                                anchors.verticalCenter: parent.verticalCenter
                                text: lockSurface.verifying ? "Verifying..." : "Password"
                                color: "#949ea3"
                                font.family: "Ubuntu"
                                font.pixelSize: 13
                                visible: passwordInput.text.length === 0
                            }

                            TextInput {
                                id: passwordInput
                                anchors.fill: parent
                                verticalAlignment: TextInput.AlignVCenter
                                color: "#d8dadd"
                                selectionColor: "#758a9b"
                                selectedTextColor: "#d8dadd"
                                font.family: "Ubuntu"
                                font.pixelSize: 13
                                echoMode: TextInput.Password
                                passwordCharacter: "•"
                                focus: true
                                enabled: !lockSurface.verifying

                                Keys.onReturnPressed: lockSurface.submitPassword()
                                Keys.onEnterPressed: lockSurface.submitPassword()
                                Keys.onEscapePressed: {
                                    passwordInput.text = "";
                                    lockSurface.statusMessage = "";
                                    lockSurface.statusIsError = false;
                                }
                            }
                        }

                        Rectangle {
                            width: 24
                            height: 24
                            radius: 3
                            visible: passwordInput.text.length > 0 && !lockSurface.verifying
                            color: submitArea.containsMouse ? "#33b7d4ed" : "transparent"

                            Text {
                                anchors.centerIn: parent
                                text: "→"
                                color: "#b7d4ed"
                                font.family: "Ubuntu"
                                font.pixelSize: 13
                            }

                            MouseArea {
                                id: submitArea
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: lockSurface.submitPassword()
                            }
                        }
                    }
                }

                // Reserved status line so layout doesn't jump on error
                Text {
                    Layout.alignment: Qt.AlignHCenter
                    Layout.topMargin: 4
                    height: 16
                    text: lockSurface.statusMessage
                    color: lockSurface.statusIsError ? "#e06c75" : "#b7d4ed"
                    font.family: "Ubuntu"
                    font.pixelSize: 12
                    opacity: lockSurface.statusMessage.length > 0 ? 1 : 0
                }
            }

            Component.onCompleted: passwordInput.forceActiveFocus()
        }
    }
}
