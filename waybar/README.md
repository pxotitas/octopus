# Octopus Waybar

This is the Waybar-only part of Octopus. It uses native Waybar modules for Niri, MPRIS, network, Bluetooth, WirePlumber, battery, and the tray. The only helper is the small Open-Meteo reader for the fixed Campo Grande location.

## Ewwii hand-off contract

Waybar does not implement Ewwii. Click actions call the Ewwii CLI using one shared window named `octopus` and one global signal named `section`:

```sh
ewwii update 'section="clock"' && ewwii open octopus
```

The section values used by this Waybar config are `main`, `clock`, `weather`, `media`, `network`, `bluetooth`, `audio`, `microphone`, and `power`. Centre-module right-clicks send `main`. The Ewwii configuration should consume `section` and reuse the same `octopus` window; it is intentionally left for the user to implement.

## Weather

`scripts/weather.py` requests current conditions from Open-Meteo using fixed coordinates for Campo Grande, MS. It does not perform geolocation and needs no API key. Waybar refreshes it every 15 minutes. A network/API failure returns empty text so no broken placeholder remains in the bar.

## Theme

`style.css` imports `colors.css`, generated from `matugen.css` by the Octopus template entry added to the user's Matugen config. `colors.css` is generated locally and is not committed. A current-palette fallback is included locally so the stylesheet can be previewed before the next Matugen run.

## Install location

The user's `~/.config/waybar` points to this directory. Keep the project here so the live config and the documented source stay in sync.
