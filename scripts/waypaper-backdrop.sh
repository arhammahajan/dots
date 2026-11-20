#!/bin/bash
set -euo pipefail

image="${1:-}"
if [ -z "$image" ] || [ ! -f "$image" ]; then
  exit 1
fi

backdrop_wall="/home/arhammahajan/.cache/niri-backdrop.png"

magick "${image}[0]" -scale 10% -blur 0x2.5 -resize 1000% "$backdrop_wall"

if ! pgrep -f "awww-daemon --namespace backdrop" >/dev/null; then
  nohup awww-daemon --namespace backdrop >/dev/null 2>&1 &
  sleep 0.5
fi

awww img --namespace backdrop "$backdrop_wall" --transition-type any
