use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxlApp {
    pub name: String,
    pub entities: Vec<Entity>,
    pub apis: Vec<Api>,
    pub auth: Option<Auth>,
    pub ui: Vec<UiComponent>,
    pub seeds: Vec<Seed>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seed {
    pub entity: String,
    pub values: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub optional: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Api {
    pub entity: String,
    pub routes: Vec<Route>,
    pub query: Option<QueryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPolicy {
    pub page_size: usize,
    pub max_page_size: usize,
    pub sort_field: String,
    pub sort_direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Auth {
    pub login_route: Option<String>,
    pub register_route: Option<String>,
    pub middleware: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiComponent {
    pub name: String,
    pub component_type: String,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: String,
}

pub fn parse_file(path: &Path) -> Result<AxlApp> {
    let content = std::fs::read_to_string(path)?;
    parse_source(&content)
}

pub fn parse_source(source: &str) -> Result<AxlApp> {
    let mut app = AxlApp {
        name: "app".to_string(),
        entities: Vec::new(),
        apis: Vec::new(),
        auth: None,
        ui: Vec::new(),
        seeds: Vec::new(),
    };
    
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            i += 1;
        } else if line.starts_with("entity ") {
            let entity = parse_entity(&lines, &mut i)?;
            app.entities.push(entity);
        } else if line.starts_with("api ") {
            let api = parse_api(&lines, &mut i)?;
            app.apis.push(api);
        } else if line.starts_with("auth") {
            let auth = parse_auth(&lines, &mut i)?;
            app.auth = Some(auth);
        } else if line.starts_with("ui ") {
            let ui = parse_ui(&lines, &mut i)?;
            app.ui.push(ui);
        } else if line.starts_with("seed ") {
            let seed = parse_seed(&lines, &mut i)?;
            app.seeds.push(seed);
        } else {
            bail!("line {}: unknown AXL declaration: {line}", i + 1);
        }
    }

    if app.entities.is_empty() {
        bail!("an AXL application requires at least one entity");
    }

    Ok(app)
}

fn parse_seed(lines: &[&str], i: &mut usize) -> Result<Seed> {
    let line = lines[*i].trim();
    let entity = line.strip_prefix("seed ").unwrap().trim()
        .trim_end_matches('{').trim().to_string();
    if entity.is_empty() {
        bail!("line {}: seed entity is required", *i + 1);
    }
    *i += 1;
    let mut values = Vec::new();
    let mut closed = false;
    while *i < lines.len() {
        let line = lines[*i].trim();
        if line == "}" {
            *i += 1;
            closed = true;
            break;
        }
        if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            let (name, value) = line.split_once(':')
                .ok_or_else(|| anyhow::anyhow!("line {}: seed value requires 'field: value'", *i + 1))?;
            let name = name.trim();
            let value = value.trim();
            if name.is_empty() || value.is_empty() {
                bail!("line {}: seed field and value cannot be empty", *i + 1);
            }
            values.push(Property { name: name.to_string(), value: value.trim_matches('"').to_string() });
        }
        *i += 1;
    }
    if !closed { bail!("seed {entity} is missing closing '}}'"); }
    Ok(Seed { entity, values })
}

fn parse_entity(lines: &[&str], i: &mut usize) -> Result<Entity> {
    let line = lines[*i].trim();
    let name = line.strip_prefix("entity ").unwrap().trim()
        .trim_end_matches('{')
        .trim()
        .to_string();
    *i += 1;
    
    if name.is_empty() {
        bail!("line {}: entity name is required", *i + 1);
    }

    let mut fields = Vec::new();
    let mut closed = false;
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            closed = true;
            break;
        }
        
        if line.starts_with("field ") {
            let field = parse_field(line)
                .with_context(|| format!("line {} in entity {name}", *i + 1))?;
            fields.push(field);
        } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            bail!("line {}: expected a field declaration or '}}'", *i + 1);
        }
        
        *i += 1;
    }
    
    if !closed {
        bail!("entity {name} is missing closing '}}'");
    }
    Ok(Entity { name, fields })
}

fn parse_field(line: &str) -> Result<Field> {
    let content = line.strip_prefix("field ").unwrap().trim();
    let (name, type_part) = content
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("field requires 'name: Type'"))?;
    let name = name.trim().to_string();
    let type_part = type_part.trim();
    if name.is_empty() || type_part.is_empty() {
        bail!("field name and type cannot be empty");
    }
    
    let (field_type, optional, default) = if type_part.ends_with('?') {
        (type_part[..type_part.len()-1].to_string(), true, None)
    } else if type_part.contains('=') {
        let (field_type, default) = type_part.split_once('=').expect("contains '='");
        let default_val = default.trim().trim_matches('"').to_string();
        (field_type.trim().to_string(), false, Some(default_val))
    } else {
        (type_part.to_string(), false, None)
    };
    
    Ok(Field {
        name,
        field_type,
        optional,
        default,
    })
}

