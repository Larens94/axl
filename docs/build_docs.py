#!/usr/bin/env python3
"""Build the static bilingual AXL documentation portal."""

from __future__ import annotations

import html
import json
import re
from pathlib import Path

import markdown

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parent

PAGES = [
    ("README.md", "welcome", "Benvenuto", "Welcome"),
    ("ax-ecosystem.md", "ax-ecosystem", "Ecosistema AX", "AX ecosystem"),
    ("overview.md", "overview", "Visione e obiettivi", "Vision and goals"),
    ("architecture.md", "architecture", "Architettura", "Architecture"),
    ("compact-syntax.md", "compact-syntax", "Compact Source 2", "Compact Source 2"),
    ("language-guide.md", "language-guide", "Guida al linguaggio", "Language guide"),
    ("agent-runtime.md", "agent-runtime", "Agenti e runtime", "Agents and runtime"),
    ("ax-ir.md", "ax-ir", "AX-IR", "AX-IR"),
    ("security.md", "security", "Sicurezza", "Security"),
    ("toolchain.md", "toolchain", "Toolchain", "Toolchain"),
    ("roadmap.md", "roadmap", "Roadmap", "Roadmap"),
    (
        "platform-demo-analysis.md",
        "platform-demo-analysis",
        "Demo e piattaforme",
        "Demos and platforms",
    ),
    ("development.md", "development", "Sviluppo", "Development"),
    ("glossary.md", "glossary", "Glossario", "Glossary"),
]

GROUPS = [
    ("Inizia", "Get started", ["welcome", "ax-ecosystem", "overview"]),
    ("Linguaggio", "Language", ["compact-syntax", "language-guide", "ax-ir"]),
    (
        "Piattaforma",
        "Platform",
        ["architecture", "agent-runtime", "security", "toolchain"],
    ),
    (
        "Progetto",
        "Project",
        ["roadmap", "platform-demo-analysis", "development", "glossary"],
    ),
]

LABELS = {slug: (it, en) for _, slug, it, en in PAGES}
SOURCE = {slug: filename for filename, slug, _, _ in PAGES}


def strip_language_line(text: str) -> str:
    return re.sub(r"^\[Italiano\].*?\n+", "", text, count=1, flags=re.MULTILINE)


def rewrite_links(text: str, lang: str) -> str:
    def replace(match: re.Match[str]) -> str:
        label, target = match.group(1), match.group(2)
        if target.startswith(("http://", "https://", "#", "mailto:")):
            return match.group(0)
        clean, anchor = (target.split("#", 1) + [""])[:2]
        suffix = f"#{anchor}" if anchor else ""
        name = Path(clean).name
        if name == "SPEC.md":
            return (
                f"[{label}](https://github.com/Larens94/axl/blob/main/SPEC.md{suffix})"
            )
        if name == "SPEC.en.md":
            return f"[{label}](https://github.com/Larens94/axl/blob/main/SPEC.en.md{suffix})"
        for filename, slug, _, _ in PAGES:
            if name == filename:
                href = "./" if slug == "welcome" else f"{slug}.html"
                return f"[{label}]({href}{suffix})"
        return match.group(0)

    return re.sub(r"\[([^\]]+)\]\(([^)]+)\)", replace, text)


def plain_title(markdown_text: str, fallback: str) -> str:
    match = re.search(r"^#\s+(.+)$", markdown_text, re.MULTILINE)
    return re.sub(r"[`*_]", "", match.group(1)) if match else fallback


def extract_toc(rendered: str) -> list[tuple[int, str, str]]:
    result = []
    for level, ident, content in re.findall(
        r'<h([23]) id="([^"]+)">(.*?)</h\1>', rendered, re.DOTALL
    ):
        label = re.sub(r"<[^>]+>", "", content)
        result.append((int(level), ident, html.unescape(label)))
    return result


def nav_html(active: str, lang: str) -> str:
    index = 0 if lang == "it" else 1
    chunks = []
    for group_it, group_en, slugs in GROUPS:
        chunks.append(
            f'<div class="nav-group"><p>{group_it if lang == "it" else group_en}</p>'
        )
        for slug in slugs:
            href = "./" if slug == "welcome" else f"{slug}.html"
            current = ' aria-current="page" class="active"' if slug == active else ""
            chunks.append(
                f'<a href="{href}"{current}>{html.escape(LABELS[slug][index])}</a>'
            )
        chunks.append("</div>")
    return "".join(chunks)


