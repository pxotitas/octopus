#!/usr/bin/env bash
pkill -x fuzzel && exit 0

entries=(
  "bloquear\0icon\x1fsystem-lock-screen"
  "suspender\0icon\x1fsystem-suspend"
  "sair\0icon\x1fsystem-log-out"
  "reiniciar\0icon\x1fsystem-reboot"
  "desligar\0icon\x1fsystem-shutdown"
)

choice=$(printf '%b\n' "${entries[@]}" |
  fuzzel --dmenu --prompt "" --placeholder "energia" --lines 5 --width 24 --hide-prompt)

case $choice in
  bloquear) octopus lock ;;
  suspender) systemctl suspend ;;
  sair) niri msg action quit --skip-confirmation ;;
  reiniciar) systemctl reboot ;;
  desligar) systemctl poweroff ;;
esac
