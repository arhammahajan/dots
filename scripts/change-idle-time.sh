#!/bin/bash
modes="5 minutes\n10 minutes\n20 minutes\n30 minutes\ninfinity"
choice=$(echo -e "$modes" | fuzzel --dmenu --lines 5 -w 20 --config /home/arhammahajan/dev/dots/fuzzel/idle-time.ini)
if [ ! -z "$choice" ]; then
  pkill swayidle
  mkdir -p "$HOME/.local/state"
  echo $choice >$HOME/.local/state/idle-time
  bash /home/arhammahajan/dev/dots/scripts/swayidle.sh &
  disown
fi
