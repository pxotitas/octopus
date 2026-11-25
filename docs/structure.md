# structure

the repo **is** `~/.config/octopus`. nothing lives in `~/Projects`.

the docs (this) live on the `eng` branch (and in portuguese on `pt-br`);
`main` has only the shell.

```
~/.config/octopus/
├── companion/        the daemon in rust + gtk4 (the `octopus` binary)
│   ├── src/
│   │   ├── main.rs       entry: daemon, socket client, `luma`
│   │   ├── shell.rs      ties it all together: makes the panels, answers commands
│   │   ├── popup.rs      layer windows that open and close animated
│   │   ├── lazy.rs       panels built only when they open and dropped later
│   │   ├── bar.rs        writes waybar's config from octopus.toml
│   │   ├── i18n.rs       portuguese → english
│   │   ├── panels/       one file per panel (control center, dash, menus…)
│   │   ├── launcher/     the spotlight (apps, calculator, files)
│   │   ├── dock/         the dock (optional), with the magnification in shelf.rs
│   │   ├── lock/         the lock screen (its own pam)
│   │   ├── services/     the system behind it all (audio, network, media, niri…)
│   │   └── widgets/      reused pieces (shapes, animation, sliders…)
│   └── style/octopus.css the css built into the binary
├── waybar/           style.css, tray-arrow.c (the animated tray button)
├── scripts/          theme, battery, weather, lyrics, emoji, rofi, matuwall
├── matugen/          color templates (companion, waybar, fuzzel, rofi, matuwall)
├── rofi/config.rasi  the rofi on Mod+D (its theme.rasi is generated)
├── niri.kdl          autostart and shortcuts, included from niri's config
├── octopus.toml      the settings (the settings window writes here)
├── LICENSE           gpl-3.0
└── NOTICE.md         what came from impasto and velatura
```

## outside the repo

| path | what |
|---|---|
| `~/.local/bin/octopus` | the installed binary |
| `~/.local/bin/octopus-theme` | link to `scripts/theme.sh` (matuwall calls it) |
| `~/.config/waybar` | link to `~/.config/octopus/waybar` |
| `~/.cache/octopus/target` | the rust build |
| `~/.cache/octopus/tray-arrow.so` | the compiled tray module |
| `~/.cache/octopus/` | wallpaper thumbnails, covers, emojis |
| `~/.local/state/octopus/` | color mode, most-opened apps, cached lyrics |

generated files (don't edit, they're in `.gitignore`): `colors.css`,
`waybar/colors.css`, `waybar/layout.css`, `rofi/theme.rasi`.
