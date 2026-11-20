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
- [ ] Prototype Ewwii shared-window content switching and Wayland anchoring before committing to the UI transport.

## 1. Waybar
- [x] Review Waybar and Ewwii package sources/metadata; no packages installed.
- [ ] Confirm weather provider and automatic-location behaviour.
- [x] Confirm Waybar has a native MPRIS module; no custom player bridge needed for the basic bar item.
- [ ] Test MPRIS behaviour for paused/stopped states and multiple available players.
- [ ] Implement left, centre, and right groups.
- [ ] Implement the shared section-window request protocol.
- [ ] Test click and right-click behaviour.
- [ ] Test CAVA lazy start/stop and cleanup.
- [ ] Preview without enabling it at login.

## 2. Octopus sections
- [ ] Shared window controller.
- [ ] Main section.
- [ ] Clock and calendar section.
- [ ] Weather section.
- [ ] Media section and CAVA lifecycle.
- [ ] Power section.
- [ ] Network, Bluetooth, audio, and clipboard panels.

## 3. Shell integration
- [ ] Matugen token pipeline.
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
