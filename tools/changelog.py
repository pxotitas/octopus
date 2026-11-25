"""The changelog, read from main's conventional commits and grouped by day."""

import html
import re
import subprocess
from datetime import date

from layout import REPO

CONVENTIONAL = re.compile(r"^(\w+)(?:\(([^)]+)\))?!?:\s*(.+)$")

KINDS = {
    "en": {"feat": "new", "fix": "fix", "perf": "faster", "refactor": "refactor", "style": "style",
           "revert": "revert", "docs": "docs", "chore": "chore", "other": "other"},
    "pt": {"feat": "novo", "fix": "correção", "perf": "mais rápido", "refactor": "refactor", "style": "estilo",
           "revert": "revertido", "docs": "docs", "chore": "manutenção", "other": "outro"},
}

TEXT = {
    "en": {"title": "changelog", "lead": "everything that changed in the shell, newest first, straight from the commits on <code>main</code>.",
           "months": "january february march april may june july august september october november december"},
    "pt": {"title": "changelog", "lead": "tudo que mudou no shell, do mais novo pro mais velho, direto dos commits da <code>main</code>. os commits são escritos em inglês.",
           "months": "janeiro fevereiro março abril maio junho julho agosto setembro outubro novembro dezembro"},
}


def commits(repo):
    log = subprocess.run(
        ["git", "-C", repo, "log", "main", "--no-merges", "--date=short", "--format=%h%x1f%ad%x1f%s"],
        capture_output=True, text=True, check=True,
    ).stdout
    for line in log.splitlines():
        sha, day, subject = line.split("\x1f", 2)
        m = CONVENTIONAL.match(subject)
        kind, scope, text = (m.group(1), m.group(2), m.group(3)) if m else ("other", None, subject)
        yield sha, day, kind if kind in KINDS["en"] else "other", scope, text


def render(lang, repo):
    t, kinds = TEXT[lang], KINDS[lang]
    months = t["months"].split()
    days = {}
    for entry in commits(repo):
        days.setdefault(entry[1], []).append(entry)

    out = [f'<section class="page-head"><h1>{t["title"]}</h1><p class="lead">{t["lead"]}</p></section>', '<div class="changelog">']
    for day, entries in days.items():
        d = date.fromisoformat(day)
        label = f"{d.day} {months[d.month - 1]} {d.year}" if lang == "pt" else f"{months[d.month - 1]} {d.day}, {d.year}"
        out.append(f'<section class="day"><h2 id="{day}"><time datetime="{day}">{label}</time></h2><ul>')
        for sha, _, kind, scope, text in entries:
            scope_tag = f'<span class="scope">{html.escape(scope)}</span>' if scope else ""
            out.append(
                f'<li><span class="kind k-{kind}">{kinds[kind]}</span>{scope_tag}'
                f'<span class="text">{html.escape(text[0].lower() + text[1:])}</span>'
                f'<a class="sha" href="{REPO}/commit/{sha}">{sha}</a></li>'
            )
        out.append("</ul></section>")
    out.append("</div>")
    return "\n".join(out)
