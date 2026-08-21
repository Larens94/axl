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

fn bool_prop(node: &UiNode, id: i32) -> bool {
    node.properties.iter()
        .find(|p| p.property_id == id)
        .and_then(|p| match &p.value { Expression::Literal(Value::Bool(b)) => Some(*b), _ => None })
        .unwrap_or(false)
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
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><link rel=\"stylesheet\" href=\"ax-ui.css\"></head><body>{content}<script src=\"ax-ui.js\"></script></body></html>"
    )
}

fn render_node(node: &UiNode) -> String {
    match node.component_id {
        // ====================================================================
        // LAYOUT
        // ====================================================================
        1 => {
            // App — root container
            let content: String = node.children.iter().map(render_node).collect();
            content
        }
        10 => {
            // Container
            let class = text(node, 1);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-container {class}\">{content}</div>")
        }
        11 => {
            // Grid
            let cols = integer(node, 1).max(1);
            let gap = text(node, 2);
            let class = text(node, 3);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-grid ax-grid-{cols} {class}\" style=\"gap:{gap}\">{content}</div>")
        }
        12 => {
            // Flex
            let dir = text(node, 1);
            let justify = text(node, 2);
            let align = text(node, 3);
            let gap = text(node, 4);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-flex\" style=\"flex-direction:{dir};justify-content:{justify};align-items:{align};gap:{gap}\">{content}</div>")
        }
        13 => {
            // Divider
            let style = text(node, 1);
            let color = text(node, 2);
            format!("<hr class=\"ax-divider\" style=\"border-style:{style};border-color:{color}\">")
        }

        // ====================================================================
        // NAVIGATION
        // ====================================================================
        20 => {
            // Navbar
            let brand = text(node, 1);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<nav class=\"ax-navbar\"><strong>{}</strong>{content}</nav>", escape_html(&brand))
        }
        21 => {
            // Sidebar
            let pos = text(node, 1);
            let width = integer(node, 2);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<aside class=\"ax-sidebar ax-sidebar-{pos}\" style=\"width:{width}px\">{content}</aside>")
        }
        22 => {
            // Tabs
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-tabs\">{content}</div>")
        }
        23 => {
            // Breadcrumb
            let content: String = node.children.iter().map(render_node).collect();
            format!("<nav class=\"ax-breadcrumb\">{content}</nav>")
        }
        24 => {
            // Pagination
            let current = integer(node, 1);
            let total = integer(node, 2);
            let mut pages = String::new();
            for i in 1..=total {
                let active = if i == current { " active" } else { "" };
                pages.push_str(&format!("<button class=\"ax-page{active}\" data-page=\"{i}\">{i}</button>"));
            }
            format!("<nav class=\"ax-pagination\">{pages}</nav>")
        }

        // ====================================================================
        // CONTENT
        // ====================================================================
        30 => {
            // Hero
            let image = text(node, 5);
            let image_attr = if safe_url(&image) { escape_html(&image) } else { String::new() };
            let buttons: String = node.events.iter().map(|ev| {
                let cls = if ev.event_id == 1 { "ax-btn-primary" } else { "ax-btn-secondary" };
                let label = if ev.event_id == 1 { "Get Started" } else { "Learn More" };
                format!("<button class=\"ax-btn {cls}\" data-action=\"{}\">{label}</button>", ev.action_id)
            }).collect();
            let children: String = node.children.iter().map(render_node).collect();
            format!(
                "<section class=\"ax-hero\" style=\"--ax-hero:url(&quot;{image_attr}&quot;)\"><div class=\"ax-hero-content\"><span class=\"ax-hero-badge\">{}</span><h1>{}</h1><b>{}</b><p>{}</p><div class=\"ax-hero-actions\">{buttons}</div>{children}</div></section>",
                escape_html(&text(node, 4)), escape_html(&text(node, 1)), escape_html(&text(node, 2)), escape_html(&text(node, 3))
            )
        }
        31 => {
            // Shelf
            let cards: String = node.children.iter().map(render_node).collect();
            let subtitle = text(node, 2);
            let sub = if subtitle.is_empty() { String::new() } else { format!("<p>{}</p>", escape_html(&subtitle)) };
            format!("<section class=\"ax-shelf\"><h2>{}</h2>{sub}<div class=\"ax-rail\">{cards}</div></section>", escape_html(&text(node, 1)))
        }
        32 => {
            // Media Card
            let tone = integer(node, 3).clamp(1, 10);
            let size = integer(node, 4).clamp(1, 2);
            let action = node.events.first().map(|ev| ev.action_id).unwrap_or(node.node_id);
            format!(
                "<article class=\"ax-card ax-tone-{tone} ax-size-{size}\" data-action=\"{action}\"><span class=\"ax-card-rank\">{}</span><strong>{}</strong><small>{}</small></article>",
                node.node_id, escape_html(&text(node, 1)), escape_html(&text(node, 2))
            )
        }
        33 => {
            // Text Block
            let tag = text(node, 2);
            let tag = if tag.is_empty() { "p".to_string() } else { tag };
            let class = text(node, 3);
            format!("<{tag} class=\"ax-text {class}\">{}</{tag}>", escape_html(&text(node, 1)))
        }
        34 => {
            // Image
            let src = text(node, 1);
            let alt = text(node, 2);
            let class = text(node, 5);
            format!("<img class=\"ax-img {class}\" src=\"{}\" alt=\"{}\">", escape_html(&src), escape_html(&alt))
        }
        35 => {
            // Video
            let src = text(node, 1);
            let poster = text(node, 2);
            let autoplay = if bool_prop(node, 3) { " autoplay" } else { "" };
            let controls = if bool_prop(node, 4) { "" } else { " nocontrols" };
            format!("<video class=\"ax-video\" src=\"{}\" poster=\"{}\"{autoplay}{controls}></video>", escape_html(&src), escape_html(&poster))
        }

        // ====================================================================
        // FORMS
        // ====================================================================
        40 => {
            // Input
            let input_type = text(node, 3);
            let input_type = if input_type.is_empty() { "text".to_string() } else { input_type };
            let label = text(node, 4);
            let label_html = if label.is_empty() { String::new() } else { format!("<label>{}</label>", escape_html(&label)) };
            format!("<div class=\"ax-field\">{label_html}<input class=\"ax-input\" type=\"{input_type}\" placeholder=\"{}\" value=\"{}\">", escape_html(&text(node, 1)), escape_html(&text(node, 2)))
        }
        41 => {
            // Textarea
            let label = text(node, 4);
            let label_html = if label.is_empty() { String::new() } else { format!("<label>{}</label>", escape_html(&label)) };
            format!("<div class=\"ax-field\">{label_html}<textarea class=\"ax-textarea\" rows=\"{}\" placeholder=\"{}\">{}</textarea></div>", integer(node, 3), escape_html(&text(node, 1)), escape_html(&text(node, 2)))
        }
        42 => {
            // Select
            let options = text(node, 1);
            let mut options_html = String::new();
            for opt in options.split(',') {
                options_html.push_str(&format!("<option>{}</option>", escape_html(opt.trim())));
            }
            let label = text(node, 3);
            let label_html = if label.is_empty() { String::new() } else { format!("<label>{}</label>", escape_html(&label)) };
            format!("<div class=\"ax-field\">{label_html}<select class=\"ax-select\">{options_html}</select></div>")
        }
        43 => {
            // Checkbox
            let checked = if bool_prop(node, 2) { " checked" } else { "" };
            format!("<label class=\"ax-checkbox\"><input type=\"checkbox\"{checked}> {}</label>", escape_html(&text(node, 1)))
        }
        44 => {
            // Radio
            let checked = if bool_prop(node, 3) { " checked" } else { "" };
            format!("<label class=\"ax-radio\"><input type=\"radio\" name=\"radio\" value=\"{}\"{checked}> {}</label>", escape_html(&text(node, 2)), escape_html(&text(node, 1)))
        }
        45 => {
            // Button
            let variant = text(node, 2);
            let variant = if variant.is_empty() { "primary".to_string() } else { variant };
            let disabled = if bool_prop(node, 4) { " disabled" } else { "" };
            let action = node.events.first().map(|ev| ev.action_id).unwrap_or(0);
            format!("<button class=\"ax-btn ax-btn-{variant}\" data-action=\"{action}\"{disabled}>{}</button>", escape_html(&text(node, 1)))
        }

        // ====================================================================
        // FEEDBACK
        // ====================================================================
        50 => {
            // Alert
            let variant = text(node, 3);
            let variant = if variant.is_empty() { "info".to_string() } else { variant };
            let dismissible = if bool_prop(node, 4) { " ax-dismissible" } else { "" };
            format!("<div class=\"ax-alert ax-alert-{variant}{dismissible}\"><strong>{}</strong><p>{}</p></div>", escape_html(&text(node, 1)), escape_html(&text(node, 2)))
        }
        51 => {
            // Toast
            let variant = text(node, 2);
            let variant = if variant.is_empty() { "info".to_string() } else { variant };
            format!("<div class=\"ax-toast ax-toast-{variant}\" role=\"status\">{}</div>", escape_html(&text(node, 1)))
        }
        52 => {
            // Modal
            let open = if bool_prop(node, 2) { " open" } else { "" };
            let size = text(node, 3);
            let size = if size.is_empty() { "md".to_string() } else { size };
            let content: String = node.children.iter().map(render_node).collect();
            format!("<dialog class=\"ax-modal ax-modal-{size}\"{open}><div class=\"ax-modal-header\"><h2>{}</h2></div><div class=\"ax-modal-body\">{content}</div></dialog>", escape_html(&text(node, 1)))
        }
        53 => {
            // Tooltip
            let content: String = node.children.iter().map(render_node).collect();
            format!("<span class=\"ax-tooltip\" data-tooltip=\"{}\">{content}</span>", escape_html(&text(node, 1)))
        }
        54 => {
            // Progress
            let value = integer(node, 1).clamp(0, 100);
            let variant = text(node, 2);
            let variant = if variant.is_empty() { "linear".to_string() } else { variant };
            format!("<div class=\"ax-progress ax-progress-{variant}\"><div class=\"ax-progress-bar\" style=\"width:{value}%\"></div></div>")
        }

        // ====================================================================
        // DATA DISPLAY
        // ====================================================================
        60 => {
            // Table
            let content: String = node.children.iter().map(render_node).collect();
            format!("<table class=\"ax-table\">{content}</table>")
        }
        61 => {
            // List
            let variant = text(node, 2);
            let variant = if variant.is_empty() { "default".to_string() } else { variant };
            let content: String = node.children.iter().map(render_node).collect();
            format!("<ul class=\"ax-list ax-list-{variant}\">{content}</ul>")
        }
        62 => {
            // Chart
            let chart_type = text(node, 1);
            let title = text(node, 2);
            format!("<div class=\"ax-chart\" data-chart-type=\"{chart_type}\"><h3>{}</h3><canvas></canvas></div>", escape_html(&title))
        }
        63 => {
            // Badge
            let variant = text(node, 2);
            let variant = if variant.is_empty() { "default".to_string() } else { variant };
            format!("<span class=\"ax-badge ax-badge-{variant}\">{}</span>", escape_html(&text(node, 1)))
        }

        // ====================================================================
        // DISPLAY
        // ====================================================================
        70 => {
            // Avatar
            let name = text(node, 2);
            let size = integer(node, 3);
            let initials: String = name.split_whitespace().take(2).map(|w| w.chars().next().unwrap_or_default()).collect();
            format!("<div class=\"ax-avatar\" style=\"width:{size}px;height:{size}px\">{}</div>", escape_html(&initials))
        }
        71 => {
            // Icon
            let name = text(node, 1);
            format!("<span class=\"ax-icon ax-icon-{name}\"></span>")
        }
        72 => {
            // Card
            let content: String = node.children.iter().map(render_node).collect();
            let title = text(node, 1);
            let subtitle = text(node, 2);
            let title_html = if title.is_empty() { String::new() } else { format!("<h3>{}</h3>", escape_html(&title)) };
            let sub_html = if subtitle.is_empty() { String::new() } else { format!("<p>{}</p>", escape_html(&subtitle)) };
            format!("<article class=\"ax-card-generic\">{title_html}{sub_html}{content}</article>")
        }
        73 => {
            // Accordion
            let open = if bool_prop(node, 2) { " open" } else { "" };
            let content: String = node.children.iter().map(render_node).collect();
            format!("<details class=\"ax-accordion\"{open}><summary>{}</summary><div class=\"ax-accordion-content\">{content}</div></details>", escape_html(&text(node, 1)))
        }
        74 => {
            // Carousel
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-carousel\">{content}</div>")
        }

        // ====================================================================
        // AGENT-SPECIFIC
        // ====================================================================
        80 => {
            // Chat
            let content: String = node.children.iter().map(render_node).collect();
            format!("<div class=\"ax-chat\"><div class=\"ax-chat-messages\">{content}</div><div class=\"ax-chat-input\"><input type=\"text\" placeholder=\"{}\"><button>Send</button></div></div>", escape_html(&text(node, 2)))
        }
        81 => {
            // Message
            let variant = text(node, 4);
            let variant = if variant.is_empty() { "user".to_string() } else { variant };
            format!("<div class=\"ax-message ax-message-{variant}\"><strong>{}</strong><p>{}</p><small>{}</small></div>", escape_html(&text(node, 1)), escape_html(&text(node, 2)), escape_html(&text(node, 3)))
        }
        82 => {
            // Typing Indicator
            let active = if bool_prop(node, 2) { " active" } else { "" };
            format!("<div class=\"ax-typing{active}\"><span></span><span></span><span></span></div>")
        }
        83 => {
            // Agent Card
            let status = text(node, 4);
            let status = if status.is_empty() { "offline".to_string() } else { status };
            format!("<div class=\"ax-agent-card\"><div class=\"ax-agent-avatar\">{}</div><strong>{}</strong><small>{}</small><span class=\"ax-status ax-status-{status}\"></span></div>", escape_html(&text(node, 3)), escape_html(&text(node, 1)), escape_html(&text(node, 2)))
        }
        84 => {
            // Tool Output
            let status = text(node, 4);
            let status = if status.is_empty() { "success".to_string() } else { status };
            format!("<div class=\"ax-tool-output ax-tool-{status}\"><div class=\"ax-tool-header\"><code>{}</code><span class=\"ax-badge ax-badge-{status}\">{status}</span></div><pre>{}</pre></div>", escape_html(&text(node, 1)), escape_html(&text(node, 3)))
        }
        85 => {
            // Memory Display
            format!("<div class=\"ax-memory\"><code>{}</code><span class=\"ax-badge\">{}</span><pre>{}</pre></div>", escape_html(&text(node, 1)), escape_html(&text(node, 3)), escape_html(&text(node, 2)))
        }
        86 => {
            // Reasoning Trace
            format!("<div class=\"ax-reasoning\"><div class=\"ax-reasoning-header\"><strong>Reasoning</strong><span>{}</span></div><div class=\"ax-reasoning-steps\"><pre>{}</pre></div><div class=\"ax-reasoning-conclusion\"><strong>Conclusion:</strong> {}</div></div>", escape_html(&text(node, 1)), escape_html(&text(node, 2)), escape_html(&text(node, 3)))
        }

        _ => String::new(),
    }
}

