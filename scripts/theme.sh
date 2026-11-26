#!/usr/bin/env bash
# Runs after Matuwall applies a wallpaper: Octopus' own templates, then the
# rest of the system's matugen templates, from the most dominant color.
image=$1
[ -f "$image" ] || exit 1
state="${XDG_STATE_HOME:-$HOME/.local/state}/octopus"
mode=$(cat "$state/mode" 2>/dev/null || echo dark)
scheme=$(cat "$state/scheme" 2>/dev/null || echo tonal-spot)
contrast=$(cat "$state/contrast" 2>/dev/null || echo 0)
look=(-t "scheme-$scheme" --contrast "$contrast")
if [ "$mode" = auto ]; then
  mode=$(octopus luma "$image" 2>/dev/null || echo dark)
  gsettings set org.gnome.desktop.interface color-scheme "prefer-$mode" 2>/dev/null
fi
matugen image "$image" -m "$mode" "${look[@]}" --source-color-index 0 -c "$HOME/.config/octopus/matugen/config.toml"
[ -f "$HOME/.config/matugen/config.toml" ] && matugen image "$image" -m "$mode" "${look[@]}" --source-color-index 0
