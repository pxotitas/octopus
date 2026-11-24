# configuração

Quase tudo dá pra mudar pela janela (Mod+Comma). Ela escreve em
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
pinned = ["firefox.desktop"]

[weather]
place = ""                 # vazio usa o ip

[notifications]
timeout_ms = 5000
max_popups = 3

[media]
lyrics = true
```

## quando vale

- **barra**: na hora (a Waybar recarrega sozinha)
- **idioma e dock**: o octopus reinicia sozinho ao aplicar
- **clima, notificações, letras**: depois de `octopus restart`

## CSS pessoal

Um `~/.config/octopus/style.css` é somado por cima do CSS embutido e
recarrega sozinho. Bom pra ajustes sem recompilar.
