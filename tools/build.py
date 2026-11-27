#!/usr/bin/env python3
"""Builds the site into the branch root: docs from the eng and pt-br branches,
the changelog from main, and the hand-written pages in pages/.

    python3 tools/build.py
"""

import pathlib
import re
import subprocess

import changelog
import markdown
import media
from layout import LANGS, REPO, SITE, page

ROOT = pathlib.Path(__file__).resolve().parent.parent

# same order and meaning in both languages: (file in docs/, page name)
DOCS = {
    "en": ("eng", [("README.md", "index"), ("structure.md", "structure"), ("shortcuts.md", "shortcuts"),
                   ("configuration.md", "configuration"), ("theme.md", "theme"), ("troubleshooting.md", "troubleshooting")]),
    "pt": ("pt-br", [("README.md", "index"), ("estrutura.md", "estrutura"), ("atalhos.md", "atalhos"),
                     ("configuracao.md", "configuracao"), ("tema.md", "tema"), ("problemas.md", "problemas")]),
}

# what each hand-written page is, for search results
ABOUT = {
    "en": {
        "features": "the octopus shell's features in real screenshots: octopus files, reminders, the bar editor, your own modules, desktop widgets, the calendar, colors from the wallpaper, the lock screen and the weather.",
        "media": "screenshots, clips and a video of octopus, a material 3 shell for niri, plus its logo and banner.",
        "changelog": "everything that changed in octopus, day by day, from its commits.",
    },
    "pt": {
        "features": "as features do shell octopus em screenshots de verdade: octopus files, lembretes, o editor da barra, módulos próprios, widgets, a agenda, cores do wallpaper, a tela de bloqueio e o clima.",
        "media": "screenshots, clipes e um vídeo do octopus, um shell material 3 pro niri, mais o logo e o banner.",
        "changelog": "tudo que mudou no octopus, dia a dia, a partir dos commits.",
    },
}

PAGES = []


def summary(body):
    """The first paragraph of a doc, as plain text short enough for a search result."""
    intro = body.split("<h2", 1)[0]
    m = re.search(r"<p>(.*?)</p>", intro, re.S)
    if not m:
        return None
    text = re.sub(r"<br\s*/?>", " ", m.group(1))
    text = re.sub(r"<[^>]+>", "", text)
    text = re.sub(r"\s+", " ", text).strip()
    text = text.replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
    if len(text) > 158:
        text = text[:155].rsplit(" ", 1)[0] + "…"
    return text


TEXT = {
    "en": {"overview": "overview", "edit": "edit this page on github", "docs": "docs",
           "missing": "this page doesn't exist", "back": "back to the start"},
    "pt": {"overview": "visão geral", "edit": "editar essa página no github", "docs": "docs",
           "missing": "essa página não existe", "back": "voltar pro início"},
}


def show(ref):
    return subprocess.run(["git", "-C", str(ROOT), "show", ref], capture_output=True, text=True, check=True).stdout


def write(path, content, listed=True):
    if listed and path.endswith(".html") and path != "404.html":
        PAGES.append(path)
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content)


def other(lang):
    return "pt" if lang == "en" else "en"


def docs(lang):
    branch, files = DOCS[lang]
    prefix = LANGS[lang]["root"] + "docs/"
    names = {f: name for f, name in files}

    def link(href):
        m = re.match(r"([\w-]+\.md)(#.*)?$", href)
        if m and m.group(1) in names:
            name = names[m.group(1)]
            return ("./" if name == "index" else f"{name}.html") + (m.group(2) or "")
        return href

    rendered = []
    for file, name in files:
        body, title, headings = markdown.render(show(f"{branch}:docs/{file}"), link)
        rendered.append((file, name, body, title, headings))

    for index, (file, name, body, title, headings) in enumerate(rendered):
        items = []
        for _, n, _, t, hs in rendered:
            label = TEXT[lang]["overview"] if n == "index" else t
            href = "./" if n == "index" else f"{n}.html"
            if n == name:
                toc = "".join(f'<li><a href="#{hid}">{text}</a></li>' for hid, text in hs)
                toc = f'<ul class="toc">{toc}</ul>' if toc else ""
                items.append(f'<li><a href="{href}" aria-current="page">{label}</a>{toc}</li>')
            else:
                items.append(f'<li><a href="{href}">{label}</a></li>')

        content = f"""<div class="docs">
  <aside class="docs-nav">
    <ul>{"".join(items)}</ul>
  </aside>
  <article class="doc">
    <h1>{title}</h1>
    {body}
    <p class="edit"><a href="{REPO}/blob/{branch}/docs/{file}">{TEXT[lang]["edit"]}</a></p>
  </article>
</div>"""
        out = f"{prefix}{name}.html"
        twin = LANGS[other(lang)]["root"] + "docs/" + DOCS[other(lang)][1][index][1] + ".html"
        doc_title = TEXT[lang]["docs"] if name == "index" else title
        write(out, page(lang, out, doc_title, content, twin, section="docs", description=summary(body)))


