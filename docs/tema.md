# tema

1. o matuwall (ou a aba de wallpapers do dash) aplica a imagem com o `awww`.
2. depois roda `octopus-theme <imagem>` (= `scripts/theme.sh`).
3. o `theme.sh` lê o modo em `~/.local/state/octopus/mode`
   (`light`, `dark` ou `auto`; no auto pergunta pro `octopus luma`).
4. o matugen gera as cores com `matugen/config.toml`:
   companion, waybar, fuzzel, wofi e matuwall.
5. depois roda o teu `~/.config/matugen/config.toml`, pro resto do sistema.

o companion e a waybar percebem o `colors.css` novo e recarregam sozinhos.

pra trocar o modo: tile de tema no control center, ou configurações → aparência.
