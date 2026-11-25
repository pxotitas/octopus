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
widgets = ["weather", "github"]
position = "top-right"     # or top-left, bottom-right, bottom-left
github = ""                # a github user, for the commits widget

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
- **github**: the latest commits of `github`, from github's public commit search (no token). a click opens the commit.

changing `[desktop]` in the settings restarts octopus on its own.
