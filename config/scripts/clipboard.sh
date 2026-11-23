#!/usr/bin/env bash
# History from cliphist; the id column stays hidden.
pkill -x fuzzel && exit 0
cliphist list |
  fuzzel --dmenu --prompt "" --placeholder "histórico" --with-nth 2 --lines 12 --width 56 |
  cliphist decode |
  wl-copy
