# estrutura

O repo **é** o `~/.config/octopus`. Nada fica em `~/Projects`.

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
│   │   ├── launcher/     o spotlight (apps, calculadora, arquivos)
│   │   ├── dock/         a dock (opcional)
│   │   ├── lock/         a tela de bloqueio (PAM próprio)
│   │   ├── services/     o sistema por trás (áudio, rede, mídia, niri…)
│   │   └── widgets/      peças reaproveitadas (shapes, animação, sliders…)
│   └── style/octopus.css o CSS embutido no binário
├── waybar/           style.css, tray-arrow.c (o botão da bandeja animado)
├── scripts/          tema, bateria, clima, letras, emoji, wofi, matuwall
├── matugen/          templates de cor (companion, waybar, fuzzel, wofi, matuwall)
├── wofi/config       o wofi do Mod+D (o style.css é gerado)
├── niri.kdl          autostart e atalhos, incluído no config do niri
├── octopus.toml      as configurações (a janela de configurações escreve aqui)
└── docs/             isto aqui
```

## fora do repo

| Caminho | O quê |
|---|---|
| `~/.local/bin/octopus` | o binário instalado |
| `~/.local/bin/octopus-theme` | link pro `scripts/theme.sh` (o Matuwall chama) |
| `~/.config/waybar` | link pra `~/.config/octopus/waybar` |
| `~/.cache/octopus/target` | build do Rust |
| `~/.cache/octopus/tray-arrow.so` | o módulo da bandeja compilado |
| `~/.cache/octopus/` | miniaturas de wallpaper, capas, emojis |
| `~/.local/state/octopus/` | modo de cor, apps mais abertos, letras em cache |

Arquivos gerados (não editar, estão no `.gitignore`): `colors.css`,
`waybar/colors.css`, `waybar/layout.css`, `wofi/style.css`.
