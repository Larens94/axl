"""Reference AX-UI web renderer for experimental Source 3 programs."""

import html
import re
from pathlib import Path

from .ir import Literal, Program, UiNode, UiView
from .validation import validate


class WebRenderError(ValueError):
    pass


def render_web(program: Program, output: Path) -> None:
    validate(program)
    views = [item for item in program.instructions if isinstance(item, UiView)]
    if len(views) != 1:
        raise WebRenderError("web build requires exactly one UI view")
    if views[0].root.component_id != 1:
        raise WebRenderError("web view root must use app component 1")
    output.mkdir(parents=True, exist_ok=True)
    output.joinpath("index.html").write_text(_document(views[0].root), encoding="utf-8")
    output.joinpath("ax-ui.css").write_text(_CSS, encoding="utf-8")
    output.joinpath("ax-ui.js").write_text(_JS, encoding="utf-8")


def _value(node: UiNode, property_id: int, default=""):
    for item in node.properties:
        if item.property_id == property_id and isinstance(item.value, Literal):
            return item.value.value
    return default


def _safe_url(value) -> str:
    if not isinstance(value, str) or not re.fullmatch(r"https://[A-Za-z0-9./_?&=%:+-]+", value):
        return ""
    return html.escape(value, quote=True)


def _document(root: UiNode) -> str:
    title = html.escape(str(_value(root, 1, "AX-UI")))
    content = "".join(_render_node(child) for child in root.children)
    return f"""<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>{title}</title><link rel="stylesheet" href="ax-ui.css"></head>
<body><header class="ax-nav"><strong>{title}</strong><nav><a href="#home">Home</a><a href="#catalogue">Series</a><a href="#catalogue">Films</a><a href="#catalogue">New &amp; Popular</a></nav><div class="ax-nav-actions"><button data-search aria-label="Search">⌕</button><button class="ax-avatar" aria-label="Profile">AX</button></div></header><main>{content}</main><div class="ax-toast" role="status" hidden></div><script src="ax-ui.js"></script></body></html>
"""


def _render_node(node: UiNode) -> str:
    if node.component_id == 2:
        image = _safe_url(_value(node, 5))
        style = f' style="--ax-hero:url(&quot;{image}&quot;)"' if image else ""
        buttons = "".join(
            f'<button class="{"ax-play" if item.event_id == 1 else "ax-more"}" data-action="{item.action_id}">{"▶ Play" if item.event_id == 1 else "ⓘ More Info"}</button>'
            for item in node.events
        )
        return f'<section id="home" class="ax-hero"{style}><div class="ax-hero-copy"><span>{html.escape(str(_value(node, 2)))}</span><h1>{html.escape(str(_value(node, 1)))}</h1><b>{html.escape(str(_value(node, 4)))}</b><p>{html.escape(str(_value(node, 3)))}</p><div class="ax-actions">{buttons}</div></div></section>'
    if node.component_id == 3:
        cards = "".join(_render_node(child) for child in node.children)
        return f'<section class="ax-shelf"><h2>{html.escape(str(_value(node, 1)))}</h2><div class="ax-rail">{cards}</div></section>'
    if node.component_id == 4:
        tone = max(1, min(10, int(_value(node, 3, 1))))
        action = node.events[0].action_id if node.events else node.node_id
        return f'<button class="ax-card ax-tone-{tone}" data-action="{action}"><span class="ax-rank">{node.node_id}</span><strong>{html.escape(str(_value(node, 1)))}</strong><small>{html.escape(str(_value(node, 2)))}</small><i>▶</i></button>'
    raise WebRenderError(f"component '{node.component_id}' has no web renderer")


_JS = """const toast=document.querySelector('.ax-toast');document.querySelectorAll('[data-action]').forEach((node)=>node.addEventListener('click',()=>{toast.textContent=`Action ${node.dataset.action} executed`;toast.hidden=false;clearTimeout(window.axToast);window.axToast=setTimeout(()=>toast.hidden=true,2200)}));document.querySelector('[data-search]')?.addEventListener('click',()=>{const value=prompt('Search titles, people, genres');if(value){toast.textContent=`Searching for ${value}`;toast.hidden=false}});"""