fn parse_api(lines: &[&str], i: &mut usize) -> Result<Api> {
    let line = lines[*i].trim();
    let entity = line.strip_prefix("api ").unwrap().trim()
        .trim_end_matches('{')
        .trim()
        .to_string();
    *i += 1;
    
    let mut routes = Vec::new();
    let mut query = None;
    let mut closed = false;
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            closed = true;
            break;
        }
        
        if line.starts_with("query ") {
            if query.is_some() {
                bail!("line {}: duplicate query policy in api {entity}", *i + 1);
            }
            query = Some(parse_query_policy(line)
                .with_context(|| format!("line {} in api {entity}", *i + 1))?);
        } else if line.contains("→") || line.contains("->") {
            let route = parse_route(line)
                .with_context(|| format!("line {} in api {entity}", *i + 1))?;
            routes.push(route);
        } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            bail!("line {}: expected an API route or '}}'", *i + 1);
        }
        
        *i += 1;
    }
    
    if !closed {
        bail!("api {entity} is missing closing '}}'");
    }
    Ok(Api { entity, routes, query })
}

fn parse_query_policy(line: &str) -> Result<QueryPolicy> {
    let parts = line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 8 || parts[0] != "query" || parts[1] != "page" || parts[3] != "max" || parts[5] != "sort" {
        bail!("query policy requires 'query page <n> max <n> sort <field> <asc|desc>'");
    }
    let page_size = parts[2].parse::<usize>().context("page size must be an integer")?;
    let max_page_size = parts[4].parse::<usize>().context("maximum page size must be an integer")?;
    Ok(QueryPolicy {
        page_size,
        max_page_size,
        sort_field: parts[6].to_string(),
        sort_direction: parts[7].to_string(),
    })
}

fn parse_route(line: &str) -> Result<Route> {
    let separator = if line.contains('→') { "→" } else { "->" };
    let (method_path, handler) = line
        .split_once(separator)
        .ok_or_else(|| anyhow::anyhow!("route requires 'METHOD /path -> handler'"))?;
    let method_path = method_path.trim();
    let handler = handler.trim().to_string();
    let method_path_parts: Vec<&str> = method_path.split_whitespace().collect();
    if method_path_parts.len() != 2 || handler.is_empty() {
        bail!("route requires 'METHOD /path -> handler'");
    }
    let method = method_path_parts[0].to_string();
    let path = method_path_parts[1].to_string();
    if !matches!(method.as_str(), "GET" | "POST" | "PUT" | "PATCH" | "DELETE") {
        bail!("unsupported HTTP method '{method}'");
    }
    if !path.starts_with('/') {
        bail!("route path must start with '/'");
    }
    
    Ok(Route {
        method,
        path,
        handler,
    })
}

fn parse_auth(lines: &[&str], i: &mut usize) -> Result<Auth> {
    *i += 1; // Skip "auth {"
    
    let mut auth = Auth {
        login_route: None,
        register_route: None,
        middleware: Vec::new(),
    };
    
    let mut closed = false;
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            closed = true;
            break;
        }
        
        if line.contains("login") {
            auth.login_route = Some(line.to_string());
        } else if line.contains("register") {
            auth.register_route = Some(line.to_string());
        } else if line.starts_with("middleware:") {
            let middleware = line.strip_prefix("middleware:").unwrap().trim();
            auth.middleware = middleware.split(',').map(|s| s.trim().to_string()).collect();
        } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            bail!("line {}: unknown auth declaration", *i + 1);
        }
        
        *i += 1;
    }
    
    if !closed {
        bail!("auth block is missing closing '}}'");
    }
    Ok(auth)
}

fn parse_ui(lines: &[&str], i: &mut usize) -> Result<UiComponent> {
    let line = lines[*i].trim();
    let name = line.strip_prefix("ui ").unwrap().trim()
        .trim_end_matches('{')
        .trim()
        .to_string();
    *i += 1;
    
    let mut component = UiComponent {
        name,
        component_type: "page".to_string(),
        properties: Vec::new(),
    };
    
    let mut closed = false;
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            closed = true;
            break;
        }
        
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            let prop_name = parts[0].trim().to_string();
            let prop_value = parts[1].trim().to_string();
            component.properties.push(Property {
                name: prop_name,
                value: prop_value,
            });
        } else if !line.is_empty() && !line.starts_with("//") && !line.starts_with('#') {
            bail!("line {}: expected a UI property or '}}'", *i + 1);
        }
        
        *i += 1;
    }
    
    if !closed {
        bail!("ui {} is missing closing '}}'", component.name);
    }
    Ok(component)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_stack_application() {
        let source = r#"
            entity Customer {
              field name: String
              field active: Boolean = true
            }
            api Customer {
              GET /api/customers -> list
            }
            ui Customers {
              table: Customer[name, active]
            }
        "#;
        let app = parse_source(source).unwrap();
        assert_eq!(app.entities.len(), 1);
        assert_eq!(app.apis[0].routes[0].handler, "list");
        assert_eq!(app.ui.len(), 1);
    }

    #[test]
    fn parses_api_query_policy() {
        let app = parse_source("entity Customer {\nfield name: String\n}\napi Customer {\nquery page 25 max 100 sort created_at desc\nGET /customers -> list\n}").unwrap();
        let query = app.apis[0].query.as_ref().unwrap();
        assert_eq!(query.page_size, 25);
        assert_eq!(query.max_page_size, 100);
        assert_eq!(query.sort_field, "created_at");
    }

    #[test]
    fn rejects_unknown_top_level_declarations() {
        let error = parse_source("magic Thing {}").unwrap_err();
        assert!(error.to_string().contains("unknown AXL declaration"));
    }

    #[test]
    fn rejects_unclosed_blocks() {
        let error = parse_source("entity Customer {\nfield name: String").unwrap_err();
        assert!(error.to_string().contains("missing closing"));
    }
}
