#!/bin/bash
if pgrep -f "quickshell.*lockscreen/shell.qml" >/dev/null 2>&1; then
  exit 0
fi

niri msg action do-screen-transition --delay-ms 300
nohup quickshell -p /home/arhammahajan/dev/dots/quickshell/lockscreen/shell.qml >/dev/null 2>&1 &
sleep 0.3