def fragment(lang, name, path):
    body = (ROOT / "pages" / lang / f"{name}.html").read_text()
    return body.replace("{{rel}}", "../" * path.count("/"))


def main():
    for lang in LANGS:
        root = LANGS[lang]["root"]
        twin_root = LANGS[other(lang)]["root"]

        path = f"{root}index.html"
        write(path, page(lang, path, "", fragment(lang, "home", path), f"{twin_root}index.html",
                         scripts=("shapes.js", "reveal.js", "home.js", "motion.js"), landing=True))

        path = f"{root}features/index.html"
        title = LANGS[lang]["nav"]["features"]
        write(path, page(lang, path, title, fragment(lang, "features", path), f"{twin_root}features/index.html",
                         section="features", scripts=("reveal.js", "media.js", "weather-demo.js", "motion.js"), styles=("features.css",),
                         description=ABOUT[lang]["features"]))

        path = f"{root}media/index.html"
        title = LANGS[lang]["nav"]["media"]
        write(path, page(lang, path, title, media.render(lang, "../" * path.count("/")), f"{twin_root}media/index.html",
                         section="media", scripts=("media.js",), description=ABOUT[lang]["media"]))

        path = f"{root}changelog/index.html"
        write(path, page(lang, path, "changelog", changelog.render(lang, str(ROOT)), f"{twin_root}changelog/index.html",
                         section="changelog", description=ABOUT[lang]["changelog"]))

        docs(lang)

    t = TEXT["en"]
    missing = f'<section class="page-head"><h1>404</h1><p class="lead">{t["missing"]}. <a href="/octopus/">{t["back"]}</a>.</p></section>'
    write("404.html", page("en", "404.html", "404", missing, "pt/index.html", base="/octopus/", index=False), listed=False)
    sitemap()
    print("built")


def sitemap():
    """Every page, with its twin in the other language, for search engines."""
    def url(path):
        return SITE + path.removesuffix("index.html")

    pt_root = LANGS["pt"]["root"]
    en = [p for p in PAGES if not p.startswith(pt_root)]
    pt = [p for p in PAGES if p.startswith(pt_root)]
    docs_en = [name for _, name in DOCS["en"][1]]
    docs_pt = [name for _, name in DOCS["pt"][1]]

    def twin(path):
        if path.startswith(pt_root):
            rest = path[len(pt_root):]
            if rest.startswith("docs/"):
                name = rest[5:].removesuffix(".html")
                return "docs/" + docs_en[docs_pt.index(name)] + ".html"
            return rest
        if path.startswith("docs/"):
            name = path[5:].removesuffix(".html")
            return pt_root + "docs/" + docs_pt[docs_en.index(name)] + ".html"
        return pt_root + path

    entries = []
    for path in en + pt:
        other = twin(path)
        en_path, pt_path = (path, other) if path in en else (other, path)
        priority = "1.0" if path in ("index.html", pt_root + "index.html") else "0.7"
        entries.append(f"""  <url>
    <loc>{url(path)}</loc>
    <xhtml:link rel="alternate" hreflang="en" href="{url(en_path)}"/>
    <xhtml:link rel="alternate" hreflang="pt-BR" href="{url(pt_path)}"/>
    <priority>{priority}</priority>
  </url>""")
    (ROOT / "sitemap.xml").write_text(
        '<?xml version="1.0" encoding="UTF-8"?>\n'
        '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">\n'
        + "\n".join(entries) + "\n</urlset>\n"
    )


if __name__ == "__main__":
    main()
