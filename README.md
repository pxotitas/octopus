# octopus · website

the project's site, for github pages: <https://pxotitas.github.io/octopus/>.
plain html, css and a little js, in english and portuguese.

## build

the docs come from the `eng` and `pt-br` branches and the changelog from the
commits on `main`, so build after any of them changes:

```sh
python3 tools/build.py
```

it needs only python and git, and writes the pages into this branch's root.

## where things are

| path | what |
|---|---|
| `tools/build.py` | builds every page |
| `tools/layout.py` | the head, navbar and footer every page shares |
| `tools/markdown.py` | the bit of markdown the docs use |
| `tools/changelog.py` | the changelog, from main's commits |
| `tools/media.py` | the media page: video, clips, screenshots, logo |
| `pages/en`, `pages/pt` | the home page, by hand |
| `assets/` | logo, banner, screenshots, clips and the full video |
| `style.css` | every page's style |
| `shapes.js` | the clock shapes on the home page |
| `media.js` | clips that play on screen, screenshots full size |
| `motion.js` | stops autoplay when the system asks for less motion |

everything else (`index.html`, `docs/`, `media/`, `changelog/`, `pt/`,
`404.html`) is generated: edit the sources above, not those.
