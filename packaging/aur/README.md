# octopus-shell-git

the AUR package. it builds from `main`, so there is nothing to bump when
octopus changes: `pkgver` comes from the commits.

## what it installs

| path | what |
|---|---|
| `/usr/bin/octopus` | the companion |
| `/usr/bin/octopus-theme` | runs `~/.config/octopus/scripts/theme.sh` (matuwall calls it) |
| `/usr/lib/octopus/tray-arrow.so` | waybar's tray button |
| `/usr/share/octopus/` | scripts, matugen templates, assets, the default `octopus.toml` and `niri.kdl` |
| `/usr/share/applications/` | octopus files and reminders |

`octopus setup` copies `/usr/share/octopus` into `~/.config/octopus` and links
`~/.config/waybar` to it. on every start after an update, octopus copies the
managed files again (scripts, templates, assets, styles); `octopus.toml` and
`niri.kdl` are copied only once, then they're the user's. a clone of the repo
in `~/.config/octopus` is never touched.

## when it needs a change

- a new runtime dependency: add it to `depends` or `optdepends`
- a new file the shell reads from `~/.config/octopus`: add it to `package()`
  and to `MANAGED` or `SEEDED` in `companion/src/setup.rs`

then bump `pkgrel`, run `makepkg --printsrcinfo > .SRCINFO`, and push both
files to the AUR repo.

## publishing it the first time

```sh
git clone ssh://aur@aur.archlinux.org/octopus-shell-git.git
cp PKGBUILD octopus-shell-git.install octopus-shell-git/
cd octopus-shell-git
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD octopus-shell-git.install .SRCINFO
git commit -m "initial import"
git push
```

it needs an AUR account with your ssh key.
