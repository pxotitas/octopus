"""The shell every page shares: head, navbar and footer, in both languages."""

import hashlib
import pathlib

ROOT = pathlib.Path(__file__).resolve().parent.parent

SITE = "https://pxotitas.github.io/octopus/"
REPO = "https://github.com/pxotitas/octopus"

LANGS = {
    "en": {
        "root": "",
        "html": "en",
        "tagline": "a cozy m3 shell for niri",
        "description": "a cozy material 3 shell for niri: a vertical waybar and a rust + gtk4 companion.",
        "nav": {"features": "some features", "docs": "docs", "media": "media", "changelog": "changelog"},
        "switch": "português",
        "footer": "gpl-3.0-or-later · made by {me}, with parts from {impasto} and {velatura}.",
    },
    "pt": {
        "root": "pt/",
        "html": "pt-BR",
        "tagline": "um shell m3 cozy pro niri",
        "description": "um shell material 3 cozy pro niri: uma waybar vertical e um companion em rust + gtk4.",
        "nav": {"features": "algumas features", "docs": "docs", "media": "mídia", "changelog": "changelog"},
        "switch": "english",
        "footer": "gpl-3.0-or-later · feito por {me}, com partes do {impasto} e do {velatura}.",
    },
}

SECTIONS = {"features": "features/", "docs": "docs/", "media": "media/", "changelog": "changelog/"}


def versioned(name):
    """A content hash in the url, so browsers drop a cached file as soon as it changes."""
    digest = hashlib.sha1((ROOT / name).read_bytes()).hexdigest()[:8]
    return f"{name}?v={digest}"


def page(lang, path, title, body, other, section=None, scripts=(), base=None, styles=()):
    """path: where the page is written, from the site root (e.g. "pt/docs/tema.html").
    other: the same page in the other language, from the site root.
    base: an absolute prefix for pages served from any path, like the 404."""
    t = LANGS[lang]
    rel = base or "../" * path.count("/")
    home = rel + t["root"]
    full_title = f"{title} · octopus" if title else f"octopus · {t['tagline']}"
    url = SITE + path.removesuffix("index.html")

    links = []
    for key, sub in SECTIONS.items():
        current = ' aria-current="page"' if key == section else ""
        links.append(f'<a href="{home}{sub}"{current}>{t["nav"][key]}</a>')
    links.append(f'<a href="{REPO}">github</a>')
    links.append(f'<a class="switch" href="{rel}{other.removesuffix("index.html")}" hreflang="{"pt-BR" if lang == "en" else "en"}">{t["switch"]}</a>')

    footer = t["footer"].format(
        me=f'<a href="https://github.com/pxotitas">pxotitas</a>',
        impasto='<a href="https://github.com/andreumassanet/impasto">impasto</a>',
        velatura='<a href="https://github.com/pxdritz1/velatura">velatura</a>',
    )
    sheets = "".join(f'\n  <link rel="stylesheet" href="{rel}{versioned(s)}">' for s in styles)
    tags = "".join(f'\n<script src="{rel}{versioned(s)}" defer></script>' for s in (*scripts, "m3.js"))

    return f"""<!DOCTYPE html>
<html lang="{t['html']}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>{full_title}</title>
  <meta name="description" content="{t['description']}">
  <meta name="theme-color" content="#fdfbef" media="(prefers-color-scheme: light)">
  <meta name="theme-color" content="#14140c" media="(prefers-color-scheme: dark)">
  <meta property="og:type" content="website">
  <meta property="og:site_name" content="octopus">
  <meta property="og:title" content="{full_title}">
  <meta property="og:description" content="{t['description']}">
  <meta property="og:url" content="{url}">
  <meta property="og:image" content="{SITE}assets/banner.png">
  <meta property="og:image:width" content="1280">
  <meta property="og:image:height" content="640">
  <meta name="twitter:card" content="summary_large_image">
  <link rel="icon" href="{rel}assets/logo.svg" type="image/svg+xml">
  <link rel="apple-touch-icon" href="{rel}assets/apple-touch-icon.png">
  <link rel="alternate" hreflang="{'pt-BR' if lang == 'en' else 'en'}" href="{SITE}{other.removesuffix('index.html')}">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
  <link href="https://fonts.googleapis.com/css2?family=Rubik:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20..48,400..600,0..1,0&display=block" rel="stylesheet">
  <link rel="stylesheet" href="{rel}{versioned('style.css')}">{sheets}
</head>
<body>
<header class="top">
  <nav>
    <a class="brand" href="{home}">
      <picture>
        <source srcset="{rel}assets/logo-dark.svg" media="(prefers-color-scheme: dark)">
        <img src="{rel}assets/logo.svg" alt="" width="36" height="36">
      </picture>
      <span>octopus</span>
    </a>
    <div class="links">
      {chr(10).join("      " + l for l in links).strip()}
    </div>
  </nav>
</header>
<main>
{body}
</main>
<footer>
  <p>{footer}</p>
</footer>{tags}
</body>
</html>
"""
