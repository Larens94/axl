use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    String(String),
    Int(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub kind: u16,
    pub target: u32,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Property {
    pub id: u16,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: u16,
    pub action: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pub id: u32,
    pub component: u16,
    pub properties: Vec<Property>,
    pub events: Vec<Event>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub id: u32,
    pub root: Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
    pub annotations: Vec<Annotation>,
    pub views: Vec<View>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxlError(pub String);

impl Display for AxlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AxlError {}

pub fn parse(source: &str) -> Result<Program, AxlError> {
    let compact = remove_unquoted_whitespace(source)?;
    let frames = split_quoted(&compact, ';')?;
    if frames.first().map(String::as_str) != Some("3") {
        return Err(AxlError("AX-UI source requires version header '3'".into()));
    }
    let mut position = 1;
    let mut annotations = Vec::new();
    let mut views = Vec::new();
    while position < frames.len() {
        let fields = split_quoted(&frames[position], '|')?;
        let frame = position;
        position += 1;
        match fields.first().map(String::as_str) {
            Some("80") if fields.len() == 4 => annotations.push(Annotation {
                kind: number(&fields[1], frame, "annotation kind")?,
                target: number(&fields[2], frame, "annotation target")?,
                value: string_literal(&fields[3], frame)?,
            }),
            Some("60") if fields.len() == 2 => {
                let id = number(&fields[1], frame, "view id")?;
                let root = parse_node(&frames, &mut position)?;
                views.push(View { id, root });
            }
            Some(opcode) => {
                return Err(AxlError(format!(
                    "frame {frame}: invalid top-level opcode or arity '{opcode}'"
                )));
            }
            None => return Err(AxlError(format!("frame {frame}: empty frame"))),
        }
    }
    let program = Program { annotations, views };
    validate(&program)?;
    Ok(program)
}

fn parse_node(frames: &[String], position: &mut usize) -> Result<Node, AxlError> {
    if *position >= frames.len() {
        return Err(AxlError("UI view missing root node".into()));
    }
    let frame = *position;
    let fields = split_quoted(&frames[*position], '|')?;
    *position += 1;
    if fields.first().map(String::as_str) != Some("61") || fields.len() != 3 {
        return Err(AxlError(format!(
            "frame {frame}: UI view requires node opcode 61"
        )));
    }
    let mut node = Node {
        id: number(&fields[1], frame, "node id")?,
        component: number(&fields[2], frame, "component id")?,
        properties: Vec::new(),
        events: Vec::new(),
        children: Vec::new(),
    };
    while *position < frames.len() {
        let child_frame = *position;
        let child_fields = split_quoted(&frames[*position], '|')?;
        match child_fields.first().map(String::as_str) {
            Some("99") if child_fields.len() == 1 => {
                *position += 1;
                return Ok(node);
            }
            Some("62") if child_fields.len() == 3 => {
                node.properties.push(Property {
                    id: number(&child_fields[1], child_frame, "property id")?,
                    value: expression(&child_fields[2], child_frame)?,
                });
                *position += 1;
            }
            Some("63") if child_fields.len() == 3 => {
                node.events.push(Event {
                    id: number(&child_fields[1], child_frame, "event id")?,
                    action: number(&child_fields[2], child_frame, "action id")?,
                });
                *position += 1;
            }
            Some("61") => node.children.push(parse_node(frames, position)?),
            Some(opcode) => {
                return Err(AxlError(format!(
                    "frame {child_frame}: invalid UI opcode or arity '{opcode}'"
                )));
            }
            None => return Err(AxlError(format!("frame {child_frame}: empty frame"))),
        }
    }
    Err(AxlError(format!(
        "frame {frame}: UI node missing end opcode 99"
    )))
}

fn expression(source: &str, frame: usize) -> Result<Value, AxlError> {
    if source.starts_with('"') {
        return string_literal(source, frame).map(Value::String);
    }
    if let Some(value) = source.strip_prefix('#') {
        return value
            .parse::<i64>()
            .map(Value::Int)
            .map_err(|_| AxlError(format!("frame {frame}: invalid integer expression")));
    }
    match source {
        "?0" => Ok(Value::Bool(false)),
        "?1" => Ok(Value::Bool(true)),
        _ => Err(AxlError(format!(
            "frame {frame}: UI properties require literal expressions"
        ))),
    }
}

fn string_literal(source: &str, frame: usize) -> Result<String, AxlError> {
    if !source.starts_with('"') || !source.ends_with('"') || source.len() < 2 {
        return Err(AxlError(format!("frame {frame}: expected string literal")));
    }
    let mut result = String::new();
    let mut escaped = false;
    for character in source[1..source.len() - 1].chars() {
        if escaped {
            result.push(match character {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                _ => return Err(AxlError(format!("frame {frame}: invalid string escape"))),
            });
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else {
            result.push(character);
        }
    }
    if escaped {
        return Err(AxlError(format!("frame {frame}: incomplete string escape")));
    }
    Ok(result)
}

fn number<T>(source: &str, frame: usize, label: &str) -> Result<T, AxlError>
where
    T: std::str::FromStr,
{
    source
        .parse()
        .map_err(|_| AxlError(format!("frame {frame}: invalid {label}")))
}

fn split_quoted(source: &str, delimiter: char) -> Result<Vec<String>, AxlError> {
    let mut values = Vec::new();
    let mut value = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in source.chars() {
        if escaped {
            value.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quoted {
            value.push(character);
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            value.push(character);
            continue;
        }
        if character == delimiter && !quoted {
            values.push(value);
            value = String::new();
        } else {
            value.push(character);
        }
    }
    if quoted || escaped {
        return Err(AxlError("unterminated string".into()));
    }
    values.push(value);
    Ok(values)
}

fn remove_unquoted_whitespace(source: &str) -> Result<String, AxlError> {
    let mut result = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for character in source.chars() {
        if escaped {
            result.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' && quoted {
            result.push(character);
            escaped = true;
            continue;
        }
        if character == '"' {
            quoted = !quoted;
            result.push(character);
        } else if quoted || !character.is_whitespace() {
            result.push(character);
        }
    }
    if quoted || escaped {
        return Err(AxlError("unterminated string".into()));
    }
    Ok(result)
}

pub fn validate(program: &Program) -> Result<(), AxlError> {
    if program.views.len() != 1 {
        return Err(AxlError("application requires exactly one UI view".into()));
    }
    let mut annotation_keys = HashSet::new();
    for item in &program.annotations {
        if !(1..=3).contains(&item.kind) {
            return Err(AxlError(format!("unknown annotation kind '{}'", item.kind)));
        }
        if item.target == 0 || item.value.is_empty() {
            return Err(AxlError("invalid annotation".into()));
        }
        annotation_keys.insert((item.kind, item.target, item.value.as_str()));
    }
    let view = &program.views[0];
    if view.id == 0 || view.root.component != 1 {
        return Err(AxlError("view root must use app component 1".into()));
    }
    let mut ids = HashSet::new();
    validate_node(&view.root, &mut ids)
}

fn validate_node(node: &Node, ids: &mut HashSet<u32>) -> Result<(), AxlError> {
    if node.id == 0 || !ids.insert(node.id) {
        return Err(AxlError(format!(
            "invalid or duplicate UI node id '{}'",
            node.id
        )));
    }
    let (properties, events, children) = match node.component {
        1 => (&[(1, "string")][..], &[][..], true),
        2 => (
            &[
                (1, "string"),
                (2, "string"),
                (3, "string"),
                (4, "string"),
                (5, "string"),
            ][..],
            &[1, 2][..],
            false,
        ),
        3 => (&[(1, "string")][..], &[][..], true),
        4 => (
            &[(1, "string"), (2, "string"), (3, "int"), (4, "int")][..],
            &[1][..],
            false,
        ),
        _ => {
            return Err(AxlError(format!(
                "unknown UI component '{}'",
                node.component
            )));
        }
    };
    let mut property_ids = HashSet::new();
    for property in &node.properties {
        if !property_ids.insert(property.id) {
            return Err(AxlError(format!(
                "duplicate property '{}' on node '{}'",
                property.id, node.id
            )));
        }
        let expected = properties
            .iter()
            .find(|(id, _)| *id == property.id)
            .map(|(_, kind)| *kind)
            .ok_or_else(|| {
                AxlError(format!(
                    "property '{}' is invalid for component '{}'",
                    property.id, node.component
                ))
            })?;
        let actual = match property.value {
            Value::String(_) => "string",
            Value::Int(_) => "int",
            Value::Bool(_) => "bool",
        };
        if actual != expected {
            return Err(AxlError(format!(
                "property '{}' requires {expected}, got {actual}",
                property.id
            )));
        }
    }
    for (id, _) in properties {
        if !property_ids.contains(id) {
            return Err(AxlError(format!(
                "component '{}' requires property '{id}'",
                node.component
            )));
        }
    }
    let mut event_ids = HashSet::new();
    for event in &node.events {
        if !events.contains(&event.id) || event.action == 0 || !event_ids.insert(event.id) {
            return Err(AxlError(format!(
                "invalid event '{}' on node '{}'",
                event.id, node.id
            )));
        }
    }
    if !children && !node.children.is_empty() {
        return Err(AxlError(format!(
            "component '{}' cannot have children",
            node.component
        )));
    }
    for child in &node.children {
        validate_node(child, ids)?;
    }
    Ok(())
}

pub fn build_web(program: &Program, output: &Path) -> Result<(), AxlError> {
    validate(program)?;
    fs::create_dir_all(output).map_err(io_error)?;
    fs::write(
        output.join("index.html"),
        render_document(&program.views[0].root),
    )
    .map_err(io_error)?;
    fs::write(output.join("ax-ui.css"), CSS).map_err(io_error)?;
    fs::write(output.join("ax-ui.js"), JS).map_err(io_error)?;
    Ok(())
}

fn io_error(error: std::io::Error) -> AxlError {
    AxlError(error.to_string())
}

fn property<'a>(node: &'a Node, id: u16) -> &'a Value {
    &node
        .properties
        .iter()
        .find(|item| item.id == id)
        .expect("validated property")
        .value
}
fn text(node: &Node, id: u16) -> String {
    match property(node, id) {
        Value::String(value) => escape_html(value),
        _ => unreachable!(),
    }
}
fn integer(node: &Node, id: u16) -> i64 {
    match property(node, id) {
        Value::Int(value) => *value,
        _ => unreachable!(),
    }
}

fn render_document(root: &Node) -> String {
    let title = text(root, 1);
    let content: String = root.children.iter().map(render_node).collect();
    format!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>{title}</title><link rel=\"stylesheet\" href=\"ax-ui.css\"></head><body><header class=\"ax-nav\"><strong>{title}</strong><nav><a href=\"#home\">Home</a><a href=\"#catalogue\">Series</a><a href=\"#catalogue\">Films</a><a href=\"#catalogue\">New &amp; Popular</a></nav><div><button data-search aria-label=\"Search\">⌕</button><button class=\"ax-avatar\" aria-label=\"Profile\">AX</button></div></header><main>{content}</main><div class=\"ax-toast\" role=\"status\" hidden></div><script src=\"ax-ui.js\"></script></body></html>"
    )
}

fn render_node(node: &Node) -> String {
    match node.component {
        2 => {
            let image = match property(node, 5) {
                Value::String(value) if safe_url(value) => escape_html(value),
                _ => String::new(),
            };
            let buttons: String = node
                .events
                .iter()
                .map(|event| {
                    format!(
                        "<button class=\"{}\" data-action=\"{}\">{}</button>",
                        if event.id == 1 { "ax-play" } else { "ax-more" },
                        event.action,
                        if event.id == 1 {
                            "▶ Play"
                        } else {
                            "ⓘ More Info"
                        }
                    )
                })
                .collect();
            format!(
                "<section id=\"home\" class=\"ax-hero\" style=\"--ax-hero:url(&quot;{image}&quot;)\"><div class=\"ax-hero-copy\"><span>{}</span><h1>{}</h1><b>{}</b><p>{}</p><div class=\"ax-actions\">{buttons}</div></div></section>",
                text(node, 2),
                text(node, 1),
                text(node, 4),
                text(node, 3)
            )
        }
        3 => format!(
            "<section id=\"catalogue\" class=\"ax-shelf\"><h2>{}</h2><div class=\"ax-rail\">{}</div></section>",
            text(node, 1),
            node.children.iter().map(render_node).collect::<String>()
        ),
        4 => {
            let tone = integer(node, 3).clamp(1, 10);
            let size = integer(node, 4).clamp(1, 2);
            let action = node
                .events
                .first()
                .map(|event| event.action)
                .unwrap_or(node.id);
            format!(
                "<button class=\"ax-card ax-tone-{tone} ax-size-{size}\" data-action=\"{action}\"><span class=\"ax-rank\">{}</span><strong>{}</strong><small>{}</small><i>▶</i></button>",
                node.id,
                text(node, 1),
                text(node, 2)
            )
        }
        _ => String::new(),
    }
}

fn safe_url(value: &str) -> bool {
    value.starts_with("https://")
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".:/_?&=%+-".contains(character))
}
fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

