#!/usr/bin/env bash
# Emoji by name or keyword, in Portuguese and English; Enter types it,
# and it stays on the clipboard too.
pkill -x fuzzel && exit 0

cache="${XDG_CACHE_HOME:-$HOME/.cache}/octopus/emoji.tsv"
source=/usr/share/unicode/emoji/emoji-test.txt
here=$(dirname "$(readlink -f "$0")")

if [ ! -s "$cache" ] || [ "$source" -nt "$cache" ] || [ "$here/emoji.py" -nt "$cache" ]; then
  mkdir -p "$(dirname "$cache")"
  python3 "$here/emoji.py" "${LANG%%.*}" --tsv >"$cache.tmp" && mv "$cache.tmp" "$cache"
fi

glyph=$(fuzzel --dmenu --prompt "" --placeholder "emoji" --lines 10 --width 44 \
  --with-nth "{1}   {2}" --match-nth "{2} {3}" --accept-nth 1 <"$cache")
[ -z "$glyph" ] && exit 0

printf '%s' "$glyph" | wl-copy
sleep 0.15
wtype "$glyph" 2>/dev/null
