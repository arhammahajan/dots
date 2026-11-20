#!/bin/bash
figlet "Updates" -f slant -w 44 -c
echo ""
sleep 0.1

pkexec dnf upgrade --refresh

echo ""
echo "[INFO] OK"
read -n 1 -s -r -p "[INFO] Press any key to finish..."
