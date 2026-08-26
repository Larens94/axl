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

fn text_or_bind(node: &UiNode, id: i32) -> (String, Option<String>) {
    if let Some(prop) = node.properties.iter().find(|p| p.property_id == id) {
        match &prop.value {
            Expression::Literal(Value::String(s)) => (s.clone(), None),
            Expression::Variable(name) => (String::new(), Some(name.clone())),
            _ => (String::new(), None),
        }
    } else {
        (String::new(), None)
    }
}

fn bind_attr(node: &UiNode, id: i32) -> String {
    let (val, bind) = text_or_bind(node, id);
    if let Some(var_name) = bind {
        format!(" data-bind=\"{}\"", escape_html(&var_name))
    } else {
        String::new()
    }
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
            let (title, title_bind) = text_or_bind(node, 1);
            let (subtitle, sub_bind) = text_or_bind(node, 2);
            let (desc, desc_bind) = text_or_bind(node, 3);
            let (badge, badge_bind) = text_or_bind(node, 4);
            let title_html = if let Some(v) = title_bind { format!("<h1 data-bind=\"{}\"></h1>", escape_html(&v)) } else { format!("<h1>{}</h1>", escape_html(&title)) };
            let sub_html = if let Some(v) = sub_bind { format!("<b data-bind=\"{}\"></b>", escape_html(&v)) } else { format!("<b>{}</b>", escape_html(&subtitle)) };
            let desc_html = if let Some(v) = desc_bind { format!("<p data-bind=\"{}\"></p>", escape_html(&v)) } else { format!("<p>{}</p>", escape_html(&desc)) };
            let badge_html = if let Some(v) = badge_bind { format!("<span class=\"ax-hero-badge\" data-bind=\"{}\"></span>", escape_html(&v)) } else { format!("<span class=\"ax-hero-badge\">{}</span>", escape_html(&badge)) };
            format!(
                "<section class=\"ax-hero\" style=\"--ax-hero:url(&quot;{image_attr}&quot;)\"><div class=\"ax-hero-content\">{badge_html}{title_html}{sub_html}{desc_html}<div class=\"ax-hero-actions\">{buttons}</div>{children}</div></section>"
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
            let (content, bind) = text_or_bind(node, 1);
            if let Some(var_name) = bind {
                format!("<{tag} class=\"ax-text {class}\" data-bind=\"{}\"></{tag}>", escape_html(&var_name))
            } else {
                format!("<{tag} class=\"ax-text {class}\">{}</{tag}>", escape_html(&content))
            }
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
            let (content, bind) = text_or_bind(node, 1);
            if let Some(var_name) = bind {
                format!("<span class=\"ax-badge ax-badge-{variant}\" data-bind=\"{}\"></span>", escape_html(&var_name))
            } else {
                format!("<span class=\"ax-badge ax-badge-{variant}\">{}</span>", escape_html(&content))
            }
        }
        64 => {
            // Resource-backed data table; a richer target can hydrate this shell.
            let resource = text(node, 1);
            let label = text(node, 2);
            let content: String = node.children.iter().map(render_node).collect();
            format!("<section class=\"ax-data-table\" data-resource=\"{}\"><h2>{}</h2><div class=\"ax-data-table-columns\">{content}</div></section>", escape_html(&resource), escape_html(&label))
        }
        65 => {
            let field = text(node, 1);
            let label = text(node, 2);
            let kind = text(node, 3);
            format!("<span class=\"ax-table-column\" data-field=\"{}\" data-kind=\"{}\">{}</span>", escape_html(&field), escape_html(&kind), escape_html(&label))
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

const JS: &str = r#"
// AX-UI Runtime v1.0
(function(){
  'use strict';

  // === State ===
  const state = { data: {}, listeners: [] };
  window.AX = {
    state,
    set(key, val) { state.data[key] = val; this.notify(key); },
    get(key) { return state.data[key]; },
    notify(key) { state.listeners.forEach(fn => fn(key, state.data[key])); },
    onChange(fn) { state.listeners.push(fn); }
  };

  // === Event Dispatch ===
  document.addEventListener('click', function(e) {
    const action = e.target.closest('[data-action]');
    if (action) {
      const ev = new CustomEvent('ax:action', { detail: { action: action.dataset.action, target: action } });
      document.dispatchEvent(ev);
    }
  });

  document.addEventListener('click', function(e) {
    const page = e.target.closest('[data-page]');
    if (page) {
      const ev = new CustomEvent('ax:page', { detail: { page: parseInt(page.dataset.page) } });
      document.dispatchEvent(ev);
    }
  });

  // === Data Binding ===
  document.addEventListener('DOMContentLoaded', function() {
    document.querySelectorAll('[data-bind]').forEach(function(el) {
      const key = el.dataset.bind;
      if (state.data[key] !== undefined) {
        el.textContent = state.data[key];
      }
    });
  });

  AX.onChange(function(key, val) {
    document.querySelectorAll('[data-bind="' + key + '"]').forEach(function(el) {
      el.textContent = val;
    });
  });

  // === Fetch API ===
  AX.fetch = async function(endpoint, options) {
    try {
      const res = await fetch('/api/' + endpoint, options);
      if (!res.ok) throw new Error('Fetch failed: ' + res.status);
      return await res.json();
    } catch (e) {
      console.error('AX.fetch error:', e);
      return null;
    }
  };

  AX.get = async function(endpoint) { return this.fetch(endpoint); };
  AX.post = async function(endpoint, data) {
    return this.fetch(endpoint, { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });
  };
  AX.put = async function(endpoint, id, data) {
    return this.fetch(endpoint + '/' + id, { method: 'PUT', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(data) });
  };
  AX.del = async function(endpoint, id) {
    return this.fetch(endpoint + '/' + id, { method: 'DELETE' });
  };

  // === Table Rendering ===
  AX.renderTable = function(selector, data, columns) {
    const el = document.querySelector(selector);
    if (!el || !data) return;
    let html = '<table class="ax-table"><thead><tr>';
    columns.forEach(function(col) { html += '<th>' + col.label + '</th>'; });
    html += '</tr></thead><tbody>';
    data.forEach(function(row) {
      html += '<tr>';
      columns.forEach(function(col) {
        const val = row[col.key] || '';
        html += '<td>' + val + '</td>';
      });
      html += '</tr>';
    });
    html += '</tbody></table>';
    el.innerHTML = html;
  };

  // === List Rendering ===
  AX.renderList = function(selector, data, template) {
    const el = document.querySelector(selector);
    if (!el || !data) return;
    el.innerHTML = data.map(template).join('');
  };

  // === Form Handling ===
  AX.handleForm = function(formId, endpoint, callback) {
    const form = document.getElementById(formId);
    if (!form) return;
    form.addEventListener('submit', async function(e) {
      e.preventDefault();
      const formData = new FormData(form);
      const data = Object.fromEntries(formData.entries());
      const result = await AX.post(endpoint, data);
      if (result && callback) callback(result);
    });
  };

  // === Routing ===
  AX.routes = {};
  AX.route = function(path, handler) { AX.routes[path] = handler; };
  AX.navigate = function(path) {
    history.pushState(null, '', path);
    AX.resolve();
  };
  AX.resolve = function() {
    const path = location.pathname;
    const handler = AX.routes[path] || AX.routes['*'];
    if (handler) handler(path);
  };
  window.addEventListener('popstate', AX.resolve);

  // === Toast ===
  AX.toast = function(message, type) {
    type = type || 'info';
    const toast = document.createElement('div');
    toast.className = 'ax-toast ax-toast-' + type;
    toast.textContent = message;
    document.body.appendChild(toast);
    setTimeout(function() { toast.remove(); }, 3000);
  };

  // === Modal ===
  AX.openModal = function(id) {
    const modal = document.getElementById(id);
    if (modal) modal.setAttribute('open', '');
  };
  AX.closeModal = function(id) {
    const modal = document.getElementById(id);
    if (modal) modal.removeAttribute('open');
  };

  // === Init ===
  document.addEventListener('DOMContentLoaded', function() {
    AX.resolve();
    document.querySelectorAll('[data-fetch]').forEach(async function(el) {
      const endpoint = el.dataset.fetch;
      const data = await AX.get(endpoint);
      if (data) {
        if (el.dataset.template === 'table') {
          const columns = JSON.parse(el.dataset.columns || '[]');
          AX.renderTable('#' + el.id, data, columns);
        } else if (el.dataset.template === 'list') {
          AX.renderList('#' + el.id, data, function(item) {
            return '<li>' + (item.name || item.title || JSON.stringify(item)) + '</li>';
          });
        }
      }
    });
  });

})();
"#;

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
