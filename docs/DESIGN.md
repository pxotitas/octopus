# Design direction

## North star

Octopus should feel like a carefully designed desktop product, not a pile of widgets. Its visual language is **contemporary minimalism**: quiet structure, precise alignment, restrained decoration, and small moments of personality.

The reference direction is the visual restraint and composition of [Nucleus Shell](https://github.com/nucleus-hq/nucleus-shell). This is a direction for study, not a request to clone its implementation or identity.

## Non-negotiable visual rules

- Every shell surface is fully opaque.
- No glassmorphism, frosted-glass effects, or decorative translucency.
- No arbitrary bright-blue blocks or accent panels.
- Colour comes from Matugen semantic roles, not one-off hard-coded colours.
- Use contrast and spacing to establish hierarchy before adding borders or decoration.
- Rounded corners are soft and consistent; avoid both sharp industrial panels and exaggerated pill shapes.
- Shadows, if used, should be subtle and functional, not a substitute for surface contrast.
- Icons should be consistent in stroke/weight and should communicate state.

## Typography

- **Interface text:** SF Pro Display is the current preferred face because it is already available on the target machine. Keep the design portable with a sensible sans-serif fallback.
- **Icon glyphs / technical readouts:** JetBrainsMono Nerd Font.
- Avoid using a monospaced face for every label. Monospace is for data that benefits from fixed-width alignment.

## Colour system

Matugen is the source of generated colour values. Components should consume semantic tokens rather than independently sampling wallpaper colours.

Initial semantic roles to define:
- surface — primary opaque panel background
- surface-container — secondary opaque region
- surface-raised — elevated surface
- text-primary — primary readable text
- text-secondary — supporting text
- outline — subtle separators and borders
- accent — interactive emphasis
- accent-foreground — readable content on accent
- success, warning, error — status states

The exact token mapping and light/dark behaviour will be defined when the Matugen integration is implemented. Do not guess fixed hex values in individual modules.

## Shape and spacing

- Use a small, consistent radius scale rather than unique radii per widget.
- Keep the Waybar compact but not cramped.
- Align icons and text optically, not just mathematically.
- Use spacing to group related controls and separate unrelated groups.
- Avoid decorative containers around every single value.

## Motion

Motion should be subtle, quick, and consistent:
- Workspace changes: clear but restrained transition.
- Panels: short enter/exit motion with no overshoot-heavy bounce.
- Content changes inside the shared Octopus window: transition the content, not the whole desktop.
- Respect reduced-motion preferences where the toolkit supports them.

Exact durations and easing curves remain implementation decisions and must be tested on the target machine.

## Interaction

- Left click on a Waybar centre module opens its matching Octopus section.
- Right click on any centre module opens the main Octopus section.
- There is one shared section window. Selecting another section replaces its content in that same window.
- The section window should be positioned in a way that feels connected to the module that opened it; final anchoring behaviour must be validated with the chosen Ewwii implementation.

## Quality bar

A component is not finished just because it renders. It should also:
- look coherent next to its neighbours;
- have clear hover, focus, active, disabled, and unavailable states where applicable;
- remain legible at the actual display scale;
- avoid unnecessary CPU or memory use;
- behave predictably with keyboard and pointer input.
