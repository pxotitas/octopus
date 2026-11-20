# Octopus roadmap

## 0. Foundation
- [x] Create local repository directory.
- [x] Record design direction and interaction rules.
- [x] Record Waybar module requirements.
- [x] Confirm workspace display: show all existing workspaces and emphasize the focused one.
- [x] Verify native Waybar Niri workspace/window modules; no helper scripts needed for these basics.
- [x] Identify the official Ewwii project and current published docs (0.10.0); inspect window/widget-control capabilities.
- [x] Identify a simple shared-window content approach: one Ewwii `Stack`, switch its selected child.
- [x] Confirm Nucleus Shell is archived; retain it as visual inspiration only.
- [x] Define the minimal Waybar/Ewwii hand-off contract; Ewwii window and anchoring implementation is user-owned.

## 1. Waybar
- [x] Review Waybar and Ewwii package sources/metadata; no packages installed.
- [x] Choose Open-Meteo for weather; fixed manual location is Campo Grande, MS.
- [x] Confirm Waybar has a native MPRIS module; no custom player bridge needed for the basic bar item.
- [ ] Test MPRIS behaviour for paused/stopped states and multiple available players.
- [x] Implement left, centre, and right groups in Waybar JSONC/CSS.
- [x] Define the Waybar-to-Ewwii CLI hand-off contract; Ewwii implementation is user-owned.
- [ ] Test click and right-click behaviour after the user-owned Ewwii window exists.
- [ ] Test CAVA lazy start/stop and cleanup.
- [ ] Install Waybar and preview manually; do not enable it at login yet.

## 2. Ewwii sections (user-owned; outside the Waybar implementation)
- [ ] Shared window and contextual anchoring.
- [ ] Main, clock, weather, media, and power sections.
- [ ] Network, Bluetooth, audio, microphone, and clipboard panels.
- [ ] Media controls and CAVA lifecycle.

## 3. Shell integration
- [x] Matugen token template and Waybar output path.
- [ ] Walker integration.
- [ ] Dunst configuration and notification icon/avatar capability test.
- [ ] Niri animation tuning.
- [ ] Hyprlock and Hypridle integration.
- [ ] Awww wallpaper integration.
- [ ] Clipse integration.

## 4. Polish and release
- [ ] Keyboard navigation and accessibility pass.
- [ ] Reduced-motion behaviour.
- [ ] Resource/lifecycle checks.
- [ ] Documentation and screenshots.
- [ ] Install/uninstall workflow only after the shell is stable.
