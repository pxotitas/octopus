# configuration

almost everything can be changed from the settings window (Mod+Comma, or
`octopus settings`). it writes to `~/.config/octopus/octopus.toml`, which you
can also edit by hand.

## the settings window

a tree on the left: five big categories, each with its branches.

- **look**: colors (mode, palette style, contrast) and language
- **bar**: the editor, presets, shape and anchoring
- **panels**: dash, notifications, weather and music
- **desktop**: widgets, contributions and the dock
- **system**: lock screen and about

`octopus settings bar/editor` (or just `octopus settings bar`) opens straight
on a page. changes wait in a draft until you press **apply**.

## the bar editor

the bar drawn as it will look. tap a module to change it in the inspector
beside it; drag it to another spot, or to the catalog to take it out; drag
new ones in from the catalog.

| module | what it is | options |
|---|---|---|
| `logo` | opens the launcher | the octopus, your distro or a symbol; what a click opens; background and shape |
| `workspaces` | niri's workspaces | dots, numbers, pills, squares or lines |
| `clock` | hours over minutes; opens the dash | 24 or 12 hours, background and shape (the bun too) |
| `tray` | background apps | |
| `battery` | a pill that fills up, with a bolt while charging | show the percentage |
| `controls` | control center, wi-fi, bluetooth, sound, power | which buttons, in a drawer or always shown |
| `cpu`, `memory`, `temperature` | usage and heat | show the value, background, corners |
| `brightness`, `volume` | scroll to change | show the value, background, corners |
| `idle` | keeps the screen awake | background, corners |
| `media` | play and pause | background, corners |
| `notifications`, `power` | shortcuts | background, corners |

the **shape** behind a module is either rounded corners (pill, rounded,
square) or any m3 shape: cookies, clovers, flower, sunny, burst… drawn in the
theme's colors, and drawn again when the wallpaper changes. **module presets**
set the background and the shape in one tap.

**make a module** (at the end of the catalog) gives you one of your own: a
name, a material symbol, a command whose output shows under the icon, how
often it runs, and a command for the click.

**presets** sets the whole bar at once: default, minimal, monitor, music and
focus. **anchoring** chooses the side, the layer (below, on top, or over
everything), whether windows keep out of its way, and the gaps to the edges.
the bar changes when you apply, without restarting.

## octopus.toml

```toml
language = "auto"          # auto follows LANG; "pt" or "en"

[look]
scheme = "tonal-spot"      # matugen's palette: expressive, vibrant, fidelity, content,
                           # neutral, monochrome, rainbow, fruit-salad
contrast = 0.0             # -1 to 1

[bar]
position = "left"          # or "right"
width = 52
spacing = 4                # between modules
layer = "top"              # "bottom", "top" or "overlay"
exclusive = true           # windows keep out of its way
margin_top = 8             # gaps to the edges, when floating
margin_bottom = 8
margin_edge = 8
style = "floating"         # or "flush": against the edge, concave corners
top = ["logo", "workspaces"]
center = ["clock"]
bottom = ["tray", "battery", "controls"]

[bar.modules.clock]        # one table per module you changed
background = "primary"     # none, tonal, primary, surface
shape = "bun"              # bun (clock only), round, soft, square, or an m3 shape:
                           # cookie4…12, clover4, clover8, flower, sunny, burst…
variant = "24h"            # the module's own: 12h; for the logo "octopus", "arch", "pets"…;
                           # for workspaces dots, numbers, pills, squares, bars
text = false               # the value under the icon
action = ""                # the logo: launcher, spotlight or dash
items = []                 # the controls' buttons

[[bar.custom]]             # a module of your own; put "user-1" in top/center/bottom
id = "user-1"
name = "uptime"
icon = "timer"             # a material symbol
exec = "uptime -p"         # its output is the value (turn on text)
interval = 60
click = ""

[dash]
page = "overview"          # or media, weather, wallpapers

[weather]
place = ""                 # a city, or "lat,lon"; empty uses your ip
unit = "celsius"           # or "fahrenheit"

[notifications]
timeout_ms = 5000
max_popups = 3
position = "top-right"     # top-left, bottom-right, bottom-left

[media]
lyrics = true              # in the dash
lyrics_offset_ms = 0

[lock]
lyrics = true

[dock]
enabled = false
mode = "smart"             # "fixed", "autohide" or "smart"
pinned = ["firefox.desktop"]  # right-click an icon to pin it
size = 44
magnify = true

[desktop]
enabled = true
widgets = ["weather", "contributions"]  # also clock, media, calendar
github = ""                # a github user, for the contributions calendar
contributions_colors = "github"  # or "theme", the wallpaper's color
```

## when it applies

- **bar and lyrics' timing**: right away
- **colors (mode)**: right away; palette style and contrast repaint everything when applied
- **everything else**: octopus restarts on its own when you apply

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
- **clock**: analog, on a cookie.
- **music**: what's playing, with the controls.
- **calendar**: the whole month.

they stay put until you arrange them: right-click the desktop and pick
**arrange widgets** (or `octopus desktop arrange`). the desktop comes up over
the windows; drag a widget to move it, its corner to resize it, and use the
case at the bottom to add the others (drag one out, or click it) or put one
away (drop it on the case, or its ×). **done** or <kbd>esc</kbd> ends it.

where they sit is kept in `~/.local/state/octopus/desktop.json`; delete that
file to put them back in the corner.
