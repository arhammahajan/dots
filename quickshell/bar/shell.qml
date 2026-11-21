import QtQuick
import QtQuick.Layouts
import Quickshell
import Quickshell.Io
import Quickshell.Wayland
import Quickshell.Services.Pipewire
import Quickshell.Services.Mpris
import Quickshell.Services.UPower

ShellRoot {
    id: root

    // Pill bar palette
    readonly property color bgPill: Qt.rgba(22/255, 23/255, 26/255, 0.92)
    readonly property color bgCard: Qt.rgba(22/255, 23/255, 26/255, 0.96)
    readonly property color bgHover: Qt.rgba(255/255, 255/255, 255/255, 0.10)
    readonly property color bgWsActive: Qt.rgba(255/255, 255/255, 255/255, 0.16)
    readonly property color borderMain: Qt.rgba(216/255, 218/255, 221/255, 0.16)
    readonly property color contentMain: "#e2e4e9"
    readonly property color contentMuted: "#8c93a0"
    readonly property color accent: "#b7d4ed"
    readonly property color warnColor: "#e5c07b"
    readonly property color critColor: "#e06c75"
    readonly property string fontFamily: "JetBrainsMono Nerd Font"

    // Niri state
    property var workspaces: []
    property string focusedTitle: ""
    property int focusedWorkspaceId: -1
    property string kbLayout: "EN"

    // System stats state
    property int cpuPct: 0
    property int memPct: 0
    property string memUsed: "0G"
    property string memTotal: "0G"
    property int tempC: 0
    property string netSpeed: "0B/s"
    property real prevCpuTotal: 0
    property real prevCpuIdle: 0
    property real prevNetBytes: 0

    // Network, Bluetooth, Brightness & Power Profile state
    property string netType: "none"
    property string netName: ""
    property bool wifiEnabled: false
    property var wifiList: []
    property bool wifiScanning: false
    property string wifiPasswordTarget: ""
    property string netStatusMsg: ""
    property bool btPowered: false
    property var btDevices: []
    property bool btScanning: false
    property string btStatusMsg: ""
    property int brightnessPct: -1
    property string powerProfile: ""

    // Bar Visibility & Expandable Bar Menu state
    property bool barVisible: true
    property string activeMenu: "" // "", "network", "bluetooth", "audio", "notifications"
    property var activeMenuScreen: null

    onBarVisibleChanged: {
        if (!barVisible) {
            activeMenu = "";
            wifiPasswordTarget = "";
        }
    }

    readonly property var focusedScreen: {
        let focusedOutput = "";
        for (let i = 0; i < workspaces.length; i++) {
            if (workspaces[i].is_focused) {
                focusedOutput = workspaces[i].output || "";
                break;
            }
        }
        const screens = Quickshell.screens;
        if (focusedOutput.length > 0) {
            for (let i = 0; i < screens.length; i++) {
                if (screens[i].name === focusedOutput) return screens[i];
            }
        }
        return screens.length > 0 ? screens[0] : null;
    }

    IpcHandler {
        target: "bar"

        function toggle(): void {
            root.barVisible = !root.barVisible;
        }

        function show(): void {
            root.barVisible = true;
        }

        function hide(): void {
            root.barVisible = false;
        }

        function isVisible(): bool {
            return root.barVisible;
        }
    }

    IpcHandler {
        target: "notifications"

        function toggleDnd(): void {
            notifManager.toggleDnd();
        }

        function toggleCenter(): void {
            root.toggleMenu("notifications", root.focusedScreen);
        }

        function clearAll(): void {
            notifManager.clearAll();
        }

        function dismissToasts(): void {
            notifManager.closeAllToasts();
        }
    }

    NotificationPopup {
        id: notifManager
        theme: root
        targetScreen: root.focusedScreen
        barVisible: root.barVisible
        centerOpen: root.activeMenu === "notifications"
    }

    // Volume OSD state
    property bool osdVisible: false
    property real lastVolume: -1
    property bool lastMuted: false

    readonly property var outputSinks: {
        const nodes = Pipewire.nodes.values;
        const list = [];
        for (let i = 0; i < nodes.length; i++) {
            const n = nodes[i];
            if (n && n.isSink && !n.isStream) {
                list.push(n);
            }
        }
        return list;
    }

    SystemClock {
        id: clock
        precision: SystemClock.Minutes
    }

    // Track PipeWire default sink
    PwObjectTracker {
        objects: [Pipewire.defaultAudioSink]
    }

    readonly property var audioSink: Pipewire.defaultAudioSink
    readonly property real sinkVolume: audioSink && audioSink.audio ? audioSink.audio.volume : 0
    readonly property bool sinkMuted: audioSink && audioSink.audio ? audioSink.audio.muted : false

    onSinkVolumeChanged: {
        if (lastVolume >= 0 && Math.abs(sinkVolume - lastVolume) > 0.001) {
            osdVisible = true;
            osdTimer.restart();
        }
        lastVolume = sinkVolume;
    }

    onSinkMutedChanged: {
        if (lastVolume >= 0 && sinkMuted !== lastMuted) {
            osdVisible = true;
            osdTimer.restart();
        }
        lastMuted = sinkMuted;
    }

    Timer {
        id: osdTimer
        interval: 1600
        onTriggered: root.osdVisible = false
    }

    function refreshNiri() {
        wsProc.running = true;
        winProc.running = true;
        kbProc.running = true;
    }

    function formatSpeed(bytesPerSec) {
        if (bytesPerSec >= 1048576) return (bytesPerSec / 1048576).toFixed(1) + "MB/s";
        if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(1) + "kB/s";
        return Math.round(bytesPerSec) + "B/s";
    }

    function formatMem(kib) {
        if (kib >= 1048576) return (kib / 1048576).toFixed(1) + "G";
        return Math.round(kib / 1024) + "M";
    }

    Process {
        id: wsProc
        command: ["niri", "msg", "-j", "workspaces"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const list = JSON.parse(this.text);
                    list.sort((a, b) => a.idx - b.idx);
                    root.workspaces = list;
                    for (let i = 0; i < list.length; i++) {
                        if (list[i].is_focused) {
                            root.focusedWorkspaceId = list[i].id;
                            break;
                        }
                    }
                } catch (e) {}
            }
        }
    }

    Process {
        id: winProc
        command: ["niri", "msg", "-j", "focused-window"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const w = JSON.parse(this.text);
                    root.focusedTitle = w && w.title ? w.title : "";
                } catch (e) {
                    root.focusedTitle = "";
                }
            }
        }
    }

    Process {
        id: kbProc
        command: ["niri", "msg", "-j", "keyboard-layouts"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    const kb = JSON.parse(this.text);
                    if (kb && kb.names && kb.current_idx >= 0 && kb.current_idx < kb.names.length) {
                        const name = kb.names[kb.current_idx];
                        const m = name.match(/\(([^)]+)\)/);
                        root.kbLayout = m ? m[1].slice(0, 2).toUpperCase() : name.slice(0, 2).toUpperCase();
                    }
                } catch (e) {}
            }
        }
    }

    Process {
        id: niriEvents
        command: ["niri", "msg", "-j", "event-stream"]
        running: true
        stdout: SplitParser {
            onRead: data => {
                if (data.indexOf("Workspace") !== -1 || data.indexOf("WindowFocus") !== -1 ||
                    data.indexOf("WindowOpened") !== -1 || data.indexOf("WindowClosed") !== -1 ||
                    data.indexOf("KeyboardLayout") !== -1) {
                    root.refreshNiri();
                }
            }
        }
        onExited: restartNiriTimer.start()
    }

    Timer {
        id: restartNiriTimer
        interval: 1000
        onTriggered: niriEvents.running = true
    }

    // Fast polling (2s): CPU, RAM, Temp, Net Speed
    Process {
        id: statsProc
        command: ["sh", "-c",
            "awk '/^cpu /{print $2,$3,$4,$5,$6,$7,$8}' /proc/stat; " +
            "awk '/MemTotal/{t=$2} /MemAvailable/{a=$2} END{printf \"%.0f %.0f\\n\", (t>a ? t-a : 0), t}' /proc/meminfo; " +
            "t=$(for d in /sys/class/hwmon/hwmon*; do n=$(cat \"$d/name\" 2>/dev/null); case \"$n\" in k10temp|coretemp|zenpower|acpitz) cat \"$d/temp1_input\" 2>/dev/null && break;; esac; done); " +
            "[ -z \"$t\" ] && t=$(cat /sys/class/thermal/thermal_zone0/temp 2>/dev/null || echo 0); echo $((t/1000)); " +
            "awk 'NR>2 && $1!~/^lo:/{rx+=$2; tx+=$10} END{printf \"%.0f\\n\", rx+tx}' /proc/net/dev"
        ]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                const lines = this.text.trim().split("\n");
                if (lines.length < 4) return;

                const cpuParts = lines[0].trim().split(/\s+/).map(Number);
                if (cpuParts.length >= 4) {
                    const idle = cpuParts[3] + (cpuParts[4] || 0);
                    const total = cpuParts.reduce((a, b) => a + b, 0);
                    if (root.prevCpuTotal > 0 && total > root.prevCpuTotal) {
                        const dt = total - root.prevCpuTotal;
                        const di = idle - root.prevCpuIdle;
                        root.cpuPct = Math.max(0, Math.min(100, Math.round((dt - di) * 100 / dt)));
                    }
                    root.prevCpuTotal = total;
                    root.prevCpuIdle = idle;
                }

                const memParts = lines[1].trim().split(/\s+/).map(Number);
                if (memParts.length >= 2 && memParts[1] > 0) {
                    const usedKb = memParts[0];
                    const totalKb = memParts[1];
                    root.memPct = Math.max(0, Math.min(100, Math.round(usedKb * 100 / totalKb)));
                    root.memUsed = root.formatMem(usedKb);
                    root.memTotal = root.formatMem(totalKb);
                }
                root.tempC = parseInt(lines[2]) || 0;

                const netBytes = parseFloat(lines[3]) || 0;
                if (root.prevNetBytes > 0 && netBytes >= root.prevNetBytes) {
                    root.netSpeed = root.formatSpeed((netBytes - root.prevNetBytes) / 2.0);
                }
                root.prevNetBytes = netBytes;
            }
        }
    }

    Timer {
        interval: 2000
        running: true
        repeat: true
        onTriggered: statsProc.running = true
    }

    // Slower polling (5s): Network connection, Bluetooth, Brightness, Power Profile
    Process {
        id: netProc
        command: ["sh", "-c", "nmcli -t -f TYPE,STATE,CONNECTION dev 2>/dev/null | grep ':connected:' | head -n1"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                const line = this.text.trim();
                if (!line) {
                    root.netType = "none";
                    root.netName = "";
                } else {
                    const parts = line.split(":");
                    root.netType = parts[0] === "wifi" ? "wifi" : "ethernet";
                    root.netName = parts.slice(2).join(":");
                }
            }
        }
    }

    Process {
        id: btProc
        command: ["sh", "-c", "bluetoothctl show 2>/dev/null | awk '/Powered:/{print $2; exit}'"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.btPowered = (this.text.trim() === "yes")
        }
    }

    Process {
        id: brightProc
        command: ["sh", "-c", "brightnessctl -c backlight -m 2>/dev/null | head -n1 | cut -d, -f4 | tr -d '%'"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: {
                const val = parseInt(this.text.trim());
                root.brightnessPct = isNaN(val) ? -1 : val;
            }
        }
    }

    Process {
        id: profileProc
        command: ["sh", "-c", "command -v powerprofilesctl >/dev/null 2>&1 && powerprofilesctl get || true"]
        running: true
        stdout: StdioCollector {
            onStreamFinished: root.powerProfile = this.text.trim()
        }
    }

    Timer {
        interval: 5000
        running: true
        repeat: true
        onTriggered: {
            netProc.running = true;
            btProc.running = true;
            brightProc.running = true;
            profileProc.running = true;
        }
    }

    Process { id: actionProc }

    function runCmd(cmd) {
        actionProc.command = cmd;
        actionProc.running = true;
    }

    function toggleMenu(menuName, screen) {
        if (root.activeMenu === menuName && root.activeMenuScreen === screen) {
            root.activeMenu = "";
            root.wifiPasswordTarget = "";
            return;
        }
        root.activeMenuScreen = screen;
        root.activeMenu = menuName;
        root.wifiPasswordTarget = "";
        root.netStatusMsg = "";
        root.btStatusMsg = "";
        if (menuName === "network") {
            wifiStateProc.running = true;
            wifiListProc.running = true;
        } else if (menuName === "bluetooth") {
            btProc.running = true;
            btListProc.running = true;
        }
    }

    // Network Menu Processes
    Process {
        id: wifiStateProc
        command: ["sh", "-c", "nmcli radio wifi 2>/dev/null || echo disabled"]
        stdout: StdioCollector {
            onStreamFinished: root.wifiEnabled = (this.text.trim() === "enabled")
        }
    }

    Process {
        id: wifiListProc
        command: ["sh", "-c", "nmcli -t -f IN-USE,SSID,SIGNAL,SECURITY dev wifi list --rescan no 2>/dev/null | head -n 20"]
        stdout: StdioCollector {
            onStreamFinished: {
                const lines = this.text.trim().split("\n");
                const items = [];
                const seen = {};
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i];
                    if (!line) continue;
                    const parts = line.split(":");
                    if (parts.length < 4) continue;
                    const inUse = parts[0] === "*";
                    const ssid = parts[1];
                    if (!ssid || seen[ssid]) continue;
                    seen[ssid] = true;
                    const signal = parseInt(parts[2]) || 0;
                    const security = parts.slice(3).join(":");
                    items.push({ inUse: inUse, ssid: ssid, signal: signal, security: security });
                }
                root.wifiList = items;
                root.wifiScanning = false;
            }
        }
    }

    Process {
        id: wifiActionProc
        stdout: StdioCollector {
            onStreamFinished: {
                netProc.running = true;
                wifiStateProc.running = true;
                wifiListProc.running = true;
            }
        }
        stderr: StdioCollector {
            onStreamFinished: {
                const err = this.text.trim();
                if (err.length > 0) root.netStatusMsg = err.split("\n")[0];
            }
        }
    }

    // Bluetooth Menu Processes
    Process {
        id: btListProc
        command: ["sh", "-c",
            "conn=$(bluetoothctl devices Connected 2>/dev/null | awk '{print $2}'); " +
            "bluetoothctl devices 2>/dev/null | while read -r _ mac name; do " +
            "  [ -z \"$mac\" ] && continue; " +
            "  is_conn=0; echo \"$conn\" | grep -qx \"$mac\" && is_conn=1; " +
            "  echo \"$is_conn|$mac|$name\"; " +
            "done"
        ]
        stdout: StdioCollector {
            onStreamFinished: {
                const lines = this.text.trim().split("\n");
                const items = [];
                for (let i = 0; i < lines.length; i++) {
                    const line = lines[i];
                    if (!line) continue;
                    const idx1 = line.indexOf("|");
                    const idx2 = line.indexOf("|", idx1 + 1);
                    if (idx1 < 0 || idx2 < 0) continue;
                    const connected = line.slice(0, idx1) === "1";
                    const mac = line.slice(idx1 + 1, idx2);
                    const name = line.slice(idx2 + 1) || mac;
                    items.push({ connected: connected, mac: mac, name: name });
                }
                items.sort((a, b) => (b.connected ? 1 : 0) - (a.connected ? 1 : 0));
                root.btDevices = items;
                root.btScanning = false;
            }
        }
    }

    Process {
        id: btActionProc
        onExited: {
            btProc.running = true;
            btListProc.running = true;
        }
    }

    Component.onCompleted: refreshNiri()

    // Floating Pill Bar per screen
    Variants {
        model: Quickshell.screens

        PanelWindow {
            id: barWindow
            required property var modelData
            screen: modelData
            visible: root.barVisible

            anchors {
                top: true
                left: true
                right: true
            }
            margins {
                top: 8
                left: 14
                right: 14
            }
            implicitHeight: 36
            color: "transparent"

            readonly property var screenWorkspaces: {
                const out = modelData.name;
                return root.workspaces.filter(ws => ws.output === out);
            }

            readonly property bool screenHasFocus: {
                for (let i = 0; i < screenWorkspaces.length; i++) {
                    if (screenWorkspaces[i].is_focused) return true;
                }
                return false;
            }

            // Outer Pill Container
            Rectangle {
                anchors.fill: parent
                radius: 10
                color: root.bgPill
                border.width: 1
                border.color: root.borderMain

                // Left: Launcher + Workspaces + Active Window
                RowLayout {
                    anchors.left: parent.left
                    anchors.leftMargin: 12
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 6

                    // OS / App Launcher icon
                    Rectangle {
                        implicitWidth: 22
                        implicitHeight: 22
                        radius: 7
                        color: launcherMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: ""
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 14
                        }

                        MouseArea {
                            id: launcherMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.runCmd(["fuzzel", "--config", "/home/arhammahajan/dev/dots/fuzzel/fuzzel.ini"])
                        }
                    }

                    // Workspaces
                    RowLayout {
                        spacing: 3

                        Repeater {
                            model: barWindow.screenWorkspaces
                            delegate: Rectangle {
                                required property var modelData
                                readonly property bool isActive: modelData.is_active || modelData.is_focused

                                implicitWidth: 24
                                implicitHeight: 22
                                radius: 7
                                color: isActive ? root.bgWsActive : (wsMouse.containsMouse ? root.bgHover : "transparent")

                                Behavior on color { ColorAnimation { duration: 120 } }

                                Text {
                                    anchors.centerIn: parent
                                    text: modelData.idx
                                    color: modelData.is_urgent ? root.critColor : (isActive ? root.contentMain : root.contentMuted)
                                    font.family: root.fontFamily
                                    font.pixelSize: 12
                                    font.weight: isActive ? Font.DemiBold : Font.Normal
                                }

                                MouseArea {
                                    id: wsMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: root.runCmd(["niri", "msg", "action", "focus-workspace", String(modelData.idx)])
                                }
                            }
                        }
                    }

                    // Focused Window Title
                    RowLayout {
                        visible: barWindow.screenHasFocus && root.focusedTitle.length > 0
                        spacing: 6
                        Layout.leftMargin: 4

                        Rectangle {
                            implicitWidth: 18
                            implicitHeight: 18
                            radius: 4
                            color: Qt.rgba(183/255, 212/255, 237/255, 0.14)

                            Text {
                                anchors.centerIn: parent
                                text: "󰆍"
                                color: root.accent
                                font.family: root.fontFamily
                                font.pixelSize: 11
                            }
                        }

                        Text {
                            text: root.focusedTitle
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 12
                            elide: Text.ElideRight
                            Layout.maximumWidth: 240
                        }
                    }
                }

                // Center: CPU, Temp, RAM, Time, Date
                RowLayout {
                    anchors.centerIn: parent
                    spacing: 14

                    Text {
                        text: "󰍛 " + root.cpuPct + "%"
                        color: root.cpuPct >= 85 ? root.critColor : root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 12
                    }

                    Text {
                        visible: root.tempC > 0
                        text: "󰔏 " + root.tempC + "°C"
                        color: root.tempC >= 85 ? root.critColor : (root.tempC >= 70 ? root.warnColor : root.contentMain)
                        font.family: root.fontFamily
                        font.pixelSize: 12
                    }

                    Text {
                        text: "󰘚 " + root.memUsed + "/" + root.memTotal
                        color: root.memPct >= 85 ? root.warnColor : root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 12
                    }

                    Text {
                        text: "󰥔 " + Qt.formatDateTime(clock.date, "HH:mm")
                        color: root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        font.weight: Font.Medium
                    }

                    Text {
                        text: "󰃭 " + Qt.formatDateTime(clock.date, "MM-dd")
                        color: root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 12
                    }
                }

                // Right: Media, Net Speed, Net, BT, Layout, Vol, Brightness, Profile, Battery, Power
                RowLayout {
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 4

                    // Media (only when active player has a title)
                    readonly property var activePlayer: {
                        const list = Mpris.players.values;
                        for (let i = 0; i < list.length; i++) {
                            if (list[i].trackTitle && list[i].trackTitle.length > 0) return list[i];
                        }
                        return null;
                    }

                    Rectangle {
                        visible: parent.activePlayer !== null
                        implicitWidth: mediaText.implicitWidth + 12
                        implicitHeight: 22
                        radius: 7
                        color: mediaMouse.containsMouse ? root.bgHover : Qt.rgba(183/255, 212/255, 237/255, 0.10)
                        Layout.rightMargin: 2

                        Text {
                            id: mediaText
                            anchors.centerIn: parent
                            readonly property var p: parent.parent.activePlayer
                            text: p ? ((p.playbackState === MprisPlaybackState.Playing ? "󰐊 " : "󰏤 ") + p.trackTitle) : ""
                            color: root.accent
                            font.family: root.fontFamily
                            font.pixelSize: 11
                            elide: Text.ElideRight
                            width: Math.min(implicitWidth, 160)
                        }

                        MouseArea {
                            id: mediaMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.RightButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: mouse => {
                                const p = parent.parent.activePlayer;
                                if (!p) return;
                                if (mouse.button === Qt.LeftButton) p.togglePlaying();
                                else p.next();
                            }
                        }
                    }

                    // Network Speed
                    Text {
                        text: "󰹹 " + root.netSpeed
                        color: root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        Layout.rightMargin: 4
                    }

                    // Network Icon
                    Rectangle {
                        implicitWidth: 22
                        implicitHeight: 22
                        radius: 7
                        color: root.activeMenu === "network" ? root.bgWsActive : (netMouse.containsMouse ? root.bgHover : "transparent")

                        Text {
                            anchors.centerIn: parent
                            text: root.netType === "wifi" ? "󰤨" : (root.netType === "ethernet" ? "󰈀" : "󰤭")
                            color: root.netType === "none" ? root.contentMuted : root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: netMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.toggleMenu("network", barWindow.modelData)
                        }
                    }

                    // Bluetooth Icon
                    Rectangle {
                        implicitWidth: 22
                        implicitHeight: 22
                        radius: 7
                        color: root.activeMenu === "bluetooth" ? root.bgWsActive : (btMouse.containsMouse ? root.bgHover : "transparent")

                        Text {
                            anchors.centerIn: parent
                            text: root.btPowered ? "󰂯" : "󰂲"
                            color: root.btPowered ? root.contentMain : root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: btMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.toggleMenu("bluetooth", barWindow.modelData)
                        }
                    }

                    // Keyboard Layout
                    Rectangle {
                        implicitWidth: kbText.implicitWidth + 10
                        implicitHeight: 22
                        radius: 7
                        color: kbMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            id: kbText
                            anchors.centerIn: parent
                            text: "󰌌 " + root.kbLayout
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: kbMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.runCmd(["niri", "msg", "action", "switch-layout", "next"])
                        }
                    }

                    // Volume
                    Rectangle {
                        implicitWidth: volText.implicitWidth + 10
                        implicitHeight: 22
                        radius: 7
                        color: root.activeMenu === "audio" ? root.bgWsActive : (volMouse.containsMouse ? root.bgHover : "transparent")

                        Text {
                            id: volText
                            anchors.centerIn: parent
                            readonly property int volPct: Math.round(root.sinkVolume * 100)
                            text: {
                                const icon = root.sinkMuted ? "󰖁" : (root.sinkVolume < 0.33 ? "󰕿" : (root.sinkVolume < 0.66 ? "󰖀" : "󰕾"));
                                return icon + " " + volPct + "%";
                            }
                            color: root.activeMenu === "audio" ? root.accent : (root.sinkMuted ? root.contentMuted : root.contentMain)
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: volMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.MiddleButton | Qt.RightButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: mouse => {
                                if (mouse.button === Qt.LeftButton) {
                                    root.toggleMenu("audio", barWindow.modelData);
                                } else if (mouse.button === Qt.MiddleButton && root.audioSink && root.audioSink.audio) {
                                    root.audioSink.audio.muted = !root.audioSink.audio.muted;
                                } else if (mouse.button === Qt.RightButton) {
                                    root.runCmd(["pavucontrol"]);
                                }
                            }
                            onWheel: wheel => {
                                if (!root.audioSink || !root.audioSink.audio) return;
                                const step = wheel.angleDelta.y > 0 ? 0.05 : -0.05;
                                root.audioSink.audio.volume = Math.max(0, Math.min(1.0, root.audioSink.audio.volume + step));
                            }
                        }
                    }

                    // Brightness (visible only if a backlight device exists)
                    Rectangle {
                        visible: root.brightnessPct >= 0
                        implicitWidth: brightText.implicitWidth + 10
                        implicitHeight: 22
                        radius: 7
                        color: brightMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            id: brightText
                            anchors.centerIn: parent
                            text: "󰃠 " + root.brightnessPct + "%"
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: brightMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onWheel: wheel => {
                                root.runCmd(["brightnessctl", "-c", "backlight", "set", wheel.angleDelta.y > 0 ? "+5%" : "5%-"]);
                                brightRefreshTimer.restart();
                            }
                        }
                    }

                    Timer {
                        id: brightRefreshTimer
                        interval: 200
                        onTriggered: brightProc.running = true
                    }

                    // Power Profile
                    Rectangle {
                        visible: root.powerProfile.length > 0
                        implicitWidth: 22
                        implicitHeight: 22
                        radius: 7
                        color: ppMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: root.powerProfile === "performance" ? "󰓅" : (root.powerProfile === "power-saver" ? "󰾆" : "✦")
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: ppMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.runCmd(["/home/arhammahajan/dev/dots/scripts/change-power-profile.sh"]);
                                profileRefreshTimer.restart();
                            }
                        }
                    }

                    Timer {
                        id: profileRefreshTimer
                        interval: 300
                        onTriggered: profileProc.running = true
                    }

                    // Battery (visible on laptop)
                    Rectangle {
                        visible: UPower.displayDevice.isLaptopBattery
                        implicitWidth: batText.implicitWidth + 8
                        implicitHeight: 22
                        color: "transparent"

                        readonly property int pct: Math.round(UPower.displayDevice.percentage * 100)
                        readonly property bool charging: UPower.displayDevice.state === UPowerDeviceState.Charging || UPower.displayDevice.state === UPowerDeviceState.FullyCharged

                        Text {
                            id: batText
                            anchors.centerIn: parent
                            text: {
                                const p = parent.pct;
                                if (parent.charging) return "󰂄 " + p + "%";
                                const icon = p >= 90 ? "󰁹" : (p >= 70 ? "󰂁" : (p >= 50 ? "󰁿" : (p >= 30 ? "󰁽" : "󰁻")));
                                return icon + " " + p + "%";
                            }
                            color: {
                                if (parent.charging) return root.accent;
                                if (parent.pct <= 15) return root.critColor;
                                if (parent.pct <= 25) return root.warnColor;
                                return root.contentMain;
                            }
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }
                    }

                    // Notifications Bell
                    Rectangle {
                        implicitWidth: notifBellRow.implicitWidth + 10
                        implicitHeight: 22
                        radius: 7
                        color: root.activeMenu === "notifications" ? root.bgWsActive : (notifBellMouse.containsMouse ? root.bgHover : "transparent")

                        RowLayout {
                            id: notifBellRow
                            anchors.centerIn: parent
                            spacing: 4

                            Text {
                                text: notifManager.dnd ? "󰂛" : "󰂚"
                                color: {
                                    if (notifManager.dnd) return root.contentMuted;
                                    if (root.activeMenu === "notifications" || notifManager.historyCount > 0) return root.accent;
                                    return root.contentMain;
                                }
                                font.family: root.fontFamily
                                font.pixelSize: 13
                            }

                            Text {
                                visible: notifManager.historyCount > 0 && !notifManager.dnd
                                text: notifManager.historyCount > 99 ? "99+" : String(notifManager.historyCount)
                                color: root.accent
                                font.family: root.fontFamily
                                font.pixelSize: 11
                                font.weight: Font.Medium
                            }
                        }

                        MouseArea {
                            id: notifBellMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            acceptedButtons: Qt.LeftButton | Qt.RightButton
                            cursorShape: Qt.PointingHandCursor
                            onClicked: mouse => {
                                if (mouse.button === Qt.RightButton) {
                                    notifManager.toggleDnd();
                                } else {
                                    root.toggleMenu("notifications", barWindow.modelData);
                                }
                            }
                        }
                    }

                    // Power Menu Button
                    Rectangle {
                        implicitWidth: 24
                        implicitHeight: 22
                        radius: 7
                        color: pwrMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "󰐥"
                            color: pwrMouse.containsMouse ? root.critColor : root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: pwrMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.runCmd(["/home/arhammahajan/dev/dots/scripts/wlogout.sh"])
                        }
                    }
                }
            }
        }
    }

    // Expandable Bar Menu (Network / Bluetooth / Audio / Notifications)
    PanelWindow {
        id: menuWindow
        visible: root.activeMenu !== ""
        screen: root.activeMenuScreen ? root.activeMenuScreen : Quickshell.screens[0]

        anchors {
            top: true
            right: true
        }
        margins {
            top: root.barVisible ? 0 : 14
            right: root.barVisible ? 24 : 14
        }
        implicitWidth: root.activeMenu === "notifications" ? 340 : 310
        implicitHeight: menuCol.implicitHeight + 24
        color: "transparent"
        exclusiveZone: 0
        WlrLayershell.layer: WlrLayer.Overlay
        WlrLayershell.keyboardFocus: root.wifiPasswordTarget.length > 0 ? WlrKeyboardFocus.OnDemand : WlrKeyboardFocus.None

        Rectangle {
            id: menuCard
            anchors.fill: parent
            topLeftRadius: root.barVisible ? 0 : 10
            topRightRadius: root.barVisible ? 0 : 10
            bottomLeftRadius: 10
            bottomRightRadius: 10
            color: root.bgPill
            border.width: 1
            border.color: root.borderMain
            focus: root.wifiPasswordTarget.length > 0
            Keys.onEscapePressed: root.activeMenu = ""

            // Seamlessly blend top edge into the bar's bottom border
            Rectangle {
                visible: root.barVisible
                anchors.top: parent.top
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 1
                anchors.rightMargin: 1
                height: 2
                color: root.bgPill
            }

            ColumnLayout {
                id: menuCol
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.margins: 12
                spacing: 8

                // Header
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 8

                    Text {
                        text: {
                            if (root.activeMenu === "network") return "󰤨  Network";
                            if (root.activeMenu === "bluetooth") return "󰂯  Bluetooth";
                            if (root.activeMenu === "notifications") return "󰂚  Notifications";
                            return "󰕾  Audio";
                        }
                        color: root.contentMain
                        font.family: root.fontFamily
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                    }

                    Item { Layout.fillWidth: true }

                    // Clear All Notifications Button
                    Rectangle {
                        visible: root.activeMenu === "notifications" && notifManager.historyCount > 0
                        implicitWidth: clearNotifText.implicitWidth + 12
                        implicitHeight: 22
                        radius: 6
                        color: clearNotifMouse.containsMouse ? root.bgHover : Qt.rgba(255/255, 255/255, 255/255, 0.05)

                        Text {
                            id: clearNotifText
                            anchors.centerIn: parent
                            text: "Clear"
                            color: clearNotifMouse.containsMouse ? root.critColor : root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: clearNotifMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: notifManager.clearAll()
                        }
                    }

                    // Do Not Disturb label + toggle pill
                    Text {
                        visible: root.activeMenu === "notifications"
                        text: "DND"
                        color: notifManager.dnd ? root.accent : root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 10
                    }

                    Rectangle {
                        visible: root.activeMenu === "notifications"
                        implicitWidth: 38
                        implicitHeight: 20
                        radius: 10
                        color: notifManager.dnd ? Qt.rgba(183/255, 212/255, 237/255, 0.28) : Qt.rgba(255/255, 255/255, 255/255, 0.10)
                        border.width: 1
                        border.color: notifManager.dnd ? root.accent : root.borderMain

                        Rectangle {
                            width: 14
                            height: 14
                            radius: 7
                            y: 3
                            x: notifManager.dnd ? 21 : 3
                            color: notifManager.dnd ? root.accent : root.contentMuted
                            Behavior on x { NumberAnimation { duration: 120 } }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: notifManager.toggleDnd()
                        }
                    }

                    // Rescan button
                    Rectangle {
                        visible: root.activeMenu === "network" || root.activeMenu === "bluetooth"
                        implicitWidth: 24
                        implicitHeight: 24
                        radius: 6
                        color: scanMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: (root.activeMenu === "network" ? root.wifiScanning : root.btScanning) ? "󰑐" : "󰑐"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 13
                        }

                        MouseArea {
                            id: scanMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activeMenu === "network") {
                                    root.wifiScanning = true;
                                    wifiActionProc.command = ["nmcli", "dev", "wifi", "rescan"];
                                    wifiActionProc.running = true;
                                } else {
                                    root.btScanning = true;
                                    btActionProc.command = ["bluetoothctl", "--timeout", "5", "scan", "on"];
                                    btActionProc.running = true;
                                }
                            }
                        }
                    }

                    // Power / Wi-Fi Toggle Pill
                    Rectangle {
                        visible: root.activeMenu === "network" || root.activeMenu === "bluetooth"
                        implicitWidth: 38
                        implicitHeight: 20
                        radius: 10
                        readonly property bool isOn: root.activeMenu === "network" ? root.wifiEnabled : root.btPowered
                        color: isOn ? Qt.rgba(183/255, 212/255, 237/255, 0.28) : Qt.rgba(255/255, 255/255, 255/255, 0.10)
                        border.width: 1
                        border.color: isOn ? root.accent : root.borderMain

                        Rectangle {
                            width: 14
                            height: 14
                            radius: 7
                            y: 3
                            x: parent.isOn ? 21 : 3
                            color: parent.isOn ? root.accent : root.contentMuted
                            Behavior on x { NumberAnimation { duration: 120 } }
                        }

                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.activeMenu === "network") {
                                    wifiActionProc.command = ["nmcli", "radio", "wifi", root.wifiEnabled ? "off" : "on"];
                                    wifiActionProc.running = true;
                                } else {
                                    btActionProc.command = ["bluetoothctl", "power", root.btPowered ? "off" : "on"];
                                    btActionProc.running = true;
                                }
                            }
                        }
                    }

                    // Close button
                    Rectangle {
                        implicitWidth: 24
                        implicitHeight: 24
                        radius: 6
                        color: closeMouse.containsMouse ? root.bgHover : "transparent"

                        Text {
                            anchors.centerIn: parent
                            text: "󰅖"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }

                        MouseArea {
                            id: closeMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: root.activeMenu = ""
                        }
                    }
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 1
                    color: root.borderMain
                }

                // === NETWORK CONTENT ===
                ColumnLayout {
                    visible: root.activeMenu === "network"
                    Layout.fillWidth: true
                    spacing: 6

                    // Active connection summary
                    RowLayout {
                        visible: root.netType !== "none"
                        Layout.fillWidth: true
                        spacing: 6

                        Text {
                            text: root.netType === "wifi" ? "󰤨" : "󰈀"
                            color: root.accent
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }
                        Text {
                            text: "Connected: " + root.netName
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 12
                            elide: Text.ElideRight
                            Layout.fillWidth: true
                        }
                    }

                    // Inline Wi-Fi password prompt
                    RowLayout {
                        visible: root.wifiPasswordTarget.length > 0
                        Layout.fillWidth: true
                        spacing: 6

                        Rectangle {
                            Layout.fillWidth: true
                            height: 28
                            radius: 6
                            color: Qt.rgba(255/255, 255/255, 255/255, 0.06)
                            border.width: 1
                            border.color: root.accent

                            TextInput {
                                id: wifiPassInput
                                anchors.fill: parent
                                anchors.leftMargin: 8
                                anchors.rightMargin: 8
                                verticalAlignment: TextInput.AlignVCenter
                                color: root.contentMain
                                font.family: root.fontFamily
                                font.pixelSize: 12
                                echoMode: TextInput.Password
                                clip: true
                                onAccepted: {
                                    wifiActionProc.command = ["nmcli", "dev", "wifi", "connect", root.wifiPasswordTarget, "password", text];
                                    wifiActionProc.running = true;
                                    text = "";
                                    root.wifiPasswordTarget = "";
                                }
                            }
                        }

                        Rectangle {
                            implicitWidth: 54
                            implicitHeight: 28
                            radius: 6
                            color: root.bgWsActive

                            Text {
                                anchors.centerIn: parent
                                text: "Join"
                                color: root.contentMain
                                font.family: root.fontFamily
                                font.pixelSize: 11
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    wifiActionProc.command = ["nmcli", "dev", "wifi", "connect", root.wifiPasswordTarget, "password", wifiPassInput.text];
                                    wifiActionProc.running = true;
                                    wifiPassInput.text = "";
                                    root.wifiPasswordTarget = "";
                                }
                            }
                        }
                    }

                    Text {
                        visible: root.netStatusMsg.length > 0
                        text: root.netStatusMsg
                        color: root.warnColor
                        font.family: root.fontFamily
                        font.pixelSize: 11
                        wrapMode: Text.Wrap
                        Layout.fillWidth: true
                    }

                    Text {
                        visible: !root.wifiEnabled
                        text: "Wi-Fi is turned off"
                        color: root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        Layout.topMargin: 4
                        Layout.bottomMargin: 4
                    }

                    Text {
                        visible: root.wifiEnabled && root.wifiList.length === 0
                        text: "No Wi-Fi networks found"
                        color: root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        Layout.topMargin: 4
                        Layout.bottomMargin: 4
                    }

                    Flickable {
                        visible: root.wifiEnabled && root.wifiList.length > 0
                        Layout.fillWidth: true
                        implicitHeight: Math.min(wifiCol.implicitHeight, 220)
                        contentHeight: wifiCol.implicitHeight
                        clip: true

                        ColumnLayout {
                            id: wifiCol
                            width: parent.width
                            spacing: 3

                            Repeater {
                                model: root.wifiList
                                delegate: Rectangle {
                                    required property var modelData
                                    Layout.fillWidth: true
                                    implicitHeight: 30
                                    radius: 7
                                    color: modelData.inUse ? root.bgWsActive : (wifiItemMouse.containsMouse ? root.bgHover : "transparent")

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 8
                                        anchors.rightMargin: 8
                                        spacing: 8

                                        Text {
                                            text: modelData.signal >= 70 ? "󰤨" : (modelData.signal >= 45 ? "󰤥" : (modelData.signal >= 25 ? "󰤢" : "󰤟"))
                                            color: modelData.inUse ? root.accent : root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 13
                                        }

                                        Text {
                                            text: modelData.ssid
                                            color: root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 12
                                            elide: Text.ElideRight
                                            Layout.fillWidth: true
                                        }

                                        Text {
                                            visible: modelData.security && modelData.security !== "--"
                                            text: "󰌾"
                                            color: root.contentMuted
                                            font.family: root.fontFamily
                                            font.pixelSize: 11
                                        }

                                        Text {
                                            text: modelData.inUse ? "✓" : modelData.signal + "%"
                                            color: modelData.inUse ? root.accent : root.contentMuted
                                            font.family: root.fontFamily
                                            font.pixelSize: 11
                                        }
                                    }

                                    MouseArea {
                                        id: wifiItemMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            root.netStatusMsg = "";
                                            if (modelData.inUse) {
                                                wifiActionProc.command = ["nmcli", "con", "down", modelData.ssid];
                                                wifiActionProc.running = true;
                                            } else if (modelData.security && modelData.security !== "--") {
                                                root.wifiPasswordTarget = modelData.ssid;
                                                wifiPassInput.forceActiveFocus();
                                            } else {
                                                wifiActionProc.command = ["nmcli", "dev", "wifi", "connect", modelData.ssid];
                                                wifiActionProc.running = true;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 28
                        radius: 7
                        color: nmtuiMouse.containsMouse ? root.bgHover : Qt.rgba(255/255, 255/255, 255/255, 0.05)

                        Text {
                            anchors.centerIn: parent
                            text: "Open nmtui…"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 11
                        }

                        MouseArea {
                            id: nmtuiMouse
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                root.activeMenu = "";
                                root.runCmd(["kitty", "--title", "nmtui", "-o", "background_opacity=0.9", "nmtui", "connect"]);
                            }
                        }
                    }
                }

                // === BLUETOOTH CONTENT ===
                ColumnLayout {
                    visible: root.activeMenu === "bluetooth"
                    Layout.fillWidth: true
                    spacing: 6

                    Text {
                        visible: !root.btPowered
                        text: "Bluetooth is turned off"
                        color: root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        Layout.topMargin: 4
                        Layout.bottomMargin: 4
                    }

                    Text {
                        visible: root.btPowered && root.btDevices.length === 0
                        text: "No paired devices found"
                        color: root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 12
                        Layout.topMargin: 4
                        Layout.bottomMargin: 4
                    }

                    Flickable {
                        visible: root.btPowered && root.btDevices.length > 0
                        Layout.fillWidth: true
                        implicitHeight: Math.min(btCol.implicitHeight, 240)
                        contentHeight: btCol.implicitHeight
                        clip: true

                        ColumnLayout {
                            id: btCol
                            width: parent.width
                            spacing: 3

                            Repeater {
                                model: root.btDevices
                                delegate: Rectangle {
                                    required property var modelData
                                    Layout.fillWidth: true
                                    implicitHeight: 32
                                    radius: 7
                                    color: modelData.connected ? root.bgWsActive : (btItemMouse.containsMouse ? root.bgHover : "transparent")

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 8
                                        anchors.rightMargin: 8
                                        spacing: 8

                                        Text {
                                            text: modelData.connected ? "󰂱" : "󰂯"
                                            color: modelData.connected ? root.accent : root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 13
                                        }

                                        Text {
                                            text: modelData.name
                                            color: root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 12
                                            elide: Text.ElideRight
                                            Layout.fillWidth: true
                                        }

                                        Text {
                                            text: modelData.connected ? "Connected" : "Connect"
                                            color: modelData.connected ? root.accent : root.contentMuted
                                            font.family: root.fontFamily
                                            font.pixelSize: 11
                                        }
                                    }

                                    MouseArea {
                                        id: btItemMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: {
                                            btActionProc.command = ["bluetoothctl", modelData.connected ? "disconnect" : "connect", modelData.mac];
                                            btActionProc.running = true;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Audio Menu Content
                ColumnLayout {
                    Layout.fillWidth: true
                    visible: root.activeMenu === "audio"
                    spacing: 8

                    // Volume + Mute Row
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: 8

                        Rectangle {
                            implicitWidth: 26
                            implicitHeight: 26
                            radius: 7
                            color: menuMuteMouse.containsMouse ? root.bgHover : Qt.rgba(255/255, 255/255, 255/255, 0.05)

                            Text {
                                anchors.centerIn: parent
                                text: root.sinkMuted ? "󰖁" : "󰕾"
                                color: root.sinkMuted ? root.contentMuted : root.accent
                                font.family: root.fontFamily
                                font.pixelSize: 14
                            }

                            MouseArea {
                                id: menuMuteMouse
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    if (root.audioSink && root.audioSink.audio) {
                                        root.audioSink.audio.muted = !root.audioSink.audio.muted;
                                    }
                                }
                            }
                        }

                        // Interactive Volume Slider
                        Item {
                            Layout.fillWidth: true
                            implicitHeight: 20

                            Rectangle {
                                anchors.verticalCenter: parent.verticalCenter
                                width: parent.width
                                height: 4
                                radius: 2
                                color: root.borderMain

                                Rectangle {
                                    width: parent.width * (root.sinkMuted ? 0 : Math.min(1, root.sinkVolume))
                                    height: parent.height
                                    radius: 2
                                    color: root.sinkMuted ? root.contentMuted : root.accent
                                }
                            }

                            Rectangle {
                                x: Math.max(0, Math.min(parent.width - width, parent.width * (root.sinkMuted ? 0 : Math.min(1, root.sinkVolume)) - width / 2))
                                anchors.verticalCenter: parent.verticalCenter
                                width: 10
                                height: 10
                                radius: 5
                                color: root.accent
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                function setVol(mx) {
                                    if (!root.audioSink || !root.audioSink.audio) return;
                                    const v = Math.max(0, Math.min(1, mx / width));
                                    root.audioSink.audio.volume = v;
                                    if (root.audioSink.audio.muted && v > 0) {
                                        root.audioSink.audio.muted = false;
                                    }
                                }
                                onPressed: mouse => setVol(mouse.x)
                                onPositionChanged: mouse => { if (pressed) setVol(mouse.x); }
                            }
                        }

                        Text {
                            text: root.sinkMuted ? "mute" : Math.round(root.sinkVolume * 100) + "%"
                            color: root.contentMain
                            font.family: root.fontFamily
                            font.pixelSize: 12
                            Layout.minimumWidth: 34
                            horizontalAlignment: Text.AlignRight
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        height: 1
                        color: root.borderMain
                    }

                    Text {
                        text: "Output Devices"
                        color: root.contentMuted
                        font.family: root.fontFamily
                        font.pixelSize: 11
                    }

                    Flickable {
                        Layout.fillWidth: true
                        implicitHeight: Math.min(220, Math.max(36, audioListCol.implicitHeight))
                        contentHeight: audioListCol.implicitHeight
                        clip: true

                        ColumnLayout {
                            id: audioListCol
                            width: parent.width
                            spacing: 4

                            Text {
                                visible: root.outputSinks.length === 0
                                text: "No output devices found"
                                color: root.contentMuted
                                font.family: root.fontFamily
                                font.pixelSize: 12
                                Layout.alignment: Qt.AlignHCenter
                                Layout.topMargin: 8
                            }

                            Repeater {
                                model: root.outputSinks
                                delegate: Rectangle {
                                    required property var modelData
                                    readonly property bool isCurrent: root.audioSink && modelData.id === root.audioSink.id

                                    Layout.fillWidth: true
                                    implicitHeight: 30
                                    radius: 7
                                    color: isCurrent ? root.bgWsActive : (audioDevMouse.containsMouse ? root.bgHover : "transparent")

                                    RowLayout {
                                        anchors.fill: parent
                                        anchors.leftMargin: 10
                                        anchors.rightMargin: 10
                                        spacing: 8

                                        Text {
                                            text: isCurrent ? "󰄬" : "·"
                                            color: isCurrent ? root.accent : root.contentMuted
                                            font.family: root.fontFamily
                                            font.pixelSize: 13
                                        }

                                        Text {
                                            Layout.fillWidth: true
                                            text: modelData.description || modelData.nickname || modelData.name
                                            color: isCurrent ? root.accent : root.contentMain
                                            font.family: root.fontFamily
                                            font.pixelSize: 12
                                            elide: Text.ElideRight
                                        }
                                    }

                                    MouseArea {
                                        id: audioDevMouse
                                        anchors.fill: parent
                                        hoverEnabled: true
                                        cursorShape: Qt.PointingHandCursor
                                        onClicked: Pipewire.preferredDefaultAudioSink = modelData
                                    }
                                }
                            }
                        }
                    }
                }

                // === NOTIFICATIONS CONTENT ===
                ColumnLayout {
                    visible: root.activeMenu === "notifications"
                    Layout.fillWidth: true
                    spacing: 8

                    ColumnLayout {
                        visible: notifManager.historyCount === 0
                        Layout.fillWidth: true
                        Layout.topMargin: 14
                        Layout.bottomMargin: 14
                        spacing: 6

                        Text {
                            Layout.alignment: Qt.AlignHCenter
                            text: notifManager.dnd ? "󰂛" : "󰂚"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 22
                        }

                        Text {
                            Layout.alignment: Qt.AlignHCenter
                            text: notifManager.dnd ? "Do Not Disturb is on" : "No notifications"
                            color: root.contentMuted
                            font.family: root.fontFamily
                            font.pixelSize: 12
                        }
                    }

                    Flickable {
                        visible: notifManager.historyCount > 0
                        Layout.fillWidth: true
                        implicitHeight: Math.min(380, notifHistoryCol.implicitHeight)
                        contentHeight: notifHistoryCol.implicitHeight
                        clip: true

                        ColumnLayout {
                            id: notifHistoryCol
                            width: parent.width
                            spacing: 6

                            Repeater {
                                model: notifManager.historyModel
                                delegate: NotificationCard {
                                    required property var modelData
                                    Layout.fillWidth: true
                                    entry: modelData
                                    theme: root
                                    compact: true
                                    nowMs: notifManager.nowMs

                                    onDismissNotification: notifManager.dismissById(modelData.notifId)
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Subtle Volume OSD Pill
    PanelWindow {
        visible: root.osdVisible
        anchors {
            bottom: true
        }
        margins {
            bottom: 32
        }
        implicitWidth: 168
        implicitHeight: 30
        color: "transparent"
        exclusiveZone: 0

        Rectangle {
            anchors.fill: parent
            radius: 10
            color: root.bgCard
            border.width: 1
            border.color: root.borderMain

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: 10
                anchors.rightMargin: 10
                spacing: 8

                Text {
                    text: root.sinkMuted ? "󰖁" : "󰕾"
                    color: root.sinkMuted ? root.contentMuted : root.accent
                    font.family: root.fontFamily
                    font.pixelSize: 13
                }

                Rectangle {
                    Layout.fillWidth: true
                    height: 3
                    radius: 2
                    color: root.borderMain

                    Rectangle {
                        width: parent.width * (root.sinkMuted ? 0 : Math.min(1, root.sinkVolume))
                        height: parent.height
                        radius: 2
                        color: root.accent
                    }
                }

                Text {
                    text: root.sinkMuted ? "mute" : Math.round(root.sinkVolume * 100) + "%"
                    color: root.contentMain
                    font.family: root.fontFamily
                    font.pixelSize: 11
                }
            }
        }
    }
}
