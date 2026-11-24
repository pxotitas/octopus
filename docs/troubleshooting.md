# when something breaks

**panels don't open / `octopus: … is the daemon running?`**
the daemon died. running `GSK_RENDERER=gl octopus &` in a terminal shows the error.

**the bar is gone or looks wrong**
`octopus reload` writes its config again. if waybar died: `waybar &`.
css errors show up when you run `waybar` in a terminal.

**the tray arrow doesn't show**
`~/.cache/octopus/tray-arrow.so` is missing. run `install.sh` again
(it needs `gcc` and `gtk3`). without it the bar uses the text button.

**stuck on the lock screen**
Ctrl+Alt+F2, log in, and `pkill -x octopus`. niri stays locked until a new
lock unlocks it: `niri msg action spawn -- octopus` and then `octopus lock`
again, or log out.

**colors don't change**
check that matuwall has `on_apply = ["octopus-theme {path}"]` and that
`~/.local/bin/octopus-theme` exists.

**recompile**
```
cd ~/.config/octopus && ./install.sh
octopus restart
```

**memory**
`grep RssAnon /proc/$(pgrep -x octopus)/status` shows what belongs to the
daemon (without the shared libraries). it sits around 45 mb when idle.
