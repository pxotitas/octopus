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

The official Ewwii project is [Ewwii-sh/ewwii](https://github.com/Ewwii-sh/ewwii). Its current published documentation identifies version 0.10.0 and describes a Rust + GTK4 widget system configured with NBCL and styled with CSS/SCSS.

### Shared-window content

Ewwii has a native `Stack` widget with a `selected` child index and transition options. Its `widget-control property-update` command can update a named widget's properties at runtime; signals can also update reactive properties. The straightforward prototype is therefore one Ewwii window containing one named `Stack`, with each Octopus section as a child. Switching sections changes `Stack.selected` rather than opening another window or building a separate window manager.

This is a documented capability, not yet a tested implementation on this machine. Keep the prototype small.

### Wayland positioning

Ewwii exposes Wayland layer-shell properties including stacking, focusability, namespace, and geometry anchors. Geometry supports screen-relative anchors and x/y offsets. The documentation does not describe anchoring a window directly to an arbitrary Waybar module or receiving that module's on-screen coordinates. Therefore, the desired contextual placement (slightly below the module that opened the section) remains an implementation test; do not add a positioning daemon or elaborate IPC before a minimal experiment shows one is necessary.

References: [Ewwii documentation](https://ewwii-sh.github.io/docs/), [widget properties including Stack](https://ewwii-sh.github.io/docs/next/widgets_and_window/props/), [window properties](https://ewwii-sh.github.io/docs/next/widgets_and_window/window-props/), [widget-control commands](https://ewwii-sh.github.io/docs/0.9.0/commands/widget-control/), [Ewwii changelog](https://ewwii-sh.github.io/changelog/).

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
