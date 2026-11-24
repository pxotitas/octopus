# Octopus

[![GitHub repo](https://img.shields.io/badge/GitHub-pxotitas%2Foctopus-blue?logo=github)](https://github.com/pxotitas/octopus)
[![License](https://img.shields.io/github/license/pxotitas/octopus)](https://github.com/pxotitas/octopus/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)](https://github.com/pxotitas/octopus)
[![GitHub stars](https://img.shields.io/github/stars/pxotitas/octopus?style=social)](https://github.com/pxotitas/octopus/stargazers)

A cozy, minimalist desktop shell for [Niri](https://github.com/YaLTeR/niri), built around a vertical Waybar and a Rust + GTK4 companion app.

Octopus combines a control center, dashboard, launcher, menus, notifications, and lock screen into a consistent interface. Its surfaces are opaque and tinted, with blur reserved for the lock screen. The theme is generated from your wallpaper through Matugen, and the interface is available in Portuguese and English.

## What's included

| Component | Description |
|---|---|
| `waybar/` | Vertical bar with workspaces, launcher, clock, tray, battery, and expandable groups. Includes `tray-arrow.c`, an animated tray button. |
| `companion/` | Rust/GTK4 daemon that provides the control center, dashboard, Spotlight-style launcher, menus, notifications, clipboard, emoji picker, power menu, and lock screen. |
| `scripts/` | Helpers for wallpaper colors, battery, themes, emoji, weather, and lyrics. |
| `matugen/` | Matugen templates for the companion, Waybar, Fuzzel, and Matuwall. |
| `niri.kdl` | Niri autostart entries, key bindings, and layer rules, ready to include in your configuration. |

More details are available in [`docs/README.md`](docs/README.md).

## Installation

Octopus expects the following tools and libraries:

```text
pam waybar matugen matuwall awww cliphist wl-clipboard wtype playerctl cava
brightnessctl networkmanager bluez-utils libpulse python unicode-emoji unicode-cldr
fd gcc gtk4 gtk4-layer-shell rustup jq curl
```

The repository should live in `~/.config/octopus`:

```sh
git clone https://github.com/pxotitas/octopus.git ~/.config/octopus
~/.config/octopus/install.sh
```

Build output is written to `~/.cache/octopus/target`.

Then add the following line to the end of `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

Restart Niri or log out and back in to apply the configuration.

## Commands

```text
octopus                      # Start the daemon
octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|emoji|power|launcher>
octopus settings
octopus close
octopus lock
octopus volume <up|down|mute>
octopus mic mute
octopus brightness <up|down>
octopus kbd <up|down>
octopus dnd
octopus luma <image>         # Choose dark or light mode for automatic theming
```

## Configuration

Edit `octopus.toml`, or use:

```sh
octopus settings
```

You can configure the language, bar, weather location, brightness devices, notifications, and lyrics.

To override the built-in styles without modifying the embedded CSS, create a `~/.config/octopus/user.css` file.

## Documentation

See the [`docs/`](docs/README.md) directory for configuration details, key bindings, theming, and troubleshooting.