const JS: &str = "const t=document.querySelector('.ax-toast');document.querySelectorAll('[data-action]').forEach(n=>n.addEventListener('click',()=>{t.textContent=`Action ${n.dataset.action} executed`;t.hidden=false;clearTimeout(window.axT);window.axT=setTimeout(()=>t.hidden=true,2200)}));document.querySelector('[data-search]')?.addEventListener('click',()=>{const q=prompt('Search titles, people, genres');if(q){t.textContent=`Searching for ${q}`;t.hidden=false}});";
const CSS: &str = include_str!("ax-ui.css");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_numeric_ui() {
        let program = parse("3;80|1|1|\"demo\";60|1;61|1|1;62|1|\"AX\";61|2|3;62|1|\"Row\";61|3|4;62|1|\"Film\";62|2|\"New\";62|3|#1;62|4|#1;63|1|3;99;99;99").unwrap();
        assert_eq!(program.views[0].root.children[0].children[0].id, 3);
    }

    #[test]
    fn rejects_wrong_property_type() {
        let error = parse("3;60|1;61|1|1;62|1|#1;99").unwrap_err();
        assert!(error.0.contains("requires string"));
    }

    #[test]
    fn delimiters_inside_strings_are_not_structural() {
        let program = parse("3;80|1|1|\"a;b|c\";60|1;61|1|1;62|1|\"AX\";99").unwrap();
        assert_eq!(program.annotations[0].value, "a;b|c");
    }

    #[test]
    fn line_breaks_between_frames_are_non_structural() {
        let source = "3;\n 80|1|1|\"spaces stay inside\";\n 60|1;\n 61|1|1;\n 62|1|\"AX\";\n 99\n";
        let program = parse(source).unwrap();
        assert_eq!(program.views[0].root.id, 1);
    }
}
