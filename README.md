# octopus

Shell M3 cozy pro niri: uma barra vertical minimalista na Waybar, com tudo que é maior (control center, dash, menus, notificações) num companion em Rust + GTK4.

Superfícies opacas e tonais, sem blur (só a tela de bloqueio desfoca). Português ou inglês. As cores saem do wallpaper via matugen.

## Peças

| Parte | O quê |
|---|---|
| `waybar/` | barra vertical: launcher, workspaces, relógio, bandeja, bateria em pílula com o percentual e o grupo com drawer |
| `companion/` | `octopus`, o daemon: control center, dash, menus (wi-fi, bluetooth, som, bandeja, clipboard, emoji, energia), configurações, notificações, OSD e a tela de bloqueio |
| `scripts/` | launcher (fuzzel), Matuwall, bateria, tema, emoji, clima e letras |
| `matugen/` | templates de cores pro companion, Waybar, fuzzel e Matuwall |
| `niri.kdl` | autostart, binds e layer rules, incluído no config do niri |

## Instalar

Depende de: `pam waybar fuzzel matugen matuwall awww cliphist wl-clipboard wtype playerctl cava brightnessctl networkmanager bluez-utils libpulse python unicode-emoji unicode-cldr gtk4 gtk4-layer-shell ttf-material-symbols-variable rust`.

O repo mora no próprio `~/.config/octopus`:

```sh
git clone <repo> ~/.config/octopus
~/.config/octopus/install.sh
```

O build fica em `~/.cache/octopus/target`.

Depois, no fim do `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

## Comandos

```
octopus                      # o daemon
octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|emoji|power>
octopus settings
octopus close
octopus lock
octopus volume <up|down|mute>
octopus mic mute
octopus brightness <up|down>
octopus kbd <up|down>
octopus dnd
octopus luma <imagem>         # dark ou light, usado pelo modo automático
```

## Configuração

`octopus.toml` (ou `octopus settings`): idioma, barra, lugar do clima, dispositivos de brilho, notificações e letras. Pra sobrescrever estilos sem mexer no CSS embutido, crie um `~/.config/octopus/style.css`; ele recarrega sozinho.
