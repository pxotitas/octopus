# Waybar specification

## Placement and composition

- Position: **top**.
- Density: **balanced**.
- Shape language: **soft corners**.
- Surface: fully opaque.
- Colour: Matugen semantic palette.
- Typeface: SF Pro Display for UI text; JetBrainsMono Nerd Font for icon glyphs.

The bar has three groups.

### Left

1. Niri workspaces.
2. Focused window title, accompanied by an application icon.

Workspace display: show **all currently existing workspaces**, with the focused workspace visually emphasized. Do not show only the focused workspace, and do not create persistent placeholder workspaces. Switching workspaces should have a restrained animation.

The native `niri/window` module provides the focused window title, app ID, and an optional application icon. Use its `icon` option; no custom icon resolver is needed unless testing reveals a concrete gap.

### Centre

Three independent clickable modules:

1. **Clock** — opens the clock section, containing a polished time and calendar view.
2. **Weather** — opens the weather section. The compact bar representation uses a condition-sensitive icon and temperature. Location mode: automatic.
3. **Media** — opens the media section. Use MPRIS so compatible players can be supported, rather than hard-coding Spotify.

The modules are independent. There is no shared expandable centre button.

#### Pointer behaviour

- Left click on Clock → show Clock section.
- Left click on Weather → show Weather section.
- Left click on Media → show Media section.
- Right click on any of the three → show the Main section.

There is exactly one shared Octopus section window. If it is already open, selecting another module replaces the content in that same window; do not spawn a second window.

#### Media section and CAVA lifecycle

- The media section provides playback information and common controls.
- Track title may use a restrained marquee/scroll when it does not fit.
- Hide the media module completely when no player is actively playing, subject to final MPRIS state handling.
- Start CAVA lazily when the media section needs it.
- When the media section is left or its window is closed, stop the CAVA process. It must not remain running in the background.

### Right

In this order:
1. System tray.
2. Network.
3. Bluetooth.
4. Audio output.
5. Microphone.
6. Battery.
7. Power icon, opening the Octopus power section/menu.

Audio and microphone modules use an icon plus a readable value/state (for example, an icon and percentage). Icons must adapt to mute and level states rather than remain static.

## Native Niri modules

Use Waybar’s built-in modules directly:

- `niri/workspaces`: keep `hide-empty` disabled (the default) so existing empty workspaces remain visible; do not enable `current-only`. The focused workspace receives the `.focused` CSS class, while other states have their own classes. Use `format-icons` only if the chosen compact representation needs it.
- `niri/window`: use the native `icon` option and format the title directly.

No custom workspace polling script or window-title script is planned. Keep the JSONC configuration and CSS straightforward; add a script only if a tested native-module limitation requires one.

## Weather

- Location mode: automatic.
- Compact format: condition icon + temperature.
- Icon changes with the reported weather condition.
- The location provider and weather data source are not selected yet. Choose a privacy-conscious source and document its network behaviour before implementation.
- Handle unavailable network/data gracefully without leaving a broken placeholder in the bar.

## Notifications

Dunst is the selected notification daemon.

Preferred notification presentation:
- use the sender's avatar when the notification payload actually provides one;
- otherwise use the application's icon;
- never fabricate a sender avatar.

Discord sender-avatar support is not assumed. It needs a real payload/integration test before being promised.

## Implementation constraints

- Use Waybar's native Niri modules where they meet the requirements.
- Add scripts only for gaps that native modules cannot cover cleanly.
- Keep scripts small, readable, and independently testable.
- Do not start or enable Waybar automatically during the initial development stage.
- Do not change the current login manager, display manager, shell, or Niri startup configuration as part of the first implementation.
