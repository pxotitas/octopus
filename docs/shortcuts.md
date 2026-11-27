# shortcuts and commands

## niri (`niri.kdl`)

| shortcut | does |
|---|---|
| Mod+Space, XF86LaunchB | spotlight |
| Mod+D | rofi (apps) |
| Mod+Shift+D | fuzzel (comes from your config.kdl, not from octopus) |
| Mod+C | control center |
| Mod+A | dash |
| Mod+Shift+V | clipboard |
| Mod+Period | emoji |
| Mod+Shift+E | power |
| Mod+Comma | settings |
| Mod+L | lock |
| Mod+Shift+W | matuwall |

## spotlight

- type: apps and their actions (e.g. "new private window")
- a sum (`2*(3+4)`, `sqrt 16`, `1,5 x 2`): calculator, enter copies it
- `/name`: files in your home folder (uses `fd`)
- `!when what`: a reminder (`!10 min tea`, `!alarm 6:45`)
- `>`: lock, suspend, log out, restart, shut down

## weather

the line at the top of the weather tab (after the chevron) takes questions,
in portuguese or english:

- `tomorrow`, `yesterday`, `today`, `now`, `day after tomorrow`
- a weekday: `thursday`, `next thursday`, `last friday`
- days from today: `+3`, `-2`, `in 4 days`, `2 days ago`
- a date: `11/28` (day first: `28/11`), `2026-12-01`, `nov 30`
- `week`, `next week`, `last week`, `weekend`

it reaches a week back and two weeks ahead. escape goes back to now.

## octopus files

| keys | does |
|---|---|
| ctrl 1 / 2 / 3 | icons, list, columns |
| space | quick look |
| ctrl i | get info |
| f2 | rename |
| ctrl shift n | new folder |
| ctrl t, ctrl w, ctrl n | new tab, close tab, new window |
| ctrl l | type a path |
| ctrl f | search (enter searches every folder under this one) |
| ctrl shift d | bookmark this folder |
| ctrl h | show hidden files |
| ctrl z | undo |
| delete, shift delete | to the trash, delete for good |
| alt ← → ↑, backspace | back, forward, up |
| ctrl alt t | a terminal here |

## commands

```
octopus                    the daemon (niri starts it)
octopus toggle <panel>     control-center, dash, wifi, bluetooth, audio,
                           tray, clipboard, emoji, power, launcher
octopus open wallpapers    the dash on its wallpapers tab
octopus open weather [q]   the weather tab, asking it q (tomorrow, week…)
octopus settings           the settings window
octopus files [folder]     octopus files
octopus remind <when what> a reminder: 10 min tea, 18h bread, alarm 6:45
octopus reminders          the reminders app
octopus caffeine [on|off]  keeps the screen awake (toggles without on/off)
octopus dnd                do not disturb
octopus desktop arrange    arrange the desktop widgets
octopus lock               lock
octopus close              closes any panel
octopus reload             writes waybar's config again
octopus restart            restarts the daemon (the language and the dock need it)
octopus luma <image>       says whether the image is dark or light
```
