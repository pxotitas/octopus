# octopus

[![GitHub repo](https://img.shields.io/badge/GitHub-pxotitas%2Foctopus-blue?logo=github)](https://github.com/pxotitas/octopus)
[![License](https://img.shields.io/github/license/pxotitas/octopus)](https://github.com/pxotitas/octopus/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/Made%20with-Rust-orange?logo=rust)](https://github.com/pxotitas/octopus)
[![GitHub stars](https://img.shields.io/github/stars/pxotitas/octopus?style=social)](https://github.com/pxotitas/octopus/stargazers)

shell m3 cozy pro niri: uma barra vertical minimalista na waybar, com tudo
que é maior (central de controle, dash, menus, notificações) num companion
em rust + gtk4.

superfícies opacas e tonais, sem blur (só a tela de bloqueio desfoca).
português ou inglês. as cores saem do wallpaper via matugen.

> essa é a branch das docs em português. o código fica na
> [`main`](https://github.com/pxotitas/octopus/tree/main); em inglês, na
> [`eng`](https://github.com/pxotitas/octopus/tree/eng).

## peças

| parte | o quê |
|---|---|
| `waybar/` | barra vertical: logo, workspaces, relógio, bandeja (com a setinha animada do `tray-arrow.c`), bateria em pílula com o percentual e o grupo com drawer |
| `companion/` | `octopus`, o daemon: central de controle, dash (com seletor de wallpaper), spotlight, menus (wi-fi, bluetooth, som, bandeja, clipboard, emoji, energia), dock, configurações, notificações, osd e a tela de bloqueio |
| `scripts/` | tema, bateria, clima, letras, emoji, wofi e matuwall |
| `matugen/` | templates de cor pro companion, waybar, fuzzel, wofi e matuwall |
| `niri.kdl` | autostart, atalhos e layer rules, incluído no config do niri |

mais detalhes em [`docs/`](docs/README.md).

## instalar

depende de: `pam waybar matugen matuwall awww cliphist wl-clipboard wtype
playerctl cava brightnessctl networkmanager bluez-utils libpulse python
unicode-emoji unicode-cldr fd gcc wofi gtk3 gtk4 gtk4-layer-shell
ttf-material-symbols-variable rust`.

o repo mora no próprio `~/.config/octopus`:

```sh
git clone https://github.com/pxotitas/octopus ~/.config/octopus
~/.config/octopus/install.sh
```

o build fica em `~/.cache/octopus/target`.

depois, no fim do `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

## comandos

```
octopus                      # o daemon
octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|emoji|power|launcher>
octopus open wallpapers      # dash na aba de wallpapers
octopus settings
octopus close
octopus lock
octopus volume <up|down|mute>
octopus mic mute
octopus brightness <up|down>
octopus kbd <up|down>
octopus dnd
octopus reload               # regera o config da waybar
octopus restart
octopus luma <imagem>        # dark ou light, usado pelo modo automático
```

## configuração

`octopus.toml` (ou `octopus settings`): idioma, barra, dock, clima,
notificações e letras. pra sobrescrever estilos sem mexer no css embutido,
crie um `~/.config/octopus/style.css`; ele recarrega sozinho.

## licença

gpl-3.0-or-later. umas partes vêm do [impasto](https://github.com/andreumassanet/impasto)
e do [velatura](https://github.com/pxdritz1/velatura); ver o
[`NOTICE.md`](https://github.com/pxotitas/octopus/blob/main/NOTICE.md) na main.
