# octopus

Shell M3 cozy pro niri: uma barra vertical minimalista na Waybar, com tudo que é maior (control center, dash, menus, notificações) num companion em Rust + GTK4.

Superfícies opacas e tonais, sem blur. As cores saem do wallpaper via matugen.

## Peças

| Parte | O quê |
|---|---|
| `waybar/` | barra vertical: launcher, workspaces, janela focada, relógio, tray, bateria em pílula e o grupo com drawer |
| `companion/` | `octopus`, o daemon: control center, dash, menus de wi-fi/bluetooth/som, notificações, OSD e a tela de bloqueio |
| `config/scripts/` | fuzzel (apps, clipboard, emoji, energia), Matuwall, bateria, tema, clima e letras |
| `config/matugen/` | templates de cores pro companion, Waybar, fuzzel e Matuwall |
| `config/niri.kdl` | autostart, binds e layer rules, incluído no config do niri |

## Instalar

Depende de: `pam waybar fuzzel matugen matuwall awww cliphist wl-clipboard wtype playerctl cava brightnessctl networkmanager bluez-utils libpulse python unicode-emoji unicode-cldr gtk4 gtk4-layer-shell ttf-material-symbols-variable rust`.

```sh
./install.sh
```

Depois, no fim do `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

## Comandos

```
octopus                      # o daemon
octopus toggle <control-center|dash|wifi|bluetooth|audio>
octopus close
octopus lock
octopus volume <up|down|mute>
octopus mic mute
octopus brightness <up|down>
octopus kbd <up|down>
octopus dnd
```

## Configuração

`config/octopus.toml`: lugar do clima, dispositivos de brilho, notificações e letras. Pra sobrescrever estilos sem mexer no CSS embutido, crie um `~/.config/octopus/style.css`; ele recarrega sozinho.
