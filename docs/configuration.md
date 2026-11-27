# configuration

almost everything can be changed from the settings window (Mod+Comma, or
`octopus settings`). it writes to `~/.config/octopus/octopus.toml`, which you
can also edit by hand.

## the settings window

a tree on the left: five big categories, each with its branches.

- **look**: colors (mode, palette style, contrast) and language
- **bar**: the editor, presets, shape and anchoring
- **panels**: dash, notifications, weather, music and calendar
- **panels**: also the control center and the reminders
- **desktop**: widgets (with each one's options) and the dock
- **system**: lock screen and about

`octopus settings bar/editor` (or just `octopus settings bar`) opens straight
on a page. changes wait in a draft until you press **apply**. the gear at the
foot of the dash's rail opens it too.

## the bar editor

the bar drawn as it will look. tap a module to change it in the inspector
beside it (place, look, behavior); drag it to another spot, or onto **drop
here to remove** to take it out.

the **+ add** under the bar opens a menu with two things:

- **add module**: the catalog, in the inspector's place. search, filter by
  kind (essentials, system, audio and media, shortcuts, yours), pick the part
  of the bar, then click a card or drag it onto the bar.
- **make module**: a window that builds one of your own, with the module
  drawn live in a little bar while you fill it in.

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

a module of your own has a name, a material symbol (search them, or type
any name), what it shows (just the icon, or the icon and a value from a
command, with a **test** button and how often it runs), a command for the
click and a look. **edit module** in the inspector opens the same window.

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

[calendar]
urls = []                  # ical addresses: google's secret one, outlook's, nextcloud's…

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

[control_center]
sections = ["header", "tiles", "sliders", "media", "notifications"]  # order and which
tiles = ["wifi", "bluetooth", "mic", "theme"]   # also caffeine, dnd
sliders = ["volume", "mic", "brightness", "keyboard"]
date = true                # in the header
weather = true
settings = true            # the gear beside power
power = true
visualizer = true          # the bars behind the player

[reminders]
enabled = true             # the daemon; off, nothing rings
snooze = 10                # minutes

[dock]
enabled = false
mode = "smart"             # "fixed", "autohide" or "smart"
pinned = ["firefox.desktop"]  # right-click an icon to pin it
size = 44
magnify = true

[desktop]
enabled = true
widgets = ["weather", "contributions"]  # also clock, media, calendar, agenda,
                                        # system, battery, note, photo
github = ""                # a github user, for the contributions calendar
contributions_colors = "github"  # or "theme", the wallpaper's color
card = "surface"           # the cards' color: surface, tonal, primary, none
corners = 32               # 0 to 48
snap = 0                   # a grid to snap to while arranging, in px; 0 is free
hide_under_windows = true  # a widget steps aside (and stops updating) under a window

[desktop.clock]
face = "analog"            # or "digital", hours over minutes
seconds = true             # the wave round the rim, on the analog one
hours = 24                 # or 12, on the digital one

[desktop.weather]
days = 4                   # the next days underneath, 0 to 6
details = true             # the place and feels like

[desktop.agenda]
count = 6

[desktop.system]
meters = ["cpu", "memory", "disk"]

[desktop.battery]
time = true                # how long until full or empty

[desktop.note]
size = 15                  # the text, in px

[desktop.photo]
fit = "cover"              # or "contain", the whole picture
shuffle = 0                # minutes between photos from the same folder; 0 keeps one
```

## when it applies

- **bar and lyrics' timing**: right away
- **colors (mode)**: right away; palette style and contrast repaint everything when applied
- **everything else**: octopus restarts on its own when you apply

## calendar

the dash's calendar and the desktop's show your events: days with something
get a tint, and a click on a day lists them. it reads ical addresses, so a
google calendar needs no login and nothing in the cloud console:

1. open google calendar on the web, **settings**
2. on the left, under **settings for my calendars**, pick the calendar
3. **integrate calendar**, copy the **secret address in ical format**
4. paste it in **panels → calendar** (or in `[calendar] urls`)

any other ical works the same (outlook, nextcloud, fastmail, a holidays
feed). it refreshes every 15 minutes and keeps a copy in
`~/.cache/octopus/calendar`, so it shows offline too. the secret address lets
anyone read the calendar: don't share it. reading recurring events needs
`python-dateutil`.

## control center

Mod+C. its sections (header, tiles, sliders, music, notifications) can be
turned off and put in any order in **panels → control center**, and so can
the tiles and the sliders. the gear beside power opens the settings.

the **caffeine** tile (or `octopus caffeine`, `octopus caffeine on|off`) keeps
the screen awake: no dimming, locking or sleep until you turn it off.
**do not disturb** keeps the toasts quiet; they still go to the list.

## reminders

a daemon inside octopus (it can be turned off in **panels → reminders**). it
rings at the time with a toast that snoozes or dismisses; an **alarm** loops a
sound until you answer it.

- from the spotlight: `!` then when and what. `!10 min tea`, `!18h bread`,
  `!tomorrow at 9 dentist`, `!friday 18h call mom`, `!alarm 6:45`
- from a terminal: `octopus remind 10 min tea`
- the little app (`octopus reminders`, or reminders in your apps menu)
  lists them, turns them on and off, edits them and makes them repeat

they are kept in `~/.local/state/octopus/reminders.json`.

## files

`octopus files [folder]` opens **octopus files**, a file manager in the same
m3 look, after the macos finder:

- **views**: icons (with a zoom slider), list and columns (<kbd>ctrl</kbd>
  <kbd>1</kbd>/<kbd>2</kbd>/<kbd>3</kbd>). in columns, a click on a folder
  opens it beside, <kbd>→</kbd>/<kbd>←</kbd> go in and out, and a file gets a
  preview at the end
- **sidebar**: favorites, bookmarks (<kbd>ctrl</kbd><kbd>shift</kbd><kbd>d</kbd>,
  remove with the × or a right-click), disks with how full they are, the
  trash, the network and colored tags
- **quick look** with <kbd>space</kbd>, **get info** with <kbd>ctrl</kbd><kbd>i</kbd>,
  tabs, undo, compress and extract, open with, a terminal here
- **search**: typing filters the folder; <kbd>enter</kbd> looks through every
  folder under it with `fd`, skipping `.git`, `node_modules` and `target`
- **git**: inside a repository a bar shows the branch (click to switch or make
  one), what changed, what is only local, and pull, push and **commit** (pick
  the files, write the message, and push too if you want). **clone
  repository…** (the + button, the ⋮ menu or a right-click) takes an address
  or picks from your github repositories; **connect to github** signs `gh` in
  through the browser
- **localsend**: right-click → **send with localsend…** finds phones and
  computers with localsend open on the same network
- **on the network**: computers that announce sftp or smb, and localsend
  devices. **connect to server** takes `sftp://`, `smb://`, `ftp://`…
- **google drive**: through `rclone`, signing in on the browser; it mounts in
  `~/Google Drive`

the installer asks whether to install it and whether to make it the default,
which also answers apps' "show in folder".

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
- **clock**: analog or digital, on a cookie.
- **music**: what's playing, with the controls.
- **calendar**: the whole month, with your events.
- **agenda**: what's coming up from your calendars.
- **system**: processor, memory and disk, as bars.
- **battery**: a big pill with the bolt, and how long until full or empty.
- **note**: a note that saves itself as you type.
- **photo**: a picture of yours, filling the card or whole; the button on hover picks
  another, and it can go through the folder's photos on its own.

in **desktop → widgets**, the cards' color, corners and grid sit at the top;
each widget's ⚙ opens its own options under it.

they stay put until you arrange them: right-click the desktop and pick
**arrange widgets** (or `octopus desktop arrange`). the desktop comes up over
the windows; drag a widget to move it, its corner to resize it, and use the
case at the bottom to add the others (drag one out, or click it) or put one
away (drop it on the case, or its ×). **done** or <kbd>esc</kbd> ends it.

where they sit is kept in `~/.local/state/octopus/desktop.json`; delete that
file to put them back in the corner.

with **step aside from windows** on (`hide_under_windows`), a widget under a
window fades out and stops updating until the window goes; with all of them
covered the whole layer goes away, so a busy workspace costs nothing.