def toc_html(items: list[tuple[int, str, str]], lang: str) -> str:
    if not items:
        return ""
    title = "In questa pagina" if lang == "it" else "On this page"
    links = "".join(
        f'<a class="toc-l{level}" href="#{ident}">{html.escape(label)}</a>'
        for level, ident, label in items[:14]
    )
    return f'<aside class="page-toc"><p>{title}</p>{links}</aside>'


def pager(active: str, lang: str) -> str:
    slugs = [slug for _, slug, _, _ in PAGES]
    position = slugs.index(active)
    index = 0 if lang == "it" else 1
    parts = []
    if position:
        slug = slugs[position - 1]
        href = "./" if slug == "welcome" else f"{slug}.html"
        caption = "Precedente" if lang == "it" else "Previous"
        parts.append(
            f'<a class="prev" href="{href}"><small>← {caption}</small><b>{html.escape(LABELS[slug][index])}</b></a>'
        )
    if position < len(slugs) - 1:
        slug = slugs[position + 1]
        caption = "Successivo" if lang == "it" else "Next"
        parts.append(
            f'<a class="next" href="{slug}.html"><small>{caption} →</small><b>{html.escape(LABELS[slug][index])}</b></a>'
        )
    aria = "Paginazione" if lang == "it" else "Pagination"
    return f'<nav class="pager" aria-label="{aria}">' + "".join(parts) + "</nav>"


def welcome_cards(lang: str) -> str:
    cards = [
        (
            "compact-syntax",
            "Sintassi compatta",
            "Compact syntax",
            "Opcode, frame e formule RPN.",
            "Opcodes, frames, and RPN formulas.",
        ),
        (
            "architecture",
            "Architettura",
            "Architecture",
            "Dal sorgente ai runtime e ai bridge.",
            "From source to runtimes and bridges.",
        ),
        (
            "agent-runtime",
            "Agenti e memoria",
            "Agents and memory",
            "Capability, workflow, policy e AM.",
            "Capabilities, workflows, policy, and AM.",
        ),
        (
            "roadmap",
            "Roadmap",
            "Roadmap",
            "Il percorso verso Rust, WASM e app native.",
            "The path to Rust, WASM, and native apps.",
        ),
    ]
    i = 0 if lang == "it" else 1
    rows = []
    for slug, it_title, en_title, it_body, en_body in cards:
        rows.append(
            f'<a class="start-card" href="{slug}.html"><span>{"0" + str(len(rows) + 1)}</span><b>{html.escape((it_title, en_title)[i])}</b><p>{html.escape((it_body, en_body)[i])}</p><em>→</em></a>'
        )
    title = "Da dove iniziare" if lang == "it" else "Choose where to start"
    return f'<section class="start-grid"><h2>{title}</h2><div>{"".join(rows)}</div></section>'


