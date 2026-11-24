#!/usr/bin/env bash
# App launcher: wofi in the shell's colors; a second press closes it.
# The list comes from the daemon, ranked by use like the spotlight's.
# It runs with a config home of its own that keeps the GTK settings and
# dconf (icons, fonts) but not ~/.config/gtk-3.0/gtk.css, whose rules
# would win over wofi's style.
pkill -x wofi && exit 0
dir="$HOME/.config/octopus/wofi"
home="${XDG_CACHE_HOME:-$HOME/.cache}/octopus/wofi-home"
mkdir -p "$home/gtk-3.0"
[ -f "$HOME/.config/gtk-3.0/settings.ini" ] && ln -sf "$HOME/.config/gtk-3.0/settings.ini" "$home/gtk-3.0/settings.ini"
[ -d "$HOME/.config/dconf" ] && ln -sfn "$HOME/.config/dconf" "$home/dconf"
export XDG_CONFIG_HOME="$home"

if ! list=$(octopus wofi-list 2>/dev/null) || [ -z "$list" ]; then
  exec wofi --show drun --conf "$dir/config" --style "$dir/style.css"
fi
selected=$(printf '%s\n' "$list" | sed '/^$/d' | wofi --dmenu --conf "$dir/config" --style "$dir/style.css") || exit 0
[ -n "$selected" ] || exit 0

# dmenu-print_line_num=true makes Wofi return the zero-based input line number.
# Octopus expects that same zero-based index.
case "$selected" in
  ''|*[!0-9]*) exit 0 ;;
esac
octopus wofi-launch "$selected"
