# theme

1. matuwall (or the dash's wallpapers tab) sets the image with `awww`.
2. then it runs `octopus-theme <image>` (= `scripts/theme.sh`).
3. `theme.sh` reads the mode from `~/.local/state/octopus/mode`
   (`light`, `dark` or `auto`; on auto it asks `octopus luma`).
4. matugen makes the colors with `matugen/config.toml`:
   companion, waybar, fuzzel, wofi and matuwall.
5. then it runs your `~/.config/matugen/config.toml`, for the rest of the system.

the companion and waybar notice the new `colors.css` and reload on their own.

to switch the mode: the theme tile in the control center, or settings → appearance.
