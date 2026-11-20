#!/bin/bash
BAR_PATH="/home/arhammahajan/dev/dots/quickshell/bar/shell.qml"

if ! quickshell ipc -p "$BAR_PATH" call bar toggle >/dev/null 2>&1; then
  QS_ICON_THEME=Papirus-Dark nohup quickshell -p "$BAR_PATH" </dev/null >/dev/null 2>&1 &
fi
