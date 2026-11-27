# estrutura

o repo **é** o `~/.config/octopus`. nada fica em `~/Projects`.

as docs (isto aqui) moram na branch `pt-br` (e em inglês na `eng`); a
`main` tem só o shell.

```
~/.config/octopus/
├── companion/        o daemon em Rust + GTK4 (binário `octopus`)
│   ├── src/
│   │   ├── main.rs       entrada: daemon, cliente do socket, `luma`
│   │   ├── shell.rs      junta tudo: cria os painéis e responde aos comandos
│   │   ├── popup.rs      janelas de layer que abrem e fecham animadas
│   │   ├── lazy.rs       painéis montados só ao abrir e soltos depois
│   │   ├── bar.rs        gera o config da Waybar a partir do octopus.toml
│   │   ├── i18n.rs       português → inglês
│   │   ├── panels/       um arquivo por painel (control center, dash, menus…)
│   │   ├── launcher/     o spotlight (apps, calculadora, arquivos, lembretes)
│   │   ├── files/        o octopus files, o gerenciador de arquivos (git.rs, columns.rs, share.rs…)
│   │   ├── dock/         a dock (opcional), com a magnificação em shelf.rs
│   │   ├── lock/         a tela de bloqueio (PAM próprio)
│   │   ├── services/     o sistema por trás (áudio, rede, mídia, niri, lembretes…)
│   │   └── widgets/      peças reaproveitadas (shapes, animação, sliders…)
│   └── style/octopus.css o CSS embutido no binário
├── waybar/           style.css, tray-arrow.c (o botão da bandeja animado)
├── scripts/          tema, bateria, clima, letras, emoji, rofi, matuwall, localsend
├── matugen/          templates de cor (companion, waybar, fuzzel, rofi, matuwall)
├── rofi/config.rasi  o rofi do Mod+D (o theme.rasi é gerado)
├── niri.kdl          autostart e atalhos, incluído no config do niri
├── octopus.toml      as configurações (a janela de configurações escreve aqui)
├── LICENSE           gpl-3.0
└── NOTICE.md         o que veio do impasto e do velatura
```

## fora do repo

| caminho | o quê |
|---|---|
| `~/.local/bin/octopus` | o binário instalado |
| `~/.local/bin/octopus-theme` | link pro `scripts/theme.sh` (o matuwall chama) |
| `~/.config/waybar` | link pra `~/.config/octopus/waybar` |
| `~/.cache/octopus/target` | build do rust |
| `~/.cache/octopus/tray-arrow.so` | o módulo da bandeja compilado |
| `~/.cache/octopus/` | miniaturas de wallpaper, capas, emojis |
| `~/.local/state/octopus/` | modo de cor, apps mais abertos, letras em cache |

arquivos gerados (não editar, estão no `.gitignore`): `colors.css`,
`waybar/colors.css`, `waybar/layout.css`, `rofi/theme.rasi`.
