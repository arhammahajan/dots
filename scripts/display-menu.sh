#!/bin/bash
if pgrep -f "quickshell.*displaymenu/shell.qml" >/dev/null 2>&1; then
  pkill -f "quickshell.*displaymenu/shell.qml"
else
  nohup quickshell -p /home/arhammahajan/dev/dots/quickshell/displaymenu/shell.qml >/dev/null 2>&1 &
fi
