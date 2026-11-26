<p align="center">
  <img src="assets/logo.svg" width="128" alt="logo do octopus">
</p>

<h1 align="center">octopus</h1>

<p align="center">
  <a href="https://pxotitas.github.io/octopus/pt/">site</a> ·
  <a href="https://pxotitas.github.io/octopus/pt/docs/">docs</a> ·
  <a href="https://pxotitas.github.io/octopus/pt/media/">screenshots e vídeos</a> ·
  <a href="https://pxotitas.github.io/octopus/pt/changelog/">changelog</a>
</p>

<p align="center">
  <a href="https://github.com/pxotitas/octopus"><img src="https://img.shields.io/badge/github-pxotitas%2Foctopus-blue?logo=github" alt="repo no github"></a>
  <a href="https://github.com/pxotitas/octopus/blob/main/LICENSE"><img src="https://img.shields.io/badge/licen%C3%A7a-gpl--3.0--or--later-blue" alt="licença"></a>
  <a href="https://github.com/pxotitas/octopus"><img src="https://img.shields.io/badge/feito%20com-rust-orange?logo=rust" alt="rust"></a>
  <a href="https://github.com/pxotitas/octopus/stargazers"><img src="https://img.shields.io/github/stars/pxotitas/octopus?label=estrelas&logo=github&color=yellow&style=flat" alt="estrelas no github"></a>
</p>

shell m3 cozy pro niri: uma barra vertical minimalista na waybar, com tudo
que é maior (central de controle, dash, menus, notificações) num companion
em rust + gtk4.

superfícies opacas e tonais, sem blur (só a tela de bloqueio desfoca).
português ou inglês. as cores saem do wallpaper via matugen.

