#!/usr/bin/env bash
# Builds the companion and links Octopus into ~/.config. Anything it would
# replace is moved aside with a timestamp first.
set -euo pipefail

repo=$(cd "$(dirname "$0")" && pwd)
config=${XDG_CONFIG_HOME:-$HOME/.config}
bin=$HOME/.local/bin
stamp=$(date +%Y%m%d-%H%M%S)

link() {
  local target=$1 path=$2
  if [ -L "$path" ] && [ "$(readlink -f "$path")" = "$(readlink -f "$target")" ]; then
    return
  fi
  if [ -e "$path" ] || [ -L "$path" ]; then
    mv "$path" "$path.bak-$stamp"
    echo "  moved aside: $path.bak-$stamp"
  fi
  ln -s "$target" "$path"
  echo "  linked: $path"
}

echo ":: building the companion"
cargo build --release --manifest-path "$repo/companion/Cargo.toml"
mkdir -p "$bin"
install -m 755 "$repo/companion/target/release/octopus" "$bin/octopus"

echo ":: linking configs"
mkdir -p "$config"
link "$repo/config" "$config/octopus"
link "$repo/waybar" "$config/waybar"
link "$repo/config/scripts/theme.sh" "$bin/octopus-theme"

if [ ! -f "$repo/config/colors.css" ]; then
  wallpaper=$(awww query 2>/dev/null | sed -n 's/.*image: //p' | head -n1 || true)
  if [ -n "$wallpaper" ] && [ -f "$wallpaper" ]; then
    echo ":: first palette from $wallpaper"
    "$bin/octopus-theme" "$wallpaper"
  else
    echo ":: no wallpaper found; pick one with Mod+W to generate the palette"
  fi
fi

cat <<EOF

Done. Add this line to the end of ~/.config/niri/config.kdl:

  include "~/.config/octopus/niri.kdl"

and remove the old shell's spawn-at-startup lines and any binds it repeats.
EOF
