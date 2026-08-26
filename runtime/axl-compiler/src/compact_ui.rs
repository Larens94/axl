use anyhow::{bail, Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum UiValue {
    String(String),
    Integer(i64),
    Boolean(bool),
    Binding(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiProperty {
    pub id: i32,
    pub value: UiValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiEvent {
    pub id: i32,
    pub action_id: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiNode {
    pub id: i32,
    pub component_id: i32,
    pub properties: Vec<UiProperty>,
    pub events: Vec<UiEvent>,
    pub children: Vec<UiNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UiView {
    pub id: i32,
    pub root: UiNode,
}

pub fn parse_file(path: &std::path::Path) -> Result<Vec<UiView>> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read compact UI '{}'", path.display()))?;
    parse_source(&source)
}

pub fn parse_source(source: &str) -> Result<Vec<UiView>> {
    let frames = split_frames(source)?;
    let Some(version) = frames.first() else { bail!("compact UI is empty"); };
    if version != "3" { bail!("compact UI requires version 3, got '{version}'"); }
    let mut position = 1;
    let mut views = Vec::new();
    while position < frames.len() {
        let fields = split_fields(&frames[position])?;
        if fields.len() != 2 || fields[0] != "60" {
            bail!("frame {position}: compact UI expects view opcode 60");
        }
        let id = fields[1].parse().with_context(|| format!("frame {position}: invalid view id"))?;
        position += 1;
        let (root, next) = parse_node(&frames, position)?;
        position = next;
        views.push(UiView { id, root });
    }
    if views.is_empty() { bail!("compact UI requires at least one view"); }
    Ok(views)
}

fn parse_node(frames: &[String], position: usize) -> Result<(UiNode, usize)> {
    let fields = frames.get(position).map(|frame| split_fields(frame)).transpose()?
        .ok_or_else(|| anyhow::anyhow!("compact UI is missing a node"))?;
    if fields.len() != 3 || fields[0] != "61" { bail!("frame {position}: expected node opcode 61"); }
    let id = fields[1].parse().with_context(|| format!("frame {position}: invalid node id"))?;
    let component_id = fields[2].parse().with_context(|| format!("frame {position}: invalid component id"))?;
    let mut properties = Vec::new();
    let mut events = Vec::new();
    let mut children = Vec::new();
    let mut cursor = position + 1;
    while cursor < frames.len() {
        let child = split_fields(&frames[cursor])?;
        match child.first().map(String::as_str) {
            Some("99") if child.len() == 1 => return Ok((UiNode { id, component_id, properties, events, children }, cursor + 1)),
            Some("62") if child.len() == 3 => {
                let id = child[1].parse().with_context(|| format!("frame {cursor}: invalid property id"))?;
                properties.push(UiProperty { id, value: parse_value(&child[2])? });
                cursor += 1;
            }
            Some("63") if child.len() == 3 => {
                events.push(UiEvent {
                    id: child[1].parse().with_context(|| format!("frame {cursor}: invalid event id"))?,
                    action_id: child[2].parse().with_context(|| format!("frame {cursor}: invalid action id"))?,
                });
                cursor += 1;
            }
            Some("61") => {
                let (node, next) = parse_node(frames, cursor)?;
                children.push(node);
                cursor = next;
            }
            _ => bail!("frame {cursor}: invalid compact UI opcode"),
        }
    }
    bail!("frame {position}: UI node is missing end opcode 99")
}

fn parse_value(source: &str) -> Result<UiValue> {
    if let Some(value) = source.strip_prefix('$') { return Ok(UiValue::Binding(value.to_string())); }
    if let Some(value) = source.strip_prefix('#') { return Ok(UiValue::Integer(value.parse()?)); }
    if source == "?1" { return Ok(UiValue::Boolean(true)); }
    if source == "?0" { return Ok(UiValue::Boolean(false)); }
    Ok(UiValue::String(serde_json::from_str(source).context("UI string properties use JSON quoting")?))
}

fn split_frames(source: &str) -> Result<Vec<String>> {
    split_quoted(source, ';').map(|items| items.into_iter().filter(|item| !item.trim().is_empty()).map(|item| item.trim().to_string()).collect())
}

fn split_fields(source: &str) -> Result<Vec<String>> { split_quoted(source, '|') }

fn split_quoted(source: &str, delimiter: char) -> Result<Vec<String>> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut escaped = false;
    for ch in source.chars() {
        if escaped { current.push(ch); escaped = false; continue; }
        if quoted && ch == '\\' { current.push(ch); escaped = true; continue; }
        if ch == '"' { quoted = !quoted; current.push(ch); continue; }
        if ch == delimiter && !quoted { result.push(std::mem::take(&mut current)); } else { current.push(ch); }
    }
    if quoted || escaped { bail!("unterminated quoted value in compact UI"); }
    result.push(current);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nested_compact_ui() {
        let views = parse_source("3;60|1;61|1|1;62|1|\"CRM\";61|2|11;62|1|#4;61|3|33;62|1|$customers.total;99;99;99").unwrap();
        assert_eq!(views[0].root.children[0].children[0].properties[0].value, UiValue::Binding("customers.total".into()));
    }
}
