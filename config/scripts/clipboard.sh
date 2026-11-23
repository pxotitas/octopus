#!/usr/bin/env bash
# cliphist history in fuzzel, images with a thumbnail. Closing the menu
# leaves the clipboard as it was.
pkill -x fuzzel && exit 0

thumbs="${XDG_CACHE_HOME:-$HOME/.cache}/octopus/clipboard"
mkdir -p "$thumbs"

entries() {
  cliphist list | while IFS=$'\t' read -r id content; do
    if [[ $content =~ ^\[\[\ binary\ data\ .*(png|jpe?g|webp|gif|bmp) ]]; then
      thumb="$thumbs/$id.png"
      [ -s "$thumb" ] || printf '%s\t' "$id" | cliphist decode >"$thumb" 2>/dev/null
      size=$(sed -E 's/.* ([0-9]+x[0-9]+) .*/\1/' <<<"$content")
      printf '%s\timagem %s\0icon\x1f%s\n' "$id" "$size" "$thumb"
    else
      printf '%s\t%s\n' "$id" "$content"
    fi
  done
}

choice=$(entries | fuzzel --dmenu --prompt "" --placeholder "histórico" --with-nth 2 \
  --lines 10 --width 56 --image-size-ratio 0.9)
[ -z "$choice" ] && exit 0

cliphist decode <<<"$choice" | wl-copy
