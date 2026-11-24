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
place = ""                 # empty uses your ip

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
