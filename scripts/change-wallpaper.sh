#!/bin/bash
wallpaper_dir="$HOME/Pictures/wallpapers"
export GUM_CHOOSE_HEADER_FOREGROUND="#d8dadd"
export GUM_CHOOSE_SELECTED_FOREGROUND="#758A9B"
export GUM_CHOOSE_CURSOR_FOREGROUND="#758A9B"

# deps check
_is_installed() {
  command -v "$1" &>/dev/null
}

deps=(fd magick awww awww-daemon)
missing=()

for dep in "${deps[@]}"; do
  _is_installed "$dep" || missing+=("$dep")
done

if [[ -n ${missing[*]} ]]; then
  echo "[ERROR] missing dependencies: ${missing[*]}"
  exit 1
fi

# main logic
if [ ! -d "$wallpaper_dir" ]; then
  mkdir -p "$wallpaper_dir"
fi
images=$(fd . --base-directory "$wallpaper_dir" | grep -iE '\.(jpe?g|png|webp|gif)$' | sort)
if [ -z "$images" ]; then
  echo "[ERROR] No image file found"
  echo "[INFO] Place your wallpapers in $wallpaper_dir"
  read -n 1 -s -r -p "[INFO] Press any key to finish..."
  exit 1
fi
image="$wallpaper_dir/$(echo "$images" | gum choose --header 'Choose your wallpaper: ')"
if [ $? -eq 1 ]; then
  exit 1
fi
mode=$(echo -e "crop\nfit\nstretch\nno" | gum choose --header "Choose wallpaper resize mode: ")
if [ $? -eq 1 ]; then
  exit 1
fi

ext="${image##*.}"
workspace_wall="/home/arhammahajan/dev/dots/wallpapers/workspace.$ext"
backdrop_wall="/home/arhammahajan/dev/dots/wallpapers/backdrop.png"

echo "[INFO] New wallpaper: $image"
echo "[INFO] Copying new wallpaper to /home/arhammahajan/dev/dots..."
rm -f /home/arhammahajan/dev/dots/wallpapers/workspace.*
cp -f "$image" "$workspace_wall"
canvas_color=$(magick "${workspace_wall}[0]" -crop x1+0+0 -resize 1x1 txt:- | grep -o '#[0-9A-Fa-f]\{6\}' | head -n 1)
canvas_color="${canvas_color#\#}"
canvas_color="${canvas_color:-010102}"

if ! pgrep -x awww-daemon >/dev/null; then
  nohup awww-daemon >/dev/null 2>&1 &
  sleep 0.5
fi
awww img "$workspace_wall" --resize "$mode" --fill-color "$canvas_color" --transition-type any

echo "[INFO] Creating new overview backdrop..."
magick "${workspace_wall}[0]" -scale 10% -blur 0x2.5 -resize 1000% "$backdrop_wall"
if ! pgrep -f "awww-daemon --namespace backdrop" >/dev/null; then
  nohup awww-daemon --namespace backdrop >/dev/null 2>&1 &
  sleep 0.5
fi
awww img --namespace backdrop "$backdrop_wall" --transition-type any

sed -i "s|awww img /home/arhammahajan/dev/dots/wallpapers/workspace\.[^ ]* --resize [^ ]* --fill-color [^ \"]*|awww img $workspace_wall --resize $mode --fill-color $canvas_color|" "/home/arhammahajan/dev/dots/niri/wallpapers.kdl"
sed -i "s|awww img --namespace backdrop /home/arhammahajan/dev/dots/wallpapers/backdrop\.[^ \"]*|awww img --namespace backdrop $backdrop_wall|" "/home/arhammahajan/dev/dots/niri/wallpapers.kdl"

echo "[INFO] Done!"
read -n 1 -s -r -p "[INFO] Press any key to finish..."
