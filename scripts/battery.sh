#!/usr/bin/env bash
# Waybar battery pill: the fill level travels as a CSS class in 5% steps.

battery=$(ls -d /sys/class/power_supply/BAT* 2>/dev/null | head -n1)
[ -z "$battery" ] && exit 0

capacity=$(<"$battery/capacity")
status=$(<"$battery/status")
step=$(( (capacity + 2) / 5 * 5 ))
classes="\"b-$step\""

case $status in
  Charging | Full) classes="$classes, \"charging\""; state="carregando" ;;
  *) state="na bateria" ;;
esac
[ "$status" = "Full" ] && state="carregada"
if [ "$capacity" -le 15 ] && [ "$status" != "Charging" ]; then
  classes="$classes, \"low\""
fi

printf '{"text": "%s", "tooltip": "%s%% · %s", "class": [%s], "percentage": %s}\n' \
  "$capacity" "$capacity" "$state" "$classes" "$capacity"
