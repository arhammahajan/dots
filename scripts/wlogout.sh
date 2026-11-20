#!/bin/bash
if pgrep -f "quickshell.*powermenu/shell.qml" >/dev/null 2>&1; then
  pkill -f "quickshell.*powermenu/shell.qml"
else
  nohup quickshell -p /home/arhammahajan/dev/dots/quickshell/powermenu/shell.qml >/dev/null 2>&1 &
fi
