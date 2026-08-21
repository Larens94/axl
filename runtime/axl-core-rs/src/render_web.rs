use std::path::Path;

use crate::ir::*;
use crate::validation;

#[derive(Debug)]
pub struct WebRenderError(pub String);

impl std::fmt::Display for WebRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for WebRenderError {}

pub fn build_web(program: &Program, output: &Path) -> Result<(), WebRenderError> {
    validation::validate(program).map_err(|e| WebRenderError(e.to_string()))?;
    let views: Vec<&UiView> = program.instructions.iter().filter_map(|i| match i {
        Instruction::UiView(v) => Some(v),
        _ => None,
    }).collect();
    if views.len() != 1 {
        return Err(WebRenderError("web build requires exactly one UI view".into()));
    }
    if views[0].root.component_id != 1 {
        return Err(WebRenderError("web view root must use app component 1".into()));
    }
    std::fs::create_dir_all(output).map_err(|e| WebRenderError(e.to_string()))?;
    std::fs::write(output.join("index.html"), render_document(&views[0].root))
        .map_err(|e| WebRenderError(e.to_string()))?;
    std::fs::write(output.join("ax-ui.css"), CSS)
        .map_err(|e| WebRenderError(e.to_string()))?;
    std::fs::write(output.join("ax-ui.js"), JS)
        .map_err(|e| WebRenderError(e.to_string()))?;
    Ok(())
}

fn text(node: &UiNode, id: i32) -> String {
    node.properties.iter()
        .find(|p| p.property_id == id)
        .and_then(|p| match &p.value { Expression::Literal(Value::String(s)) => Some(s.clone()), _ => None })
        .unwrap_or_default()
}

fn integer(node: &UiNode, id: i32) -> i64 {
    node.properties.iter()
        .find(|p| p.property_id == id)
        .and_then(|p| match &p.value { Expression::Literal(Value::Int(n)) => Some(*n), _ => None })
        .unwrap_or(1)
}

fn safe_url(value: &str) -> bool {
    value.starts_with("https://") && value.bytes().all(|b| b.is_ascii_alphanumeric() || b".:/_?&=%+-".contains(&b))
}

fn escape_html(value: &str) -> String {
    value.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

fn render_document(root: &UiNode) -> String {
    let title = escape_html(&text(root, 1));
    let content: String = root.children.iter().map(render_node).collect();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><link rel=\"stylesheet\" href=\"ax-ui.css\"></head><body><header class=\"ax-nav\"><strong>{title}</strong><nav><a href=\"#home\">Home</a><a href=\"#catalogue\">Series</a><a href=\"#catalogue\">Films</a><a href=\"#catalogue\">New &amp; Popular</a></nav><div><button data-search aria-label=\"Search\">&#x2315;</button><button class=\"ax-avatar\" aria-label=\"Profile\">AX</button></div></header><main>{content}</main><div class=\"ax-toast\" role=\"status\" hidden></div><script src=\"ax-ui.js\"></script></body></html>"
    )
}

fn render_node(node: &UiNode) -> String {
    match node.component_id {
        2 => {
            let image = text(node, 5);
            let image_attr = if safe_url(&image) { escape_html(&image) } else { String::new() };
            let buttons: String = node.events.iter().map(|ev| {
                let cls = if ev.event_id == 1 { "ax-play" } else { "ax-more" };
                let label = if ev.event_id == 1 { "\u{25b6} Play" } else { "\u{2139}\u{fe0f} More Info" };
                format!("<button class=\"{cls}\" data-action=\"{}\">{label}</button>", ev.action_id)
            }).collect();
            format!(
                "<section id=\"home\" class=\"ax-hero\" style=\"--ax-hero:url(&quot;{image_attr}&quot;)\"><div class=\"ax-hero-copy\"><span>{}</span><h1>{}</h1><b>{}</b><p>{}</p><div class=\"ax-actions\">{buttons}</div></div></section>",
                escape_html(&text(node, 2)), escape_html(&text(node, 1)), escape_html(&text(node, 4)), escape_html(&text(node, 3))
            )
        }
        3 => {
            let cards: String = node.children.iter().map(render_node).collect();
            format!("<section id=\"catalogue\" class=\"ax-shelf\"><h2>{}</h2><div class=\"ax-rail\">{cards}</div></section>", escape_html(&text(node, 1)))
        }
        4 => {
            let tone = integer(node, 3).clamp(1, 10);
            let size = integer(node, 4).clamp(1, 2);
            let action = node.events.first().map(|ev| ev.action_id).unwrap_or(node.node_id);
            format!(
                "<button class=\"ax-card ax-tone-{tone} ax-size-{size}\" data-action=\"{action}\"><span class=\"ax-rank\">{}</span><strong>{}</strong><small>{}</small><i>\u{25b6}</i></button>",
                node.node_id, escape_html(&text(node, 1)), escape_html(&text(node, 2))
            )
        }
        _ => String::new(),
    }
}

