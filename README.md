# octopus 🐙

*A little home for a desktop that feels like yours.*

Octopus is a personal, modular desktop shell for **Niri on Wayland**. It is being built in small, understandable pieces: deliberate visuals, predictable interactions, and no unnecessary background work.

The goal is not to install somebody else's desktop and repaint it. The goal is to make a shell with its own character — calm, tactile, contemporary, and thoughtfully put together.

> Cozy by nature. Opinionated by design. Open to the codeocracy.

## The feeling

- **Contemporary minimalism:** clear hierarchy, generous but purposeful spacing, and quiet details.
- **Opaque surfaces:** no accidental transparency, glassmorphism, or blurry layers.
- **Soft geometry:** restrained corner radii and consistent shapes.
- **Adaptive colour:** Matugen provides the palette; components use semantic colour roles rather than hard-coded accents.
- **Readable typography:** SF Pro Display for interface text where available, with JetBrainsMono Nerd Font for icon glyphs and technical details.
- **Motion with intent:** transitions should feel fluid and useful, never noisy or sluggish.

## Planned pieces

| Piece | Role |
| --- | --- |
| Waybar | The top-level status and interaction bar |
| Ewwii | The Octopus section surface and contextual panels |
| Walker | Application launcher |
| Dunst | Notifications |
| Hyprlock / Hypridle | Lock screen and idle handling |
| Awww | Wallpaper daemon |
| Clipse | Clipboard history |
| Matugen | Dynamic colour generation |
| Niri | Window management and workspace animations |

The list is a plan, not a claim that every component is installed or configured yet.

## Project principles

1. Build in small, testable stages.
2. Keep the existing desktop usable while Octopus is being developed.
3. Do not change the login manager or replace the user's current shell/session.
4. Prefer lazy loading and short-lived helpers over idle background processes.
5. Document decisions and unresolved questions as they happen.
6. Reuse upstream ideas only after checking their behaviour, compatibility, and licence.

## Status

Octopus is in its foundation and Waybar specification stage. See [TODO](TODO.md), [design principles](docs/DESIGN.md), and the [Waybar specification](docs/WAYBAR.md).

## Dependencies

Install the packages used by the planned Octopus components on CachyOS/Arch:

```bash
sudo pacman -S --needed waybar walker dunst hyprlock hypridle awww matugen cava playerctl python wl-clipboard
```

Clipse is distributed through the AUR, so install it separately with an AUR helper:

```bash
yay -S --needed clipse
```

Notes:
- `playerctl` provides the MPRIS interface used by Waybar; it is listed explicitly even where Waybar may pull it as a dependency.
- `python` runs the Open-Meteo weather helper.
- `wl-clipboard` supports clipboard integration with Clipse.
- `cava` is used by the media section and should only run when that section needs its visualizer.
- Niri is the existing compositor/session and is not included in the install command.
- Ewwii is intentionally not included: its implementation and installation are user-managed.

These commands install packages only. They do not enable services, change the login manager, or configure startup.

## Development

The repository is intentionally local-first. No installer or system-wide changes are part of the initial scaffold.
