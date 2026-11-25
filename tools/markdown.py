"""The small slice of markdown the docs use: headings, paragraphs, lists,
tables, fenced code, quotes, bold, italics, inline code and links."""

import html
import re

INLINE_CODE = re.compile(r"`([^`]+)`")
LINK = re.compile(r"\[([^\]]+)\]\(([^)\s]+)\)")
BOLD = re.compile(r"\*\*(.+?)\*\*")
ITALIC = re.compile(r"(?<![\w*])\*(?!\s)(.+?)(?<!\s)\*(?![\w*])")


def inline(text, link):
    codes = []

    def keep(m):
        codes.append(f"<code>{html.escape(m.group(1))}</code>")
        return f"\0{len(codes) - 1}\0"

    text = INLINE_CODE.sub(keep, text)
    text = html.escape(text, quote=False)
    text = LINK.sub(lambda m: f'<a href="{html.escape(link(m.group(2)))}">{m.group(1)}</a>', text)
    text = BOLD.sub(r"<strong>\1</strong>", text)
    text = ITALIC.sub(r"<em>\1</em>", text)
    return re.sub(r"\0(\d+)\0", lambda m: codes[int(m.group(1))], text)


def slug(text):
    text = re.sub(r"<[^>]+>", "", text).lower()
    return re.sub(r"[^\w]+", "-", text).strip("-")


def cells(row):
    return [c.strip() for c in row.strip().strip("|").split("|")]


def render(source, link=lambda href: href):
    """Returns (html, title, headings) where headings are the h2s as (id, text)."""
    lines = source.splitlines()
    out, headings, title = [], [], ""
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if not stripped:
            i += 1
            continue

        if stripped.startswith("```"):
            lang = stripped[3:].strip()
            body = []
            i += 1
            while i < len(lines) and not lines[i].strip().startswith("```"):
                body.append(lines[i])
                i += 1
            i += 1
            cls = f' class="language-{lang}"' if lang else ""
            out.append(f"<pre><code{cls}>{html.escape(chr(10).join(body))}</code></pre>")
            continue

        heading = re.match(r"(#{1,4})\s+(.*)", stripped)
        if heading:
            level = len(heading.group(1))
            text = inline(heading.group(2), link)
            if level == 1 and not title:
                title = re.sub(r"<[^>]+>", "", text)
                i += 1
                continue
            hid = slug(text)
            if level == 2:
                headings.append((hid, re.sub(r"<[^>]+>", "", text)))
            out.append(f'<h{level} id="{hid}">{text}</h{level}>')
            i += 1
            continue

        if stripped.startswith("|") and i + 1 < len(lines) and re.match(r"^\|?[\s:|-]+\|?$", lines[i + 1].strip()):
            head = cells(stripped)
            i += 2
            rows = []
            while i < len(lines) and lines[i].strip().startswith("|"):
                rows.append(cells(lines[i]))
                i += 1
            th = "".join(f"<th>{inline(c, link)}</th>" for c in head)
            body = "".join("<tr>" + "".join(f"<td>{inline(c, link)}</td>" for c in r) + "</tr>" for r in rows)
            out.append(f'<div class="table"><table><thead><tr>{th}</tr></thead><tbody>{body}</tbody></table></div>')
            continue

        if stripped.startswith(">"):
            quote = []
            while i < len(lines) and lines[i].strip().startswith(">"):
                quote.append(lines[i].strip()[1:].strip())
                i += 1
            out.append(f"<blockquote><p>{inline(' '.join(quote), link)}</p></blockquote>")
            continue

        item = re.match(r"([-*]|\d+\.)\s+(.*)", stripped)
        if item:
            ordered = item.group(1)[0].isdigit()
            items = []
            while i < len(lines):
                cur = lines[i]
                m = re.match(r"\s*([-*]|\d+\.)\s+(.*)", cur)
                if m and not cur.startswith("   "):
                    items.append(m.group(2))
                elif cur.strip() and cur.startswith(" ") and items:
                    items[-1] += " " + cur.strip()
                else:
                    break
                i += 1
            tag = "ol" if ordered else "ul"
            body = "".join(f"<li>{inline(t, link)}</li>" for t in items)
            out.append(f"<{tag}>{body}</{tag}>")
            continue

        para = []
        while i < len(lines):
            cur = lines[i].strip()
            if not cur or cur.startswith(("```", "#", "|", ">")) or re.match(r"([-*]|\d+\.)\s", cur):
                break
            para.append(cur)
            i += 1
        # a bold line on its own opens a question in troubleshooting: keep the break
        text = "<br>".join(inline(p, link) for p in para) if para[0].startswith("**") else inline(" ".join(para), link)
        out.append(f"<p>{text}</p>")

    return "\n".join(out), title, headings