def page_template(
    *, lang: str, slug: str, title: str, content: str, toc: list[tuple[int, str, str]]
) -> str:
    italian = lang == "it"
    prefix = "" if italian else "../"
    if italian:
        switch_href = "en/" if slug == "welcome" else f"en/{slug}.html"
    else:
        switch_href = "../" if slug == "welcome" else f"../{slug}.html"
    switch_label = "English" if italian else "Italiano"
    search_label = "Cerca nella documentazione" if italian else "Search documentation"
    github_docs = "https://github.com/Larens94/axl/tree/main/docs" + (
        "/en" if not italian else ""
    )
    breadcrumb = "Documentazione" if italian else "Documentation"
    edit_label = "Modifica questa pagina" if italian else "Edit this page"
    source_path = f"docs/{'en/' if not italian else ''}{SOURCE[slug]}"
    cards = welcome_cards(lang) if slug == "welcome" else ""
    return f'''<!doctype html>
<html lang="{lang}">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="theme-color" content="#ffffff">
  <meta name="description" content="AXL {breadcrumb}: {html.escape(title)}">
  <title>{html.escape(title)} · AXL {breadcrumb}</title>
  <link rel="stylesheet" href="{prefix}docs.css">
  <script defer src="{prefix}docs.js"></script>
</head>
<body data-lang="{lang}" data-page="{slug}">
  <a class="skip" href="#content">{"Vai al contenuto" if italian else "Skip to content"}</a>
  <header class="docs-header">
    <a class="brand" href="./"><span>A</span><b>AXL</b><i>{breadcrumb}</i></a>
    <nav class="product-nav" aria-label="{breadcrumb}">
      <a class="active" href="./">{breadcrumb}</a>
      <a href="https://github.com/Larens94/axl">GitHub</a>
    </nav>
    <button class="search-trigger" type="button" aria-label="{search_label}" aria-haspopup="dialog" aria-controls="docs-search" data-open-search><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m21 21-4.35-4.35m2.35-5.65a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z"/></svg><span>{search_label}</span><kbd>⌘ K</kbd></button>
    <a class="language" href="{switch_href}" hreflang="{"en" if italian else "it"}">{switch_label}</a>
    <button class="menu-trigger" type="button" aria-label="Menu" aria-expanded="false" data-menu>☰</button>
  </header>
  <div class="docs-shell">
    <aside class="sidebar" data-sidebar>
      <div class="sidebar-scroll">{nav_html(slug, lang)}</div>
      <a class="sidebar-github" href="{github_docs}">GitHub <span>↗</span></a>
    </aside>
    <main id="content" class="content-wrap">
      <div class="article-column">
        <p class="breadcrumb"><a href="./">{breadcrumb}</a><span>/</span>{html.escape(title)}</p>
        <article class="prose">{content}</article>
        {cards}
        <div class="article-meta"><a href="https://github.com/Larens94/axl/edit/main/{source_path}">{edit_label} ↗</a></div>
        {pager(slug, lang)}
      </div>
      {toc_html(toc, lang)}
    </main>
  </div>
  <dialog id="docs-search" class="search-dialog" aria-label="{search_label}" data-search-dialog>
    <form method="dialog"><button aria-label="{"Chiudi" if italian else "Close"}">×</button></form>
    <div class="search-box"><svg aria-hidden="true" viewBox="0 0 24 24"><path d="m21 21-4.35-4.35m2.35-5.65a8 8 0 1 1-16 0 8 8 0 0 1 16 0Z"/></svg><input type="search" aria-label="{search_label}" placeholder="{search_label}" autocomplete="off" data-search-input></div>
    <div class="search-results" data-search-results></div>
  </dialog>
</body>
</html>'''


def build_language(lang: str) -> list[dict[str, str]]:
    source_root = ROOT if lang == "it" else ROOT / "en"
    output_root = ROOT if lang == "it" else ROOT / "en"
    output_root.mkdir(parents=True, exist_ok=True)
    search_index = []
    extension_configs = {
        "codehilite": {"guess_lang": False},
        "toc": {"permalink": False},
    }
    extensions = ["extra", "sane_lists", "toc", "codehilite"]
    for filename, slug, it_label, en_label in PAGES:
        source = source_root / filename
        if not source.exists():
            print(f"skip missing {source}")
            continue
        raw = rewrite_links(
            strip_language_line(source.read_text(encoding="utf-8")), lang
        )
        label = it_label if lang == "it" else en_label
        title = plain_title(raw, label)
        rendered = markdown.markdown(
            raw, extensions=extensions, extension_configs=extension_configs
        )
        toc = extract_toc(rendered)
        output = output_root / ("index.html" if slug == "welcome" else f"{slug}.html")
        output.write_text(
            page_template(lang=lang, slug=slug, title=title, content=rendered, toc=toc),
            encoding="utf-8",
        )
        clean = re.sub(r"[`#*_>\[\]()|]", " ", raw)
        clean = re.sub(r"\s+", " ", clean).strip()
        search_index.append(
            {
                "title": title,
                "label": label,
                "url": "./" if slug == "welcome" else f"{slug}.html",
                "text": clean[:12000],
            }
        )
    (output_root / "search-index.json").write_text(
        json.dumps(search_index, ensure_ascii=False), encoding="utf-8"
    )
    return search_index


def main() -> None:
    it = build_language("it")
    en = build_language("en")
    print(f"built {len(it)} Italian and {len(en)} English pages")


if __name__ == "__main__":
    main()
