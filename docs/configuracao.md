# configuração

quase tudo dá pra mudar pela janela de configurações (Mod+Comma, ou
`octopus settings`). ela escreve em `~/.config/octopus/octopus.toml`, que
também pode ser editado à mão.

## a janela de configurações

uma árvore na esquerda: cinco categorias grandes, cada uma com seus galhos.

- **aparência**: cores (modo, estilo da paleta, contraste) e idioma
- **barra**: o editor, presets e forma
- **painéis**: dash, notificações, clima e música
- **área de trabalho**: widgets, contribuições e a dock
- **sistema**: tela de bloqueio e sobre

`octopus settings bar/editor` (ou só `octopus settings bar`) abre direto numa
página. as mudanças esperam num rascunho até você apertar **aplicar**.

## o editor da barra

a barra desenhada como ela vai ficar. toque num módulo pra mudar ele no
inspetor do lado; arraste pra outro lugar, ou pro catálogo pra tirar; arraste
novos do catálogo.

| módulo | o que é | opções |
|---|---|---|
| `logo` | abre o launcher | glifo, o que o clique abre, fundo, cantos |
| `workspaces` | os workspaces do niri | pontos ou números |
| `clock` | horas sobre minutos; abre o dash | 24 ou 12 horas, bun/pílula/arredondado/quadrado, fundo |
| `tray` | apps em segundo plano | |
| `battery` | uma pílula que enche | mostrar a porcentagem |
| `controls` | central de controle, wi-fi, bluetooth, som, energia | quais botões, numa gaveta ou sempre à mostra |
| `cpu`, `memory`, `temperature` | uso e calor | mostrar o valor, fundo, cantos |
| `brightness`, `volume` | role pra mudar | mostrar o valor, fundo, cantos |
| `idle` | mantém a tela acordada | fundo, cantos |
| `media` | tocar e pausar | fundo, cantos |
| `notifications`, `power` | atalhos | fundo, cantos |

**presets** troca a barra inteira de uma vez: padrão, mínima, monitor, música
e foco. a barra muda ao aplicar, sem reiniciar.

## octopus.toml

```toml
language = "auto"          # auto segue o LANG; "pt" ou "en"

[look]
scheme = "tonal-spot"      # a paleta do matugen: expressive, vibrant, fidelity, content,
                           # neutral, monochrome, rainbow, fruit-salad
contrast = 0.0             # de -1 a 1

[bar]
position = "left"          # ou "right"
width = 52
spacing = 4                # entre os módulos
style = "floating"         # ou "flush": encostada na borda, cantos côncavos
top = ["logo", "workspaces"]
center = ["clock"]
bottom = ["tray", "battery", "controls"]

[bar.modules.clock]        # uma tabela por módulo que você mudou
background = "primary"     # none, tonal, primary, surface
shape = "bun"              # bun (só o relógio), round, soft, square
variant = "24h"            # a escolha do próprio módulo: 12h, numbers, um glifo…
text = false               # o valor embaixo do ícone
action = ""                # o logo: launcher, spotlight ou dash
items = []                 # os botões dos controles

[dash]
page = "overview"          # ou media, weather, wallpapers

[weather]
place = ""                 # uma cidade, ou "lat,lon"; vazio usa o ip
unit = "celsius"           # ou "fahrenheit"

[notifications]
timeout_ms = 5000
max_popups = 3
position = "top-right"     # top-left, bottom-right, bottom-left

[media]
lyrics = true              # no dash
lyrics_offset_ms = 0

[lock]
lyrics = true

[dock]
enabled = false
mode = "smart"             # "fixed", "autohide" ou "smart"
pinned = ["firefox.desktop"]  # clique direito num ícone pra fixar
size = 44
magnify = true

[desktop]
enabled = true
widgets = ["weather", "contributions"]  # também clock, media, calendar
github = ""                # um usuário do github, pro calendário de contribuições
contributions_colors = "github"  # ou "theme", a cor do wallpaper
```

## quando vale

- **barra e tempo das letras**: na hora
- **cores (modo)**: na hora; estilo da paleta e contraste repintam tudo ao aplicar
- **todo o resto**: o octopus reinicia sozinho ao aplicar

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
- **contributions**: o calendário do perfil de `github`, os quadradinhos verdes, com o total do
  ano e a sequência atual. passe o mouse num quadrado pra ver o dia. mostra quantas semanas
  couberem na largura e atualiza a cada meia hora (sem token).
- **clock**: analógico, num cookie.
- **media**: o que está tocando, com os controles.
- **calendar**: o mês inteiro.

eles ficam parados até você organizar: clique direito na área de trabalho e
escolha **organizar widgets** (ou `octopus desktop arrange`). a área de
trabalho sobe por cima das janelas; arraste um widget pra mover, o canto pra
mudar o tamanho, e use a maleta embaixo pra pôr os outros (arraste um pra fora,
ou clique) ou guardar um (solte na maleta, ou no ×). **pronto** ou <kbd>esc</kbd>
termina.

onde eles ficam é guardado em `~/.local/state/octopus/desktop.json`; apague
esse arquivo pra voltar pro canto.
