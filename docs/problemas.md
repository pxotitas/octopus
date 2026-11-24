# quando algo quebra

**painéis não abrem / `octopus: … is the daemon running?`**
o daemon caiu. rodar `GSK_RENDERER=gl octopus &` num terminal mostra o erro.

**a barra sumiu ou ficou estranha**
`octopus reload` regera o config. se a waybar morreu: `waybar &`.
erros de css aparecem rodando `waybar` num terminal.

**a setinha da bandeja não aparece**
falta o `~/.cache/octopus/tray-arrow.so`. rodar o `install.sh` de novo
(precisa de `gcc` e `gtk3`). sem ele a barra usa o botão de texto.

**tela travada no bloqueio**
Ctrl+Alt+F2, entrar, e `pkill -x octopus`. o niri continua bloqueado até
um bloqueio novo destravar: `niri msg action spawn -- octopus` e depois
`octopus lock` de novo, ou sair da sessão.

**cores não mudam**
ver se o matuwall tem `on_apply = ["octopus-theme {path}"]` e se
`~/.local/bin/octopus-theme` existe.

**recompilar**
```
cd ~/.config/octopus && ./install.sh
octopus restart
```

**memória**
`grep RssAnon /proc/$(pgrep -x octopus)/status` mostra o que é do daemon
(sem as bibliotecas compartilhadas). fica em torno de 45 mb parado.
