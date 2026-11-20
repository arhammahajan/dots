import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Wayland
import Quickshell.Services.Notifications

Scope {
    id: manager

    required property var theme
    property var targetScreen: null
    property bool barVisible: true
    property bool centerOpen: false

    property bool dnd: false
    property int defaultTimeoutMs: 7000
    property int maxPopups: 5
    property int maxHistory: 25
    property real nowMs: Date.now()

    property var allEntries: []

    readonly property var popupEntries: {
        const list = [];
        for (let i = 0; i < allEntries.length && list.length < maxPopups; i++) {
            if (allEntries[i].inPopup) list.push(allEntries[i]);
        }
        return list;
    }

    readonly property int historyCount: allEntries.length
    readonly property alias historyModel: historyScriptModel

    ScriptModel {
        id: popupScriptModel
        values: manager.popupEntries
    }

    ScriptModel {
        id: historyScriptModel
        values: manager.allEntries
    }

    Component {
        id: entryComponent

        QtObject {
            property int notifId: 0
            property var notif: null
            property real createdAt: 0
            property bool inPopup: false
            property int timeoutMs: 0
            property int remainingMs: 0
            property bool hovered: false
            property bool closedConnActive: true
        }
    }

    function computeTimeoutMs(notification) {
        if (!notification) return defaultTimeoutMs;
        if (notification.urgency === NotificationUrgency.Critical) return 0;
        if (notification.expireTimeout === 0) return 0;
        if (notification.expireTimeout > 0) {
            const ms = Math.round(notification.expireTimeout * 1000);
            return Math.max(1500, Math.min(60000, ms));
        }
        return defaultTimeoutMs;
    }

    function removeEntryById(notifId, callDismiss, callExpire) {
        let removed = null;
        const next = [];
        for (let i = 0; i < allEntries.length; i++) {
            const e = allEntries[i];
            if (e.notifId === notifId) {
                removed = e;
            } else {
                next.push(e);
            }
        }
        if (!removed) return;
        allEntries = next;

        const n = removed.notif;
        removed.closedConnActive = false;
        removed.destroy();

        if (n) {
            try {
                if (callDismiss) n.dismiss();
                else if (callExpire) n.expire();
            } catch (err) {}
        }
    }

    function hideToastById(notifId) {
        let changed = false;
        for (let i = 0; i < allEntries.length; i++) {
            const e = allEntries[i];
            if (e.notifId === notifId && e.inPopup) {
                if (e.notif && e.notif.transient) {
                    removeEntryById(notifId, false, true);
                    return;
                }
                e.inPopup = false;
                e.hovered = false;
                changed = true;
                break;
            }
        }
        if (changed) {
            allEntries = allEntries.slice();
        }
    }

    function dismissById(notifId) {
        removeEntryById(notifId, true, false);
    }

    function closeAllToasts() {
        const toExpire = [];
        let changed = false;
        for (let i = 0; i < allEntries.length; i++) {
            const e = allEntries[i];
            if (e.inPopup) {
                if (e.notif && e.notif.transient) {
                    toExpire.push(e.notifId);
                } else {
                    e.inPopup = false;
                    e.hovered = false;
                    changed = true;
                }
            }
        }
        for (let i = 0; i < toExpire.length; i++) {
            removeEntryById(toExpire[i], false, true);
        }
        if (changed) {
            allEntries = allEntries.slice();
        }
    }

    function clearAll() {
        const snapshot = allEntries.slice();
        allEntries = [];
        for (let i = 0; i < snapshot.length; i++) {
            const e = snapshot[i];
            const n = e.notif;
            e.closedConnActive = false;
            e.destroy();
            if (n) {
                try { n.dismiss(); } catch (err) {}
            }
        }
    }

    function toggleDnd() {
        dnd = !dnd;
        if (dnd) closeAllToasts();
    }

    onCenterOpenChanged: {
        if (centerOpen) {
            nowMs = Date.now();
            closeAllToasts();
        }
    }

    // Relative-time refresh when notifications exist
    Timer {
        interval: 30000
        running: manager.allEntries.length > 0
        repeat: true
        onTriggered: manager.nowMs = Date.now()
    }

    // Smooth countdown ticker for active toasts (runs only while popups are visible)
    Timer {
        id: popupTicker
        interval: 100
        running: manager.popupEntries.length > 0
        repeat: true
        onTriggered: {
            const expiredIds = [];
            for (let i = 0; i < manager.popupEntries.length; i++) {
                const e = manager.popupEntries[i];
                if (e.timeoutMs <= 0 || e.hovered) continue;
                e.remainingMs = Math.max(0, e.remainingMs - interval);
                if (e.remainingMs <= 0) {
                    expiredIds.push(e.notifId);
                }
            }
            for (let i = 0; i < expiredIds.length; i++) {
                manager.hideToastById(expiredIds[i]);
            }
        }
    }

    NotificationServer {
        id: server
        keepOnReload: true
        persistenceSupported: true
        bodySupported: true
        bodyMarkupSupported: true
        bodyHyperlinksSupported: false
        bodyImagesSupported: false
        actionsSupported: true
        actionIconsSupported: false
        imageSupported: true
        inlineReplySupported: false

        onNotification: notification => {
            notification.tracked = true;
            manager.nowMs = Date.now();

            const showPopup = !manager.centerOpen && (!manager.dnd || notification.urgency === NotificationUrgency.Critical);
            const tMs = manager.computeTimeoutMs(notification);

            // Check if replacing an existing entry with the same ID
            for (let i = 0; i < manager.allEntries.length; i++) {
                const existing = manager.allEntries[i];
                if (existing.notifId === notification.id) {
                    existing.notif = notification;
                    existing.createdAt = manager.nowMs;
                    existing.timeoutMs = tMs;
                    existing.remainingMs = tMs;
                    existing.inPopup = showPopup;
                    // Move to top
                    const reordered = manager.allEntries.slice();
                    reordered.splice(i, 1);
                    reordered.unshift(existing);
                    manager.allEntries = reordered;
                    return;
                }
            }

            const entry = entryComponent.createObject(manager, {
                notifId: notification.id,
                notif: notification,
                createdAt: manager.nowMs,
                inPopup: showPopup,
                timeoutMs: tMs,
                remainingMs: tMs,
                hovered: false
            });

            notification.closed.connect(reason => {
                if (entry && entry.closedConnActive) {
                    manager.removeEntryById(entry.notifId, false, false);
                }
            });

            const next = [entry].concat(manager.allEntries);

            // Enforce history cap (evict oldest non-critical first)
            while (next.length > manager.maxHistory) {
                let dropIdx = -1;
                for (let i = next.length - 1; i >= 0; i--) {
                    if (!next[i].notif || next[i].notif.urgency !== NotificationUrgency.Critical) {
                        dropIdx = i;
                        break;
                    }
                }
                if (dropIdx < 0) dropIdx = next.length - 1;
                const dropped = next.splice(dropIdx, 1)[0];
                if (dropped) {
                    const dn = dropped.notif;
                    dropped.closedConnActive = false;
                    dropped.destroy();
                    if (dn) {
                        try { dn.expire(); } catch (err) {}
                    }
                }
            }

            manager.allEntries = next;
        }
    }

    // Floating Toast Stack Window (top-right, matching floating pill aesthetic)
    PanelWindow {
        id: toastWindow
        visible: manager.popupEntries.length > 0
        screen: manager.targetScreen ? manager.targetScreen : (Quickshell.screens.length > 0 ? Quickshell.screens[0] : null)

        anchors {
            top: true
            right: true
        }
        margins {
            top: manager.barVisible ? 52 : 14
            right: 14
        }
        implicitWidth: 340
        implicitHeight: Math.max(1, toastCol.implicitHeight)
        color: "transparent"
        exclusiveZone: 0
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

        ColumnLayout {
            id: toastCol
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            spacing: 8

            Repeater {
                model: popupScriptModel
                delegate: NotificationCard {
                    required property var modelData
                    Layout.fillWidth: true
                    entry: modelData
                    theme: manager.theme
                    compact: false
                    nowMs: manager.nowMs

                    onCloseToast: manager.hideToastById(modelData.notifId)
                    onDismissNotification: manager.dismissById(modelData.notifId)
                    onCloseAllToasts: manager.closeAllToasts()
                }
            }
        }
    }
}
