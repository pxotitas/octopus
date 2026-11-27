#!/usr/bin/env bash
# Octopus lives in ~/.config/octopus: clone it there and run this. It builds
# the companion and links Waybar and the theme hook. Anything it would
# replace is moved aside with a timestamp first.
set -euo pipefail

# ./install.sh [--yes] [--no-files] [--no-default]
#   --yes         answer yes to every question
#   --no-files    skip the file manager
#   --no-default  install the file manager without making it the default
yes=0 files=ask default=ask
for arg in "$@"; do
  case $arg in
    --yes|-y) yes=1 ;;
    --no-files) files=no ;;
    --no-default) default=no ;;
  esac
done

ask() {
  local question=$1 answer
  if [ "$yes" = 1 ] || [ ! -t 0 ]; then
    return 0
  fi
  read -r -p "$question [Y/n] " answer
  case $answer in
    [nN]*) return 1 ;;
    *) return 0 ;;
  esac
}

repo=$(cd "$(dirname "$0")" && pwd)
config=${XDG_CONFIG_HOME:-$HOME/.config}
bin=$HOME/.local/bin
stamp=$(date +%Y%m%d-%H%M%S)
export CARGO_TARGET_DIR=${XDG_CACHE_HOME:-$HOME/.cache}/octopus/target

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

if [ "$(readlink -f "$repo")" != "$(readlink -f "$config/octopus")" ]; then
  echo ":: octopus is not in $config/octopus, linking it there"
  link "$repo" "$config/octopus"
fi

echo ":: building the companion"
cargo build --release --manifest-path "$repo/companion/Cargo.toml"
mkdir -p "$bin"
install -m 755 "$CARGO_TARGET_DIR/release/octopus" "$bin/octopus"

echo ":: building the tray button"
cache=${XDG_CACHE_HOME:-$HOME/.cache}/octopus
mkdir -p "$cache"
gcc -shared -fPIC -O2 -o "$cache/tray-arrow.so" "$repo/waybar/tray-arrow.c" \
  $(pkg-config --cflags --libs gtk+-3.0) -lm || echo "  skipped: needs gcc and gtk3"

echo ":: linking"
link "$repo/waybar" "$config/waybar"
link "$repo/scripts/theme.sh" "$bin/octopus-theme"

apps=${XDG_DATA_HOME:-$HOME/.local/share}/applications
services=${XDG_DATA_HOME:-$HOME/.local/share}/dbus-1/services
mkdir -p "$apps"

# the reminders app, run by the shell
install -m 644 "$repo/assets/octopus-reminders.desktop" "$apps/octopus-reminders.desktop"
echo "  installed: reminders"

if [ "$files" != no ] && ask ":: install octopus files, the file manager?"; then
  install -m 644 "$repo/assets/octopus-files.desktop" "$apps/octopus-files.desktop"
  echo "  installed: octopus files (octopus files [path])"
  if [ "$default" != no ] && ask ":: make it the default file manager (folders and \"show in folder\" open in it)?"; then
    xdg-mime default octopus-files.desktop inode/directory
    mkdir -p "$services"
    cat > "$services/org.freedesktop.FileManager1.service" <<SERVICE
[D-BUS Service]
Name=org.freedesktop.FileManager1
Exec=$bin/octopus files --service
SERVICE
    echo "  default file manager: octopus files"
  fi
  update-desktop-database "$apps" 2>/dev/null || true
  missing=()
  for tool in fd ffmpegthumbnailer avahi-browse unzip zip git; do
    command -v "$tool" >/dev/null || missing+=("$tool")
  done
  [ ${#missing[@]} -gt 0 ] && echo "  optional, not found: ${missing[*]}"
  command -v rclone >/dev/null || echo "  google drive in files needs rclone (sudo pacman -S rclone)"
  command -v gh >/dev/null || echo "  cloning your github repositories needs gh (sudo pacman -S github-cli)"
fi

if [ ! -f "$repo/colors.css" ]; then
  wallpaper=$(awww query 2>/dev/null | sed -n 's/.*image: //p' | head -n1 || true)
  if [ -n "$wallpaper" ] && [ -f "$wallpaper" ]; then
    echo ":: first palette from $wallpaper"
    "$bin/octopus-theme" "$wallpaper"
  else
    echo ":: no wallpaper found; pick one with Mod+W to generate the palette"
  fi
fi

cat <<MSG

Done. Add this line to the end of ~/.config/niri/config.kdl:

  include "~/.config/octopus/niri.kdl"

and remove the old shell's spawn-at-startup lines and any binds it repeats.
MSG
