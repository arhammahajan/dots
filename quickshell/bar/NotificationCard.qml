import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Services.Notifications

Rectangle {
    id: card

    required property var entry
    required property var theme
    property bool compact: false
    property real nowMs: Date.now()

    signal closeToast()
    signal dismissNotification()
    signal closeAllToasts()

    readonly property var notif: entry ? entry.notif : null
    readonly property int urgency: notif ? notif.urgency : NotificationUrgency.Normal
    readonly property bool isCritical: urgency === NotificationUrgency.Critical
    readonly property bool isLow: urgency === NotificationUrgency.Low

    readonly property int progressValue: {
        if (!notif || !notif.hints) return -1;
        const v = notif.hints["value"];
        if (v === undefined || v === null) return -1;
        const n = Number(v);
        return isNaN(n) ? -1 : Math.max(0, Math.min(100, Math.round(n)));
    }

    function resolveIcon(name) {
        if (!name || name.length === 0) return "";
        if (name.startsWith("image://") || name.startsWith("file://")) return name;
        if (name.startsWith("/")) return "file://" + name;
        const p = Quickshell.iconPath(name, true);
        return (p && p.length > 0) ? p : "";
    }

    readonly property string resolvedIconSource: {
        if (!notif) return "";

        const img = resolveIcon(notif.image);
        if (img.length > 0) return img;

        const appIcon = resolveIcon(notif.appIcon);
        if (appIcon.length > 0) return appIcon;

        const desktopId = notif.desktopEntry || "";
        if (desktopId.length > 0) {
            const de = DesktopEntries.byId(desktopId) || DesktopEntries.heuristicLookup(desktopId);
            if (de && de.icon) {
                const p = resolveIcon(de.icon);
                if (p.length > 0) return p;
            }
        }

        const appName = notif.appName || "";
        if (appName.length > 0) {
            const de = DesktopEntries.heuristicLookup(appName);
            if (de && de.icon) {
                const p = resolveIcon(de.icon);
                if (p.length > 0) return p;
            }
        }

        return "";
    }

    readonly property var visibleActions: {
        if (!notif || !notif.actions) return [];
        const out = [];
        for (let i = 0; i < notif.actions.length; i++) {
            const a = notif.actions[i];
            if (a && a.text && a.text.trim().length > 0) {
                out.push(a);
            }
        }
        return out;
    }

    function formatRelativeTime(createdMs, currentMs) {
        const diffSec = Math.max(0, Math.floor((currentMs - createdMs) / 1000));
        if (diffSec < 45) return "now";
        const mins = Math.floor(diffSec / 60);
        if (mins < 60) return mins + "m";
        const hrs = Math.floor(mins / 60);
        if (hrs < 24) return hrs + "h";
        return Math.floor(hrs / 24) + "d";
    }

    // Safely keep common Pango/HTML tags supported by Qt StyledText, escaping everything else
    function sanitizeMarkup(raw) {
        if (!raw) return "";
        let text = String(raw);
        // Normalize newlines
        text = text.replace(/\r\n/g, "\n");

        // Temporarily protect allowed tags
        const placeholders = [];
        text = text.replace(/<\/?(?:b|i|u|s|br)\s*\/?>/gi, match => {
            const lower = match.toLowerCase().replace(/\s+/g, "");
            const normalized = (lower === "<br>" || lower === "<br/>") ? "<br/>" : lower;
            const idx = placeholders.length;
            placeholders.push(normalized);
            return "\u0001TAG" + idx + "\u0002";
        });

        // Strip other HTML tags (e.g. <span>, <a ...>) keeping their inner text
        text = text.replace(/<\/?[a-zA-Z][^>]*>/g, "");

        // Escape remaining < and > (preserve valid XML entities)
        text = text.replace(/&(?!amp;|lt;|gt;|quot;|apos;|#\d+;|#x[0-9a-fA-F]+;)/g, "&amp;");
        text = text.replace(/</g, "&lt;").replace(/>/g, "&gt;");
        text = text.replace(/\n/g, "<br/>");

        // Restore allowed tags
        text = text.replace(/\u0001TAG(\d+)\u0002/g, (_, idx) => placeholders[Number(idx)] || "");
        return text;
    }

    readonly property bool isHovered: cardHover.hovered

    onIsHoveredChanged: {
        if (!compact && entry) {
            entry.hovered = isHovered;
        }
    }

    implicitHeight: contentCol.implicitHeight + (compact ? 16 : 20) + (!compact && entry && entry.timeoutMs > 0 ? 3 : 0)
    radius: compact ? 8 : 10
    color: compact
        ? (isHovered ? Qt.rgba(255/255, 255/255, 255/255, 0.07) : Qt.rgba(255/255, 255/255, 255/255, 0.03))
        : theme.bgCard
    border.width: 1
    border.color: {
        if (isCritical) return Qt.rgba(224/255, 108/255, 117/255, 0.65);
        if (isHovered && !compact) return Qt.rgba(183/255, 212/255, 237/255, 0.35);
        return theme.borderMain;
    }
    clip: true

    Behavior on border.color { ColorAnimation { duration: 120 } }
    Behavior on color { ColorAnimation { duration: 120 } }

    HoverHandler {
        id: cardHover
    }

    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
        cursorShape: Qt.PointingHandCursor
        onClicked: mouse => {
            if (mouse.button === Qt.RightButton) {
                if (!compact) card.closeAllToasts();
                else card.dismissNotification();
            } else if (mouse.button === Qt.MiddleButton) {
                if (card.notif && card.notif.actions && card.notif.actions.length > 0) {
                    let target = card.notif.actions[0];
                    for (let i = 0; i < card.notif.actions.length; i++) {
                        if (card.notif.actions[i].identifier === "default") {
                            target = card.notif.actions[i];
                            break;
                        }
                    }
                    target.invoke();
                    if (!card.notif.resident) {
                        if (!compact) card.closeToast();
                    }
                }
            } else if (mouse.button === Qt.LeftButton) {
                if (!compact) card.closeToast();
            }
        }
    }

    ColumnLayout {
        id: contentCol
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.leftMargin: 10
        anchors.rightMargin: 10
        anchors.topMargin: compact ? 8 : 10
        spacing: 6

        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            // App / Notification Icon
            Rectangle {
                Layout.alignment: Qt.AlignVCenter
                implicitWidth: compact ? 30 : 38
                implicitHeight: compact ? 30 : 38
                radius: 7
                color: isCritical
                    ? Qt.rgba(224/255, 108/255, 117/255, 0.14)
                    : Qt.rgba(255/255, 255/255, 255/255, 0.06)
                clip: true

                Image {
                    id: iconImg
                    anchors.fill: parent
                    anchors.margins: (card.notif && card.notif.image && card.notif.image.length > 0) ? 0 : 4
                    source: card.resolvedIconSource
                    sourceSize.width: compact ? 30 : 38
                    sourceSize.height: compact ? 30 : 38
                    fillMode: (card.notif && card.notif.image && card.notif.image.length > 0)
                        ? Image.PreserveAspectCrop
                        : Image.PreserveAspectFit
                    smooth: true
                    asynchronous: true
                    visible: status === Image.Ready && implicitWidth > 1 && implicitHeight > 1
                }

                Text {
                    anchors.centerIn: parent
                    visible: !iconImg.visible
                    text: isCritical ? "󰀪" : "󰂚"
                    color: isCritical ? theme.critColor : theme.accent
                    font.family: theme.fontFamily
                    font.pixelSize: compact ? 14 : 17
                }
            }

            // Text Content
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 3

                // Header: App Name + Time + Close
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6

                    Text {
                        text: (card.notif && card.notif.appName && card.notif.appName.length > 0)
                            ? card.notif.appName
                            : "Notification"
                        color: theme.contentMuted
                        font.family: theme.fontFamily
                        font.pixelSize: 10
                        elide: Text.ElideRight
                        Layout.maximumWidth: 140
                    }

                    Text {
                        visible: card.entry !== null
                        text: "· " + (card.entry ? card.formatRelativeTime(card.entry.createdAt, card.nowMs) : "")
                        color: Qt.rgba(140/255, 147/255, 160/255, 0.75)
                        font.family: theme.fontFamily
                        font.pixelSize: 10
                    }

                    Rectangle {
                        visible: card.isCritical
                        implicitWidth: critLabel.implicitWidth + 8
                        implicitHeight: 15
                        radius: 4
                        color: Qt.rgba(224/255, 108/255, 117/255, 0.18)

                        Text {
                            id: critLabel
                            anchors.centerIn: parent
                            text: "urgent"
                            color: theme.critColor
                            font.family: theme.fontFamily
                            font.pixelSize: 9
                            font.weight: Font.DemiBold
                        }
                    }

                    Item { Layout.fillWidth: true }

                    Rectangle {
                        implicitWidth: 20
                        implicitHeight: 20
                        radius: 5
                        color: dismissMouse.containsMouse ? theme.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "󰅖"
                            color: dismissMouse.containsMouse ? theme.contentMain : theme.contentMuted
                            font.family: theme.fontFamily
                            font.pixelSize: 10
                        }

                        MouseArea {
                            id: dismissMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: card.dismissNotification()
                        }
                    }
                }

                // Summary
                Text {
                    visible: card.notif && card.notif.summary && card.notif.summary.length > 0
                    Layout.fillWidth: true
                    text: card.notif ? card.notif.summary : ""
                    color: theme.contentMain
                    font.family: theme.fontFamily
                    font.pixelSize: 12
                    font.weight: Font.DemiBold
                    wrapMode: Text.Wrap
                    maximumLineCount: 2
                    elide: Text.ElideRight
                    textFormat: Text.PlainText
                }

                // Body
                Text {
                    visible: card.notif && card.notif.body && card.notif.body.length > 0
                    Layout.fillWidth: true
                    text: card.notif ? card.sanitizeMarkup(card.notif.body) : ""
                    color: Qt.rgba(226/255, 228/255, 233/255, 0.80)
                    font.family: theme.fontFamily
                    font.pixelSize: 11
                    wrapMode: Text.Wrap
                    maximumLineCount: compact ? 3 : 4
                    elide: Text.ElideRight
                    textFormat: Text.StyledText
                }

                // Progress bar (e.g. notify-send -h int:value:60)
                Rectangle {
                    visible: card.progressValue >= 0
                    Layout.fillWidth: true
                    Layout.topMargin: 3
                    height: 4
                    radius: 2
                    color: theme.borderMain

                    Rectangle {
                        width: parent.width * (Math.max(0, card.progressValue) / 100)
                        height: parent.height
                        radius: 2
                        color: card.isCritical ? theme.critColor : theme.accent
                    }
                }
            }
        }

        // Action Buttons
        Flow {
            visible: card.visibleActions.length > 0
            Layout.fillWidth: true
            Layout.topMargin: 2
            spacing: 6

            Repeater {
                model: card.visibleActions
                delegate: Rectangle {
                    required property var modelData
                    implicitWidth: Math.min(actLabel.implicitWidth + 16, 150)
                    implicitHeight: 24
                    radius: 6
                    color: actMouse.containsMouse ? theme.bgWsActive : theme.bgHover
                    border.width: 1
                    border.color: actMouse.containsMouse ? Qt.rgba(183/255, 212/255, 237/255, 0.40) : theme.borderMain

                    Behavior on color { ColorAnimation { duration: 100 } }

                    Text {
                        id: actLabel
                        anchors.centerIn: parent
                        width: Math.min(implicitWidth, parent.width - 10)
                        text: modelData.text
                        color: actMouse.containsMouse ? theme.accent : theme.contentMain
                        font.family: theme.fontFamily
                        font.pixelSize: 11
                        elide: Text.ElideRight
                        horizontalAlignment: Text.AlignHCenter
                    }

                    MouseArea {
                        id: actMouse
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            modelData.invoke();
                            if (card.notif && !card.notif.resident && !card.compact) {
                                card.closeToast();
                            }
                        }
                    }
                }
            }
        }
    }

    // Subtle remaining-time bar on auto-expiring floating toasts
    Rectangle {
        visible: !compact && entry && entry.timeoutMs > 0
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        height: 2
        width: (entry && entry.timeoutMs > 0)
            ? parent.width * Math.max(0, Math.min(1, entry.remainingMs / entry.timeoutMs))
            : 0
        color: isHovered
            ? Qt.rgba(183/255, 212/255, 237/255, 0.65)
            : Qt.rgba(183/255, 212/255, 237/255, 0.32)
    }
}
