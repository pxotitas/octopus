# configuração

quase tudo dá pra mudar pela janela (Mod+Comma). ela escreve em
`~/.config/octopus/octopus.toml`, que também pode ser editado à mão.

```toml
language = "auto"          # auto segue o LANG; "pt" ou "en"

[bar]
position = "left"          # ou "right"
width = 40
style = "floating"         # ou "flush": encostada na borda, cantos côncavos
top = ["logo", "workspaces"]
center = ["clock"]
bottom = ["tray", "battery", "controls"]

[dock]
enabled = false
mode = "smart"             # "fixed", "autohide" ou "smart"
pinned = ["firefox.desktop"]  # clique direito num ícone pra fixar
size = 44                  # tamanho dos ícones, em pixels
magnify = true             # os ícones crescem embaixo do ponteiro

[weather]
place = ""                 # uma cidade, ou "lat,lon"; vazio usa o ip

[desktop]
enabled = true             # widgets embaixo das janelas
widgets = ["weather", "github"]
position = "top-right"     # ou top-left, bottom-right, bottom-left
github = ""                # um usuário do github, pro widget de commits

[notifications]
timeout_ms = 5000
max_popups = 3

[media]
lyrics = true
```

## quando vale

- **barra**: na hora (a waybar recarrega sozinha)
- **idioma e dock**: o octopus reinicia sozinho ao aplicar
- **clima, notificações, letras**: depois de `octopus restart`

## css pessoal

um `~/.config/octopus/style.css` é somado por cima do css embutido e
recarrega sozinho. bom pra ajustes sem recompilar.

## dock

- **fixa**: ocupa o espaço dela, as janelas não passam por baixo
- **esconde sozinha**: some e volta quando o ponteiro encosta na borda de baixo
- **inteligente**: só sai da frente quando uma janela do workspace fica embaixo dela

clique: abre, foca ou alterna entre as janelas do app. meio: janela nova.
direito: menu com as janelas, fixar e fechar.

## widgets de desktop

uma camada embaixo das janelas, então eles aparecem onde nenhuma janela cobre:

- **weather**: o clima agora num cookie grande, e os próximos quatro dias. um clique abre a aba de clima.
- **github**: os últimos commits de `github`, pela busca pública de commits do github (sem token). um clique abre o commit.

mudar o `[desktop]` nas configurações reinicia o octopus sozinho.
