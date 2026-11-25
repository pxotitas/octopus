#!/usr/bin/env bash
# App launcher: rofi in the shell's colors; a second press closes it.
# The list comes from the daemon, ranked by use like the spotlight's. It goes
# through a file because its icon rows carry NUL bytes.
pkill -x rofi && exit 0
conf="$HOME/.config/octopus/rofi/config.rasi"
list="${XDG_RUNTIME_DIR:-/tmp}/octopus-rofi"

if ! octopus rofi-list > "$list" 2>/dev/null || [ ! -s "$list" ]; then
  exec rofi -show drun -config "$conf"
fi
selected=$(rofi -dmenu -i -no-custom -format i -p "" -config "$conf" < "$list") || exit 0

case "$selected" in
  ''|*[!0-9]*) exit 0 ;;
esac
octopus rofi-launch "$selected"