_CSS = """*{box-sizing:border-box}html{background:#090909;scroll-behavior:smooth}body{margin:0;background:#090909;color:#f5f5f1;font-family:Arial,Helvetica,sans-serif}.ax-nav{position:fixed;z-index:20;inset:0 0 auto;height:72px;padding:0 clamp(22px,4vw,64px);display:flex;align-items:center;gap:34px;background:linear-gradient(#090909ed,transparent)}.ax-nav strong{color:#e50914;font-size:clamp(24px,3vw,36px);font-weight:950;letter-spacing:-2px}.ax-nav nav{display:flex;gap:22px}.ax-nav a{color:#eee;text-decoration:none;font-size:14px}.ax-nav-actions{margin-left:auto;display:flex;gap:12px}.ax-nav button{border:0;background:transparent;color:white;font-size:27px;cursor:pointer}.ax-nav .ax-avatar{width:34px;height:34px;border-radius:5px;background:#e50914;font-size:13px;font-weight:900}.ax-hero{min-height:78vh;padding:120px clamp(22px,4vw,64px);display:flex;align-items:center;background-image:linear-gradient(90deg,#090909 4%,#090909d1 38%,#0909091f 72%),linear-gradient(0deg,#090909 0%,transparent 45%),var(--ax-hero);background-size:cover;background-position:center}.ax-hero-copy{width:min(610px,84vw)}.ax-hero-copy>span{text-transform:uppercase;letter-spacing:.2em;font-size:14px;font-weight:800}.ax-hero h1{margin:12px 0 6px;font-size:clamp(58px,9vw,124px);line-height:.82;letter-spacing:-.075em;font-weight:950;text-shadow:0 8px 32px #000}.ax-hero b{display:block;color:#46d369;margin:22px 0 12px}.ax-hero p{font-size:clamp(16px,1.7vw,21px);line-height:1.45;text-shadow:0 2px 14px #000}.ax-actions{display:flex;gap:12px;margin-top:28px}.ax-actions button{border:0;border-radius:5px;padding:13px 25px;font-size:18px;font-weight:800;cursor:pointer}.ax-play{background:#fff;color:#111}.ax-more{background:#6d6d6eb3;color:#fff}.ax-shelf{position:relative;z-index:5;margin-top:-80px;margin-bottom:115px;padding-left:clamp(22px,4vw,64px)}.ax-shelf+.ax-shelf{margin-top:-70px}.ax-shelf h2{font-size:clamp(20px,2vw,28px)}.ax-rail{display:grid;grid-auto-flow:column;grid-auto-columns:minmax(220px,22vw);gap:9px;overflow-x:auto;padding:8px 4vw 28px 0;scrollbar-width:none}.ax-card{--a:#7b1b18;--b:#17131b;position:relative;isolation:isolate;min-height:148px;overflow:hidden;padding:20px;border:0;border-radius:5px;color:#fff;text-align:left;cursor:pointer;background:radial-gradient(circle at 75% 25%,#ffffff35,transparent 24%),linear-gradient(135deg,var(--a),var(--b));box-shadow:inset 0 -80px 80px #0008;transition:transform .2s}.ax-card:hover{z-index:2;transform:scale(1.05)}.ax-card strong{display:block;margin-top:52px;font-size:22px}.ax-card small{display:block;margin-top:6px;color:#d1d5db}.ax-card i{position:absolute;right:14px;bottom:14px;border:1px solid #ffffff88;border-radius:50%;width:31px;height:31px;display:grid;place-items:center;font-size:10px}.ax-rank{position:absolute;right:12px;top:-14px;font-size:88px;font-weight:950;color:#ffffff13;-webkit-text-stroke:1px #ffffff35}.ax-tone-2{--a:#532177}.ax-tone-3{--a:#075985}.ax-tone-4{--a:#8a5d12}.ax-tone-5{--a:#881337}.ax-tone-6{--a:#1e3a8a}.ax-tone-7{--a:#701a75}.ax-tone-8{--a:#166534}.ax-tone-9{--a:#334155}.ax-tone-10{--a:#9f1239}.ax-toast{position:fixed;z-index:50;right:24px;bottom:24px;padding:14px 18px;border:1px solid #ffffff25;border-radius:6px;background:#222;box-shadow:0 12px 40px #000}@media(max-width:720px){.ax-nav nav{display:none}.ax-hero{min-height:72vh;background-position:65% center}.ax-rail{grid-auto-columns:72vw}.ax-shelf{margin-top:-55px}.ax-actions button{font-size:15px;padding:11px 17px}}@media(prefers-reduced-motion:reduce){*{scroll-behavior:auto!important;transition:none!important}}"""
