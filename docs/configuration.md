# configuration

almost everything can be changed from the window (Mod+Comma). it writes to
`~/.config/octopus/octopus.toml`, which you can also edit by hand.

```toml
language = "auto"          # auto follows LANG; "pt" or "en"

[bar]
position = "left"          # or "right"
width = 40
style = "floating"         # or "flush": against the edge, concave corners
top = ["logo", "workspaces"]
center = ["clock"]
bottom = ["tray", "battery", "controls"]

[dock]
enabled = false
mode = "smart"             # "fixed", "autohide" or "smart"
pinned = ["firefox.desktop"]  # right-click an icon to pin it
size = 44                  # icon size, in pixels
magnify = true             # icons swell under the pointer

[weather]
place = ""                 # a city, or "lat,lon"; empty uses your ip

[desktop]
enabled = true             # widgets under the windows
widgets = ["weather", "contributions"]
github = ""                # a github user, for the contributions calendar
contributions_colors = "github"  # or "theme", the wallpaper's color

[notifications]
timeout_ms = 5000
max_popups = 3

[media]
lyrics = true
```

## when it applies

- **bar**: right away (waybar reloads on its own)
- **language and dock**: octopus restarts on its own when you apply
- **weather, notifications, lyrics**: after `octopus restart`

## your own css

a `~/.config/octopus/style.css` goes on top of the built-in css and reloads
on its own. good for tweaks without recompiling.

## dock

- **fixed**: keeps its own space, windows don't go under it
- **autohide**: goes away and comes back when the pointer touches the bottom edge
- **smart**: only steps aside when a window on the workspace sits under it

click: opens, focuses or cycles through the app's windows. middle: new
window. right: a menu with the windows, pin and close.

## desktop widgets

a layer under the windows, so they show wherever no window covers them:

- **weather**: now in a big cookie, and the next four days. a click opens the weather tab.
- **contributions**: the calendar from `github`'s profile, the little green squares, with the
  year's total and the current streak. hover a square for its day. it shows as many weeks as it
  is wide, and refreshes every half hour (no token).

drag a widget to move it, and drag the handle in its corner to resize it. where they sit is
kept in `~/.local/state/octopus/desktop.json`; delete that file to put them back in the corner.

changing `[desktop]` in the settings restarts octopus on its own.
