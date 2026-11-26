#!/usr/bin/env bash
# Waybar battery pill: the fill level travels as a CSS class in 5% steps.

battery=$(ls -d /sys/class/power_supply/BAT* 2>/dev/null | head -n1)
[ -z "$battery" ] && exit 0

# Against what the battery holds now, like waybar's percentage: a worn
# battery reports "Full" with `capacity` well under 100.
capacity=$(<"$battery/capacity")
for kind in charge energy; do
  if [ -r "$battery/${kind}_now" ] && [ -r "$battery/${kind}_full" ]; then
    now=$(<"$battery/${kind}_now") full=$(<"$battery/${kind}_full")
    [ "$full" -gt 0 ] && capacity=$(( (now * 100 + full / 2) / full ))
    break
  fi
done
[ "$capacity" -gt 100 ] && capacity=100
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
