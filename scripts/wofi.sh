#!/usr/bin/env bash
# App launcher: wofi in the shell's colors; a second press closes it.
pkill -x wofi && exit 0
dir="$HOME/.config/octopus/wofi"
exec wofi --conf "$dir/config" --style "$dir/style.css"
