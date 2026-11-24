# tema

1. O Matuwall (ou a aba de wallpapers do dash) aplica a imagem com o `awww`.
2. Depois roda `octopus-theme <imagem>` (= `scripts/theme.sh`).
3. O `theme.sh` lê o modo em `~/.local/state/octopus/mode`
   (`light`, `dark` ou `auto`; no auto pergunta pro `octopus luma`).
4. O matugen gera as cores com `matugen/config.toml`:
   companion, Waybar, fuzzel, wofi e Matuwall.
5. Depois roda o teu `~/.config/matugen/config.toml`, pro resto do sistema.

O companion e a Waybar percebem o `colors.css` novo e recarregam sozinhos.

Pra trocar o modo: tile de tema no control center, ou configurações → aparência.