const JS: &str = "document.querySelectorAll('[data-action]').forEach(n=>n.addEventListener('click',()=>{const e=new CustomEvent('ax:action',{detail:{action:n.dataset.action}});n.dispatchEvent(e)}));document.querySelectorAll('[data-page]').forEach(n=>n.addEventListener('click',()=>{const e=new CustomEvent('ax:page',{detail:{page:parseInt(n.dataset.page)}});n.dispatchEvent(e)}));";

const CSS: &str = r#"*{box-sizing:border-box;margin:0;padding:0}
html{background:#090909;scroll-behavior:smooth}
body{background:#090909;color:#f5f5f1;font-family:system-ui,-apple-system,sans-serif;line-height:1.5}
a{color:#e50914;text-decoration:none}
h1,h2,h3,h4{font-weight:700;line-height:1.1}
pre{background:#1a1a1a;padding:16px;border-radius:8px;overflow-x:auto;font-size:14px}
code{background:#1a1a1a;padding:2px 6px;border-radius:4px;font-size:14px}
.ax-container{max-width:1200px;margin:0 auto;padding:0 24px}
.ax-grid{display:grid}
.ax-grid-2{grid-template-columns:repeat(2,1fr)}
.ax-grid-3{grid-template-columns:repeat(3,1fr)}
.ax-grid-4{grid-template-columns:repeat(4,1fr)}
.ax-flex{display:flex}
.ax-divider{border:none;border-top:1px solid #333;margin:24px 0}
.ax-navbar{display:flex;align-items:center;padding:16px 24px;background:#0d0d0d;border-bottom:1px solid #222}
.ax-navbar strong{color:#e50914;font-size:24px;font-weight:900}
.ax-sidebar{background:#0d0d0d;padding:24px;border-right:1px solid #222;height:100vh;position:sticky;top:0}
.ax-tabs{display:flex;gap:0;border-bottom:1px solid #333}
.ax-breadcrumb{display:flex;gap:8px;align-items:center;font-size:14px;color:#888}
.ax-pagination{display:flex;gap:4px}
.ax-page{background:#1a1a1a;border:1px solid #333;color:#fff;padding:8px 12px;border-radius:4px;cursor:pointer}
.ax-page.active{background:#e50914;border-color:#e50914}
.ax-hero{min-height:80vh;padding:120px 48px;display:flex;align-items:center;background:linear-gradient(90deg,#090909 4%,#090909d1 38%,#0909091f 72%),linear-gradient(0deg,#090909 0%,transparent 45%),var(--ax-hero);background-size:cover;background-position:center}
.ax-hero-content{max-width:640px}
.ax-hero-badge{text-transform:uppercase;letter-spacing:.2em;font-size:14px;font-weight:800;color:#e50914}
.ax-hero h1{margin:16px 0;font-size:clamp(48px,8vw,96px);line-height:.9;letter-spacing:-.04em;font-weight:900}
.ax-hero b{display:block;color:#46d369;margin:16px 0;font-size:20px}
.ax-hero p{font-size:18px;color:#aaa;line-height:1.6}
.ax-hero-actions{display:flex;gap:12px;margin-top:32px}
.ax-btn{border:0;border-radius:8px;padding:12px 24px;font-size:16px;font-weight:600;cursor:pointer;transition:transform .15s}
.ax-btn:hover{transform:scale(1.02)}
.ax-btn-primary{background:#fff;color:#111}
.ax-btn-secondary{background:rgba(255,255,255,.15);color:#fff}
.ax-btn-ghost{background:transparent;color:#fff;border:1px solid #444}
.ax-btn-danger{background:#e50914;color:#fff}
.ax-shelf{margin:32px 0;padding-left:48px}
.ax-shelf h2{font-size:24px;margin-bottom:16px}
.ax-shelf p{color:#888;margin-bottom:16px}
.ax-rail{display:grid;grid-auto-flow:column;grid-auto-columns:minmax(200px,1fr);gap:12px;overflow-x:auto;scroll-snap-type:x mandatory}
.ax-rail::-webkit-scrollbar{display:none}
.ax-card{scroll-snap-align:start;display:flex;flex-direction:column;border:0;border-radius:8px;padding:20px;min-height:140px;color:#fff;text-align:left;cursor:pointer;position:relative;transition:transform .15s}
.ax-card:hover{transform:scale(1.03)}
.ax-card-rank{font-size:56px;font-weight:900;position:absolute;right:12px;bottom:8px;opacity:.15}
.ax-card strong{font-size:16px;font-weight:600;z-index:1}
.ax-card small{opacity:.7;font-size:14px;z-index:1}
.ax-tone-1{background:#1a1a2e}.ax-tone-2{background:#16213e}.ax-tone-3{background:#0f3460}
.ax-tone-4{background:#533483}.ax-tone-5{background:#e94560}.ax-tone-6{background:#2b2d42}
.ax-tone-7{background:#8d99ae}.ax-tone-8{background:#606c38}.ax-tone-9{background:#283618}
.ax-tone-10{background:#bc6c25}
.ax-size-2{grid-column:span 2;min-height:180px}
.ax-field{margin-bottom:16px}
.ax-field label{display:block;margin-bottom:6px;font-size:14px;font-weight:500;color:#ccc}
.ax-input,.ax-textarea,.ax-select{width:100%;padding:10px 14px;background:#1a1a1a;border:1px solid #333;border-radius:6px;color:#fff;font-size:16px}
.ax-input:focus,.ax-textarea:focus,.ax-select:focus{outline:none;border-color:#e50914}
.ax-checkbox,.ax-radio{display:flex;align-items:center;gap:8px;cursor:pointer}
.ax-alert{padding:16px;border-radius:8px;margin-bottom:16px}
.ax-alert-info{background:#1a2332;border-left:4px solid #3b82f6}
.ax-alert-success{background:#1a2e1a;border-left:4px solid #46d369}
.ax-alert-warning{background:#2e2a1a;border-left:4px solid #f59e0b}
.ax-alert-error{background:#2e1a1a;border-left:4px solid #ef4444}
.ax-toast{position:fixed;bottom:24px;left:50%;transform:translateX(-50%);padding:12px 24px;border-radius:8px;z-index:100}
.ax-progress{height:8px;background:#1a1a1a;border-radius:4px;overflow:hidden}
.ax-progress-bar{height:100%;background:#e50914;border-radius:4px;transition:width .3s}
.ax-badge{display:inline-block;padding:2px 8px;border-radius:4px;font-size:12px;font-weight:600}
.ax-badge-default{background:#333;color:#fff}
.ax-badge-success{background:#1a4a1a;color:#46d369}
.ax-badge-warning{background:#3a3a1a;color:#f59e0b}
.ax-badge-error{background:#4a1a1a;color:#ef4444}
.ax-badge-info{background:#1a2a4a;color:#3b82f6}
.ax-avatar{display:flex;align-items:center;justify-content:center;border-radius:50%;background:#e50914;font-weight:700;font-size:14px}
.ax-agent-card{display:flex;flex-direction:column;align-items:center;gap:8px;padding:16px;background:#1a1a1a;border-radius:8px}
.ax-chat{display:flex;flex-direction:column;height:400px;background:#0d0d0d;border-radius:8px;overflow:hidden}
.ax-chat-messages{flex:1;overflow-y:auto;padding:16px}
.ax-chat-input{display:flex;gap:8px;padding:16px;border-top:1px solid #222}
.ax-chat-input input{flex:1;padding:10px;background:#1a1a1a;border:1px solid #333;border-radius:6px;color:#fff}
.ax-message{padding:12px;border-radius:8px;margin-bottom:8px}
.ax-message-user{background:#1a2332;margin-left:48px}
.ax-message-agent{background:#1a1a1a;margin-right:48px}
.ax-message-system{background:#1a1a1a;text-align:center;color:#888}
.ax-typing{display:flex;gap:4px;padding:12px}
.ax-typing span{width:8px;height:8px;background:#666;border-radius:50%;animation:typing 1.4s infinite}
.ax-typing span:nth-child(2){animation-delay:.2s}
.ax-typing span:nth-child(3){animation-delay:.4s}
@keyframes typing{0%,60%,100%{transform:translateY(0)}30%{transform:translateY(-8px)}}
.ax-tool-output{background:#1a1a1a;border-radius:8px;overflow:hidden;margin-bottom:16px}
.ax-tool-header{display:flex;justify-content:space-between;align-items:center;padding:12px 16px;border-bottom:1px solid #222}
.ax-tool-output pre{margin:0;border-radius:0}
.ax-reasoning{background:#1a1a1a;border-radius:8px;padding:16px;margin-bottom:16px}
.ax-reasoning-header{display:flex;justify-content:space-between;margin-bottom:12px}
.ax-reasoning-steps{margin-bottom:12px}
.ax-reasoning-conclusion{padding-top:12px;border-top:1px solid #333}
.ax-memory{background:#1a1a1a;border-radius:8px;padding:12px;margin-bottom:8px}
.ax-accordion{border:1px solid #333;border-radius:8px;overflow:hidden;margin-bottom:8px}
.ax-accordion summary{padding:12px 16px;cursor:pointer;background:#0d0d0d}
.ax-accordion-content{padding:16px}
"#;
