# octopus

[![GitHub repo](https://img.shields.io/badge/GitHub-pxotitas%2Foctopus-blue?logo=github)](https://github.com/pxotitas/octopus)
[![License](https://img.shields.io/github/license/pxotitas/octopus)](https://github.com/pxotitas/octopus/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)](https://github.com/pxotitas/octopus)
[![GitHub stars](https://img.shields.io/github/stars/pxotitas/octopus?style=social)](https://github.com/pxotitas/octopus/stargazers)

a cozy m3 shell for niri: a minimal vertical bar on waybar, with everything
bigger (control center, dash, menus, notifications) in a companion written
in rust + gtk4.

opaque, tonal surfaces, no blur (only the lock screen blurs). portuguese or
english. the colors come from the wallpaper through matugen.

> this is the branch for the docs in english. the code lives on
> [`main`](https://github.com/pxotitas/octopus/tree/main); in portuguese, on
> [`pt-br`](https://github.com/pxotitas/octopus/tree/pt-br).

## pieces

| part | what |
|---|---|
| `waybar/` | the vertical bar: logo, workspaces, clock, tray (with the animated arrow from `tray-arrow.c`), a battery pill with its percentage and the group with a drawer |
| `companion/` | `octopus`, the daemon: control center, dash (with a wallpaper picker), spotlight, menus (wi-fi, bluetooth, sound, tray, clipboard, emoji, power), dock, settings, notifications, osd and the lock screen |
| `scripts/` | theme, battery, weather, lyrics, emoji, wofi and matuwall |
| `matugen/` | color templates for the companion, waybar, fuzzel, wofi and matuwall |
| `niri.kdl` | autostart, shortcuts and layer rules, included from niri's config |

more in [`docs/`](docs/README.md).

## install

needs: `pam waybar matugen matuwall awww cliphist wl-clipboard wtype
playerctl cava brightnessctl networkmanager bluez-utils libpulse python
unicode-emoji unicode-cldr fd gcc wofi gtk3 gtk4 gtk4-layer-shell
ttf-material-symbols-variable rust`.

the repo lives in `~/.config/octopus` itself:

```sh
git clone https://github.com/pxotitas/octopus ~/.config/octopus
~/.config/octopus/install.sh
```

the build goes to `~/.cache/octopus/target`.

then, at the end of `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

## commands

```
octopus                      # the daemon
octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|emoji|power|launcher>
octopus open wallpapers      # the dash on its wallpapers tab
octopus settings
octopus close
octopus lock
octopus volume <up|down|mute>
octopus mic mute
octopus brightness <up|down>
octopus kbd <up|down>
octopus dnd
octopus reload               # writes waybar's config again
octopus restart
octopus luma <image>         # dark or light, used by the automatic mode
```

## settings

`octopus.toml` (or `octopus settings`): language, bar, dock, weather,
notifications and lyrics. to override styles without touching the built-in
css, make a `~/.config/octopus/style.css`; it reloads on its own.

## license

gpl-3.0-or-later. some parts come from [impasto](https://github.com/andreumassanet/impasto)
and [velatura](https://github.com/pxdritz1/velatura); see
[`NOTICE.md`](https://github.com/pxotitas/octopus/blob/main/NOTICE.md) on main.
