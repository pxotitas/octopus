# Architecture notes

## Current target

- OS family: Arch-based CachyOS
- Session: Niri on Wayland
- Primary language/toolkit choices: not locked yet
- Repository strategy: local-first

## Planned responsibility boundaries

| Component | Responsibility |
| --- | --- |
| Niri | Window management, workspaces, compositor-level animations |
| Waybar | Persistent top bar and lightweight status modules |
| Ewwii | Shared Octopus section window and contextual panels |
| Small scripts/services | Bridge data or behaviour only where native modules are insufficient |
| Matugen | Generate semantic palette values |
| Dunst | Notification delivery |
| Walker | Application launching |
| Hyprlock / Hypridle | Locking and idle lifecycle |
| Awww | Wallpaper management |
| Clipse | Clipboard history |

These are intended boundaries. The exact IPC and process model must be verified against the installed versions before implementation.

## Ewwii research note

The official Ewwii project is [Ewwii-sh/ewwii](https://github.com/Ewwii-sh/ewwii). Its current published documentation identifies version 0.10.0 and describes a Rust + GTK4 widget system configured with NBCL and styled with CSS/SCSS. Its Wayland window model exposes layer-shell-oriented properties such as stacking, focusability, namespace, and screen-edge geometry anchors. This looks like a promising fit for Octopus, but the shared-window content-switching mechanism and exact anchor behaviour must be prototyped before we lock the architecture.

References: [Ewwii documentation](https://ewwii-sh.github.io/docs/), [window properties](https://ewwii-sh.github.io/docs/next/widgets_and_window/window-props/), [widget-control commands](https://ewwii-sh.github.io/docs/0.9.0/commands/widget-control/).

## Shared section window

The centre Waybar modules send a section request to one Octopus UI controller:
- clock
- weather
- media
- main

The controller should update the existing window's content if it is already open. It should not create one window per section.

The final transport (Ewwii IPC, a local socket, or another supported mechanism) is deliberately undecided until the actual Ewwii interface and installed version are inspected.

## Process lifecycle

CAVA is an on-demand child process of the media-section lifecycle:
- start only when the media section is opened and visualisation is needed;
- stop when media section closes, another section replaces it, or the shared window is dismissed;
- handle repeated open/close requests without orphaned processes.

## Safety and rollout

- Develop and preview components without replacing the active desktop.
- Keep user configuration changes separate from repository source.
- Back up any existing config before eventual integration.
- Do not alter the login manager or enable a new session.
- Do not install packages until the dependency list and package sources have been reviewed.
