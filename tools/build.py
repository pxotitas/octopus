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
from layout import LANGS, REPO, page

ROOT = pathlib.Path(__file__).resolve().parent.parent

# same order and meaning in both languages: (file in docs/, page name)
DOCS = {
    "en": ("eng", [("README.md", "index"), ("structure.md", "structure"), ("shortcuts.md", "shortcuts"),
                   ("configuration.md", "configuration"), ("theme.md", "theme"), ("troubleshooting.md", "troubleshooting")]),
    "pt": ("pt-br", [("README.md", "index"), ("estrutura.md", "estrutura"), ("atalhos.md", "atalhos"),
                     ("configuracao.md", "configuracao"), ("tema.md", "tema"), ("problemas.md", "problemas")]),
}

TEXT = {
    "en": {"overview": "overview", "edit": "edit this page on github", "docs": "docs",
           "missing": "this page doesn't exist", "back": "back to the start"},
    "pt": {"overview": "visão geral", "edit": "editar essa página no github", "docs": "docs",
           "missing": "essa página não existe", "back": "voltar pro início"},
}


def show(ref):
    return subprocess.run(["git", "-C", str(ROOT), "show", ref], capture_output=True, text=True, check=True).stdout


def write(path, content):
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
        write(out, page(lang, out, doc_title, content, twin, section="docs"))


def fragment(lang, name, path):
    body = (ROOT / "pages" / lang / f"{name}.html").read_text()
    return body.replace("{{rel}}", "../" * path.count("/"))


def main():
    for lang in LANGS:
        root = LANGS[lang]["root"]
        twin_root = LANGS[other(lang)]["root"]

        path = f"{root}index.html"
        write(path, page(lang, path, "", fragment(lang, "home", path), f"{twin_root}index.html", scripts=("shapes.js", "motion.js", "weather-demo.js")))

        path = f"{root}media/index.html"
        title = LANGS[lang]["nav"]["media"]
        write(path, page(lang, path, title, media.render(lang, "../" * path.count("/")), f"{twin_root}media/index.html",
                         section="media", scripts=("media.js",)))

        path = f"{root}changelog/index.html"
        write(path, page(lang, path, "changelog", changelog.render(lang, str(ROOT)), f"{twin_root}changelog/index.html",
                         section="changelog"))

        docs(lang)

    t = TEXT["en"]
    missing = f'<section class="page-head"><h1>404</h1><p class="lead">{t["missing"]}. <a href="/octopus/">{t["back"]}</a>.</p></section>'
    write("404.html", page("en", "404.html", "404", missing, "pt/index.html", base="/octopus/"))
    print("built")


if __name__ == "__main__":
    main()
