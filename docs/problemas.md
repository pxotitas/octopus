# quando algo quebra

**Painéis não abrem / `octopus: … is the daemon running?`**
O daemon caiu. Rodar `GSK_RENDERER=gl octopus &` num terminal mostra o erro.

**A barra sumiu ou ficou estranha**
`octopus reload` regera o config. Se a Waybar morreu: `waybar &`.
Erros de CSS aparecem rodando `waybar` num terminal.

**A setinha da bandeja não aparece**
Falta o `~/.cache/octopus/tray-arrow.so`. Rodar o `install.sh` de novo
(precisa de `gcc` e `gtk3`). Sem ele a barra usa o botão de texto.

**Tela travada no bloqueio**
Ctrl+Alt+F2, entrar, e `pkill -x octopus`. O niri continua bloqueado até
um bloqueio novo destravar: `niri msg action spawn -- octopus` e depois
`octopus lock` de novo, ou sair da sessão.

**Cores não mudam**
Ver se o Matuwall tem `on_apply = ["octopus-theme {path}"]` e se
`~/.local/bin/octopus-theme` existe.

**Recompilar**
```
cd ~/.config/octopus && ./install.sh
octopus restart
```

**Memória**
`grep RssAnon /proc/$(pgrep -x octopus)/status` mostra o que é do daemon
(sem as bibliotecas compartilhadas). Fica em torno de 45 MB parado.
