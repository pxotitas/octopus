# atalhos e comandos

## niri (`niri.kdl`)

| atalho | faz |
|---|---|
| Mod+Space, XF86LaunchB | spotlight |
| Mod+D | rofi (apps) |
| Mod+Shift+D | fuzzel (vem do teu config.kdl, não do octopus) |
| Mod+C | central de controle |
| Mod+A | dash |
| Mod+Shift+V | clipboard |
| Mod+Period | emoji |
| Mod+Shift+E | energia |
| Mod+Comma | configurações |
| Mod+L | bloquear |
| Mod+Shift+W | matuwall |

## spotlight

- digitar: apps e as ações deles (ex. "nova janela privativa")
- uma conta (`2*(3+4)`, `sqrt 16`, `1,5 x 2`): calculadora, enter copia
- `/nome`: arquivos na pasta pessoal (usa o `fd`)
- `!quando o quê`: um lembrete (`!10 min chá`, `!alarme 6:45`)
- `>`: bloquear, suspender, sair, reiniciar, desligar

## clima

a linha no topo da aba de clima (depois da setinha) aceita perguntas, em
português ou inglês:

- `amanhã`, `ontem`, `hoje`, `agora`, `depois de amanhã`, `anteontem`
- um dia da semana: `quinta`, `próxima quinta`, `sexta passada`
- dias a partir de hoje: `+3`, `-2`, `em 4 dias`, `3 dias atrás`
- uma data: `28/11`, `2026-12-01`, `30 de novembro`
- `semana`, `próxima semana`, `semana passada`, `fim de semana`

vai de uma semana atrás até duas pra frente. esc volta pro agora.

## octopus files

| teclas | faz |
|---|---|
| ctrl 1 / 2 / 3 | ícones, lista, colunas |
| espaço | olhada rápida |
| ctrl i | informações |
| f2 | renomear |
| ctrl shift n | nova pasta |
| ctrl t, ctrl w, ctrl n | nova aba, fechar aba, nova janela |
| ctrl l | digitar um caminho |
| ctrl f | buscar (enter procura em todas as pastas dentro dessa) |
| ctrl shift d | marcar essa pasta |
| ctrl h | mostrar ocultos |
| ctrl z | desfazer |
| delete, shift delete | pra lixeira, apagar de vez |
| alt ← → ↑, backspace | voltar, avançar, subir |
| ctrl alt t | um terminal aqui |

## comandos

```
octopus                    o daemon (o niri inicia sozinho)
octopus toggle <painel>    control-center, dash, wifi, bluetooth, audio,
                           tray, clipboard, emoji, power, launcher
octopus open wallpapers    dash na aba de wallpapers
octopus open weather [p]   a aba de clima, perguntando p (amanhã, semana…)
octopus settings           janela de configurações
octopus files [pasta]      octopus files
octopus remind <quando o quê>  um lembrete: 10 min chá, 18h pão, alarme 6:45
octopus reminders          o app de lembretes
octopus caffeine [on|off]  mantém a tela acordada (sem on/off, alterna)
octopus dnd                não perturbe
octopus desktop arrange    organizar os widgets da área de trabalho
octopus lock               bloquear
octopus close              fecha qualquer painel
octopus reload             regera o config da Waybar
octopus restart            reinicia o daemon (o idioma e a dock precisam)
octopus luma <imagem>      diz se a imagem é dark ou light
```