const JS: &str = "const t=document.querySelector('.ax-toast');document.querySelectorAll('[data-action]').forEach(n=>n.addEventListener('click',()=>{t.textContent=`Action ${n.dataset.action} executed`;t.hidden=false;clearTimeout(window.axT);window.axT=setTimeout(()=>t.hidden=true,2200)}));document.querySelector('[data-search]')?.addEventListener('click',()=>{const q=prompt('Search titles, people, genres');if(q){t.textContent=`Searching for ${q}`;t.hidden=false}});";

const CSS: &str = "*{box-sizing:border-box}html{background:#090909;scroll-behavior:smooth}body{margin:0;background:#090909;color:#f5f5f1;font-family:Arial,Helvetica,sans-serif}.ax-nav{position:fixed;z-index:20;inset:0 0 auto;height:72px;padding:0 clamp(22px,4vw,64px);display:flex;align-items:center;gap:34px;background:linear-gradient(#090909ed,transparent)}.ax-nav strong{color:#e50914;font-size:clamp(24px,3vw,36px);font-weight:950;letter-spacing:-2px}.ax-nav nav{display:flex;gap:22px}.ax-nav a{color:#eee;text-decoration:none;font-size:14px}.ax-nav>div{margin-left:auto;display:flex;gap:12px}.ax-nav button{border:0;background:transparent;color:white;font-size:27px;cursor:pointer}.ax-nav .ax-avatar{width:34px;height:34px;border-radius:5px;background:#e50914;font-size:13px;font-weight:900}.ax-hero{min-height:78vh;padding:120px clamp(22px,4vw,64px);display:flex;align-items:center;background-image:linear-gradient(90deg,#090909 4%,#090909d1 38%,#0909091f 72%),linear-gradient(0deg,#090909 0%,transparent 45%),var(--ax-hero);background-size:cover;background-position:center}.ax-hero-copy{width:min(610px,84vw)}.ax-hero-copy>span{text-transform:uppercase;letter-spacing:.2em;font-size:14px;font-weight:800}.ax-hero h1{margin:12px 0 6px;font-size:clamp(58px,9vw,124px);line-height:.82;letter-spacing:-.075em;font-weight:950;text-shadow:0 8px 32px #000}.ax-hero b{display:block;color:#46d369;margin:22px 0 12px}.ax-hero p{font-size:clamp(16px,1.7vw,21px);line-height:1.45;text-shadow:0 2px 14px #000}.ax-actions{display:flex;gap:12px;margin-top:28px}.ax-actions button{border:0;border-radius:5px;padding:13px 25px;font-size:18px;font-weight:800;cursor:pointer}.ax-play{background:#fff;color:#111}.ax-more{background:#6d6d6eb3;color:#fff}.ax-shelf{position:relative;z-index:5;margin-top:-80px;margin-bottom:115px;padding-left:clamp(22px,4vw,64px)}.ax-shelf+.ax-shelf{margin-top:-70px}.ax-shelf h2{font-size:clamp(20px,2vw,28px)}.ax-rail{display:grid;grid-auto-flow:column;grid-auto-columns:minmax(220px,1fr);gap:10px;overflow-x:auto;scroll-snap-type:x mandatory;-webkit-overflow-scrolling:touch}.ax-rail::-webkit-scrollbar{display:none}.ax-card{scroll-snap-align:start;display:flex;flex-direction:column;border:0;border-radius:8px;padding:22px 18px;min-height:140px;color:#fff;font-size:15px;text-align:left;cursor:pointer;position:relative;overflow:hidden;transition:transform .2s}.ax-card:hover{transform:scale(1.04)}.ax-card i{position:absolute;bottom:14px;right:14px;font-style:normal;font-size:22px;opacity:.8}.ax-rank{font-size:64px;font-weight:950;position:absolute;right:12px;bottom:8px;opacity:.18;line-height:1}.ax-card strong{font-size:17px;font-weight:700;margin-bottom:6px;z-index:1}.ax-card small{opacity:.8;z-index:1}.ax-tone-1{background:#1a1a2e}.ax-tone-2{background:#16213e}.ax-tone-3{background:#0f3460}.ax-tone-4{background:#533483}.ax-tone-5{background:#e94560}.ax-tone-6{background:#2b2d42}.ax-tone-7{background:#8d99ae}.ax-tone-8{background:#606c38}.ax-tone-9{background:#283618}.ax-tone-10{background:#bc6c25}.ax-size-2{grid-column:span 2;min-height:190px}.ax-toast{position:fixed;bottom:22px;left:50%;transform:translateX(-50%);background:#333;color:#fff;padding:12px 22px;border-radius:6px;font-size:14px;z-index:100}";
