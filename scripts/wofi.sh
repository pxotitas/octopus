#!/usr/bin/env bash
# App launcher: wofi in the shell's colors; a second press closes it.
# It runs with a config home of its own that keeps the GTK settings and
# dconf (icons, fonts) but not ~/.config/gtk-3.0/gtk.css, whose rules
# would win over wofi's style.
pkill -x wofi && exit 0
dir="$HOME/.config/octopus/wofi"
home="${XDG_CACHE_HOME:-$HOME/.cache}/octopus/wofi-home"
mkdir -p "$home/gtk-3.0"
[ -f "$HOME/.config/gtk-3.0/settings.ini" ] && ln -sf "$HOME/.config/gtk-3.0/settings.ini" "$home/gtk-3.0/settings.ini"
[ -d "$HOME/.config/dconf" ] && ln -sfn "$HOME/.config/dconf" "$home/dconf"
XDG_CONFIG_HOME="$home" exec wofi --conf "$dir/config" --style "$dir/style.css"
