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

## commands

```
octopus                    the daemon (niri starts it)
octopus toggle <panel>     control-center, dash, wifi, bluetooth, audio,
                           tray, clipboard, emoji, power, launcher
octopus open wallpapers    the dash on its wallpapers tab
octopus open weather [q]   the weather tab, asking it q (tomorrow, week…)
octopus settings           the settings window
octopus lock               lock
octopus close              closes any panel
octopus reload             writes waybar's config again
octopus restart            restarts the daemon (the language and the dock need it)
octopus luma <image>       says whether the image is dark or light
```