> essa é a branch das docs em português. o código fica na
> [`main`](https://github.com/pxotitas/octopus/tree/main); em inglês, na
> [`eng`](https://github.com/pxotitas/octopus/tree/eng).

## showcase

o vídeo inteiro, com som, tá no [site](https://pxotitas.github.io/octopus/pt/media/). aqui ele tá cortado em clipes curtos; clica num pra abrir.

<details>
<summary><b>dash</b>: visão geral, música e a letra sincronizada</summary>
<br>

<img src="screenshots/clips/dash.webp" alt="dash" width="100%">

</details>

<details>
<summary><b>clima e wallpapers</b>: a previsão, depois o seletor de wallpaper</summary>
<br>

<img src="screenshots/clips/weather.webp" alt="clima e wallpapers" width="100%">

</details>

<details>
<summary><b>central de controle</b>: atalhos, sliders e o player</summary>
<br>

<img src="screenshots/clips/control-center.webp" alt="central de controle" width="100%">

</details>

<details>
<summary><b>tela de bloqueio</b>: shapes que mudam a cada tecla, com a letra sendo cantada</summary>
<br>

<img src="screenshots/clips/lock.webp" alt="tela de bloqueio" width="100%">

</details>

<details>
<summary><b>launchers</b>: o rofi e o spotlight</summary>
<br>

<img src="screenshots/clips/launchers.webp" alt="launchers" width="100%">

</details>

<details>
<summary><b>emoji</b>: o seletor, com busca</summary>
<br>

<img src="screenshots/clips/emoji.webp" alt="emoji" width="100%">

</details>

<details>
<summary><b>configurações</b>: cores, idioma, barra, widgets e dock</summary>
<br>

<img src="screenshots/clips/settings.webp" alt="configurações" width="100%">

</details>

<details>
<summary><b>conta no spotlight</b>: digitando uma soma com <code>=</code></summary>
<br>

<img src="screenshots/clips/calculator.webp" alt="conta no spotlight" width="100%">

</details>

<details>
<summary><b>paleta do wallpaper</b>: um wallpaper novo repinta o shell inteiro</summary>
<br>

<img src="screenshots/clips/palette.webp" alt="paleta do wallpaper" width="100%">

</details>

## screenshots

estão todas em [`screenshots/`](screenshots). clica num grupo pra abrir.

<details>
<summary><b>desktop</b>: a barra, os widgets de desktop, notificações e o osd</summary>
<br>

a barra vertical na esquerda, e os widgets de desktop na direita: o clima e as contribuições no github, que dá pra mover e redimensionar.

<img src="screenshots/desktop.webp" alt="desktop com a barra e a dock" width="100%">

uma notificação, com o ícone do app.

<img src="screenshots/notification.webp" alt="uma notificação" width="100%">

o osd, pra volume e brilho.

<img src="screenshots/osd.webp" alt="osd de volume" width="100%">

</details>

<details>
<summary><b>central de controle</b>: atalhos, sliders, player e notificações</summary>
<br>

<img src="screenshots/control-center.webp" alt="central de controle" width="100%">

</details>

<details>
<summary><b>dash</b>: rail, relógios, calendário, player, notificações, clima e wallpapers</summary>
<br>

a visão geral, que abre pelo relógio da barra: rail de navegação, um relógio analógico num cookie, e as notificações em cima de um card de clima pequeno.

<img src="screenshots/dash.webp" alt="visão geral do dash" width="100%">

a aba de wallpapers; escolher um também gera a paleta.

<img src="screenshots/wallpapers.webp" alt="seletor de wallpaper no dash" width="100%">

</details>

<details>
<summary><b>clima</b>: shapes m3, e perguntas depois de um <code>&gt;</code></summary>
<br>

agora: métricas em shapes m3, o arco do sol e a lua, as próximas 24 horas e os dias.

<img src="screenshots/weather.webp" alt="a aba de clima" width="100%">

<code>&gt; amanhã</code>: um dia, e como ele fica comparado com hoje.

<img src="screenshots/weather-tomorrow.webp" alt="o clima de amanhã" width="100%">

<code>&gt; ontem</code>: tem uma semana pra trás também.

<img src="screenshots/weather-yesterday.webp" alt="o clima de ontem" width="100%">

<code>&gt; semana</code> e <code>&gt; fim de semana</code>: os dias em shapes, com o mais quente, o mais fresco e o mais chuvoso.

<img src="screenshots/weather-week.webp" alt="o clima da semana" width="100%">

<img src="screenshots/weather-weekend.webp" alt="o clima do fim de semana" width="100%">

</details>

<details>
<summary><b>menus</b>: wi-fi, bluetooth, som, bandeja e energia</summary>
<br>

wi-fi

<img src="screenshots/wifi.webp" alt="menu de wi-fi" width="100%">

bluetooth

<img src="screenshots/bluetooth.webp" alt="menu de bluetooth" width="100%">

som, com saída, microfone e dispositivos

<img src="screenshots/audio.webp" alt="menu de som" width="100%">

bandeja

<img src="screenshots/tray.webp" alt="menu da bandeja" width="100%">

energia

<img src="screenshots/power.webp" alt="menu de energia" width="100%">

</details>

<details>
<summary><b>launchers</b>: o spotlight e o rofi</summary>
<br>

o spotlight (mod+space): apps, `=` pra conta, `/` pra arquivos e `>` pra energia.

<img src="screenshots/launcher.webp" alt="spotlight" width="100%">

o rofi (mod+d), com os apps ordenados pelo uso.

<img src="screenshots/rofi.webp" alt="rofi" width="100%">

</details>

<details>
<summary><b>ferramentas</b>: clipboard e emoji</summary>
<br>

clipboard (mod+shift+v)

<img src="screenshots/clipboard.webp" alt="histórico do clipboard" width="100%">

emoji (mod+.)

<img src="screenshots/emoji.webp" alt="seletor de emoji" width="100%">

</details>

<details>
<summary><b>configurações</b>: cores, idioma, barra, widgets e dock</summary>
<br>

<img src="screenshots/settings.webp" alt="janela de configurações" width="100%">

</details>

> as screenshots estão com o shell em inglês.

## peças

| parte | o quê |
|---|---|
| `waybar/` | barra vertical: logo, workspaces, relógio, bandeja (com a setinha animada do `tray-arrow.c`), bateria em pílula com o percentual e o grupo com drawer |
| `companion/` | `octopus`, o daemon: central de controle, dash (com seletor de wallpaper), spotlight, menus (wi-fi, bluetooth, som, bandeja, clipboard, emoji, energia), dock, configurações, notificações, osd e a tela de bloqueio |
| `scripts/` | tema, bateria, clima, letras, emoji, rofi e matuwall |
| `matugen/` | templates de cor pro companion, waybar, fuzzel, rofi e matuwall |
| `niri.kdl` | autostart, atalhos e layer rules, incluído no config do niri |

mais detalhes em [`docs/`](docs/README.md).

## instalar

depende de: `pam waybar matugen matuwall awww cliphist wl-clipboard wtype
playerctl cava brightnessctl networkmanager bluez-utils libpulse python python-dateutil
unicode-emoji unicode-cldr fd gcc rofi gtk3 gtk4 gtk4-layer-shell
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
[`notice.md`](https://github.com/pxotitas/octopus/blob/main/NOTICE.md) na main.
