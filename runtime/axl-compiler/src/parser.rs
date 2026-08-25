use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AxlApp {
    pub name: String,
    pub entities: Vec<Entity>,
    pub apis: Vec<Api>,
    pub auth: Option<Auth>,
    pub ui: Vec<UiComponent>,
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
    };
    
    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    
    while i < lines.len() {
        let line = lines[i].trim();
        
        if line.starts_with("entity ") {
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
        } else {
            i += 1;
        }
    }
    
    Ok(app)
}

fn parse_entity(lines: &[&str], i: &mut usize) -> Result<Entity> {
    let line = lines[*i].trim();
    let name = line.strip_prefix("entity ").unwrap().trim()
        .trim_end_matches('{')
        .trim()
        .to_string();
    *i += 1;
    
    let mut fields = Vec::new();
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            break;
        }
        
        if line.starts_with("field ") {
            let field = parse_field(line)?;
            fields.push(field);
        }
        
        *i += 1;
    }
    
    Ok(Entity { name, fields })
}

fn parse_field(line: &str) -> Result<Field> {
    let content = line.strip_prefix("field ").unwrap().trim();
    let parts: Vec<&str> = content.split(':').collect();
    
    let name = parts[0].trim().to_string();
    let type_part = parts[1].trim();
    
    let (field_type, optional, default) = if type_part.ends_with('?') {
        (type_part[..type_part.len()-1].to_string(), true, None)
    } else if type_part.contains('=') {
        let type_default: Vec<&str> = type_part.split('=').collect();
        let default_val = type_default[1].trim().trim_matches('"').to_string();
        (type_default[0].trim().to_string(), false, Some(default_val))
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
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            break;
        }
        
        if line.contains("→") || line.contains("->") {
            let route = parse_route(line)?;
            routes.push(route);
        }
        
        *i += 1;
    }
    
    Ok(Api { entity, routes })
}

fn parse_route(line: &str) -> Result<Route> {
    let separator = if line.contains('→') { "→" } else { "->" };
    let parts: Vec<&str> = line.split(separator).collect();
    
    let method_path = parts[0].trim();
    let handler = parts[1].trim().to_string();
    
    let method_path_parts: Vec<&str> = method_path.split_whitespace().collect();
    let method = method_path_parts[0].to_string();
    let path = method_path_parts[1].to_string();
    
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
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
            break;
        }
        
        if line.contains("login") {
            auth.login_route = Some(line.to_string());
        } else if line.contains("register") {
            auth.register_route = Some(line.to_string());
        } else if line.starts_with("middleware:") {
            let middleware = line.strip_prefix("middleware:").unwrap().trim();
            auth.middleware = middleware.split(',').map(|s| s.trim().to_string()).collect();
        }
        
        *i += 1;
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
    
    while *i < lines.len() {
        let line = lines[*i].trim();
        
        if line == "}" {
            *i += 1;
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
        }
        
        *i += 1;
    }
    
    Ok(component)
}
