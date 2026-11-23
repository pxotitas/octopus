#!/usr/bin/env bash
# Runs after Matuwall applies a wallpaper: Octopus' own templates, then the
# rest of the system's matugen templates, from the most dominant color.
image=$1
[ -f "$image" ] || exit 1
matugen image "$image" -m dark --source-color-index 0 -c "$HOME/.config/octopus/matugen/config.toml"
[ -f "$HOME/.config/matugen/config.toml" ] && matugen image "$image" -m dark --source-color-index 0
