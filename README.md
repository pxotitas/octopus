<p align="center">
  <img src="assets/logo.svg" width="128" alt="octopus logo">
</p>

<h1 align="center">octopus</h1>

<p align="center">
  <a href="https://pxotitas.github.io/octopus/">site</a> ·
  <a href="https://pxotitas.github.io/octopus/docs/">docs</a> ·
  <a href="https://pxotitas.github.io/octopus/media/">screenshots and videos</a> ·
  <a href="https://pxotitas.github.io/octopus/changelog/">changelog</a>
</p>

<p align="center">
  <a href="https://github.com/pxotitas/octopus"><img src="https://img.shields.io/badge/github-pxotitas%2Foctopus-blue?logo=github" alt="github repo"></a>
  <a href="https://github.com/pxotitas/octopus/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-gpl--3.0--or--later-blue" alt="license"></a>
  <a href="https://github.com/pxotitas/octopus"><img src="https://img.shields.io/badge/made%20with-rust-orange?logo=rust" alt="rust"></a>
  <a href="https://github.com/pxotitas/octopus/stargazers"><img src="https://img.shields.io/github/stars/pxotitas/octopus?label=stars&logo=github&color=yellow&style=flat" alt="github stars"></a>
</p>

a cozy m3 shell for niri: a minimal vertical bar on waybar, with everything
bigger (control center, dash, menus, notifications) in a companion written
in rust + gtk4.

<img src="https://raw.githubusercontent.com/pxotitas/octopus/eng/screenshots/clips/dash.webp" alt="the dash, with the music and its lyrics" width="100%">

this branch is the shell itself: the companion, the scripts, the waybar and
the niri config. the docs, screenshots and clips live in branches of their own.

| branch | what |
|---|---|
| [`main`](https://github.com/pxotitas/octopus/tree/main) | the code |
| [`eng`](https://github.com/pxotitas/octopus/tree/eng) | readme, docs, screenshots and clips in english |
| [`pt-br`](https://github.com/pxotitas/octopus/tree/pt-br) | the same, in portuguese |
| [`website`](https://github.com/pxotitas/octopus/tree/website) | the [site](https://pxotitas.github.io/octopus/) |

## install

```sh
git clone https://github.com/pxotitas/octopus ~/.config/octopus
~/.config/octopus/install.sh
```

then, at the end of `~/.config/niri/config.kdl`:

```kdl
include "~/.config/octopus/niri.kdl"
```

the dependencies and everything else are in the [docs](https://pxotitas.github.io/octopus/docs/).

## license

gpl-3.0-or-later. some parts come from [impasto](https://github.com/andreumassanet/impasto)
and [velatura](https://github.com/pxdritz1/velatura); see [`notice.md`](NOTICE.md).
