use std::collections::HashSet;

use crate::parser::AxlApp;
use anyhow::{bail, Result};

pub struct AnalyzedApp {
    pub name: String,
    pub entities: Vec<AnalyzedEntity>,
    pub apis: Vec<AnalyzedApi>,
    pub auth: Option<AnalyzedAuth>,
    pub ui: Vec<AnalyzedUi>,
    pub seeds: Vec<AnalyzedSeed>,
}

pub struct AnalyzedSeed {
    pub entity: String,
    pub values: Vec<(String, String)>,
}

pub struct AnalyzedEntity {
    pub name: String,
    pub table_name: String,
    pub fields: Vec<AnalyzedField>,
    pub has_timestamps: bool,
    pub has_soft_delete: bool,
}

pub struct AnalyzedField {
    pub name: String,
    pub field_type: String,
    pub rust_type: String,
    pub ts_type: String,
    pub optional: bool,
    pub default: Option<String>,
    pub is_primary_key: bool,
    pub is_foreign_key: bool,
}

pub struct AnalyzedApi {
    pub entity: String,
    pub routes: Vec<AnalyzedRoute>,
    pub query: Option<AnalyzedQueryPolicy>,
}

pub struct AnalyzedQueryPolicy {
    pub page_size: usize,
    pub max_page_size: usize,
    pub sort_field: String,
    pub descending: bool,
}

pub struct AnalyzedRoute {
    pub method: String,
    pub path: String,
    pub handler: String,
    pub handler_fn: String,
}

pub struct AnalyzedAuth {
    pub login_route: Option<String>,
    pub register_route: Option<String>,
    pub middleware: Vec<String>,
}

pub struct AnalyzedUi {
    pub name: String,
    pub component_type: String,
    pub properties: Vec<(String, String)>,
}

pub fn analyze(app: AxlApp) -> Result<AnalyzedApp> {
    let mut entity_names = HashSet::new();
    for entity in &app.entities {
        if !entity_names.insert(entity.name.clone()) {
            bail!("duplicate entity '{}'", entity.name);
        }
        let mut field_names = HashSet::new();
        for field in &entity.fields {
            if !field_names.insert(field.name.clone()) {
                bail!("duplicate field '{}.{}'", entity.name, field.name);
            }
            if !matches!(
                field.field_type.as_str(),
                "String"
                    | "Integer"
                    | "Int"
                    | "Boolean"
                    | "Bool"
                    | "Float"
                    | "Double"
                    | "DateTime"
            )
            {
                bail!("unsupported AXL type '{}' on '{}.{}'", field.field_type, entity.name, field.name);
            }
        }
    }
    for api in &app.apis {
        if !entity_names.contains(&api.entity) {
            bail!("api '{}' references an unknown entity", api.entity);
        }
        if let Some(query) = &api.query {
            if query.page_size == 0 || query.max_page_size == 0 {
                bail!("query page sizes for '{}' must be greater than zero", api.entity);
            }
            if query.page_size > query.max_page_size {
                bail!("query default page size for '{}' cannot exceed its maximum", api.entity);
            }
            if query.max_page_size > 1000 {
                bail!("query maximum page size for '{}' cannot exceed 1000", api.entity);
            }
            if !matches!(query.sort_direction.as_str(), "asc" | "desc") {
                bail!("query sort direction for '{}' must be 'asc' or 'desc'", api.entity);
            }
            let entity = app.entities.iter().find(|entity| entity.name == api.entity).unwrap();
            let generated = matches!(query.sort_field.as_str(), "id" | "created_at" | "updated_at");
            if !generated && !entity.fields.iter().any(|field| field.name == query.sort_field) {
                bail!("query for '{}' sorts by unknown field '{}'", api.entity, query.sort_field);
            }
            if !api.routes.iter().any(|route| route.method == "GET" && route.handler == "list") {
                bail!("query policy for '{}' requires a GET list route", api.entity);
            }
        }
    }
    for seed in &app.seeds {
        let entity = app.entities.iter().find(|entity| entity.name == seed.entity)
            .ok_or_else(|| anyhow::anyhow!("seed references unknown entity '{}'", seed.entity))?;
        let mut names = HashSet::new();
        for value in &seed.values {
            if !names.insert(value.name.clone()) {
                bail!("duplicate seed field '{}.{}'", seed.entity, value.name);
            }
            if !entity.fields.iter().any(|field| field.name == value.name) {
                bail!("seed for '{}' references unknown field '{}'", seed.entity, value.name);
            }
        }
        for field in &entity.fields {
            if !field.optional && field.default.is_none() && !seed.values.iter().any(|value| value.name == field.name) {
                bail!("seed for '{}' is missing required field '{}'", seed.entity, field.name);
            }
        }
    }

    let mut analyzed = AnalyzedApp {
        name: app.name,
        entities: Vec::new(),
        apis: Vec::new(),
        auth: None,
        ui: Vec::new(),
        seeds: Vec::new(),
    };
    
    // Analyze entities
    for entity in &app.entities {
        let analyzed_entity = analyze_entity(entity)?;
        analyzed.entities.push(analyzed_entity);
    }
    
    // Analyze APIs
    for api in &app.apis {
        let analyzed_api = analyze_api(api)?;
        analyzed.apis.push(analyzed_api);
    }
    
    // Analyze auth
    if let Some(auth) = &app.auth {
        analyzed.auth = Some(AnalyzedAuth {
            login_route: auth.login_route.clone(),
            register_route: auth.register_route.clone(),
            middleware: auth.middleware.clone(),
        });
    }
    
    // Analyze UI
    for ui in &app.ui {
        analyzed.ui.push(AnalyzedUi {
            name: ui.name.clone(),
            component_type: ui.component_type.clone(),
            properties: ui.properties.iter().map(|p| (p.name.clone(), p.value.clone())).collect(),
        });
    }
    for seed in &app.seeds {
        analyzed.seeds.push(AnalyzedSeed {
            entity: seed.entity.clone(),
            values: seed.values.iter().map(|value| (value.name.clone(), value.value.clone())).collect(),
        });
    }
    
    Ok(analyzed)
}

fn pluralize(word: &str) -> String {
    let lower = word.to_lowercase();
    if lower.ends_with('y') && !lower.ends_with("ay") && !lower.ends_with("ey") && !lower.ends_with("oy") && !lower.ends_with("uy") {
        format!("{}ies", &lower[..lower.len()-1])
    } else if lower.ends_with('s') || lower.ends_with("sh") || lower.ends_with("ch") || lower.ends_with("x") || lower.ends_with("z") {
        format!("{}es", lower)
    } else {
        format!("{}s", lower)
    }
}

fn analyze_entity(entity: &crate::parser::Entity) -> Result<AnalyzedEntity> {
    let table_name = pluralize(&entity.name);
    
    let mut fields = Vec::new();
    
    // Add ID field
    fields.push(AnalyzedField {
        name: "id".to_string(),
        field_type: "Integer".to_string(),
        rust_type: "i32".to_string(),
        ts_type: "number".to_string(),
        optional: false,
        default: None,
        is_primary_key: true,
        is_foreign_key: false,
    });
    
    // Analyze user-defined fields
    for field in &entity.fields {
        let (rust_type, ts_type) = map_type(&field.field_type);
        fields.push(AnalyzedField {
            name: field.name.clone(),
            field_type: field.field_type.clone(),
            rust_type,
            ts_type,
            optional: field.optional,
            default: field.default.clone(),
            is_primary_key: false,
            is_foreign_key: field.name.ends_with("_id"),
        });
    }
    
    // Add timestamp fields
    fields.push(AnalyzedField {
        name: "created_at".to_string(),
        field_type: "DateTime".to_string(),
        rust_type: "chrono::NaiveDateTime".to_string(),
        ts_type: "string".to_string(),
        optional: false,
        default: Some("CURRENT_TIMESTAMP".to_string()),
        is_primary_key: false,
        is_foreign_key: false,
    });
    
    fields.push(AnalyzedField {
        name: "updated_at".to_string(),
        field_type: "DateTime".to_string(),
        rust_type: "chrono::NaiveDateTime".to_string(),
        ts_type: "string".to_string(),
        optional: false,
        default: Some("CURRENT_TIMESTAMP".to_string()),
        is_primary_key: false,
        is_foreign_key: false,
    });
    
    Ok(AnalyzedEntity {
        name: entity.name.clone(),
        table_name,
        fields,
        has_timestamps: true,
        has_soft_delete: false,
    })
}

fn map_type(axl_type: &str) -> (String, String) {
    match axl_type {
        "String" => ("String".to_string(), "string".to_string()),
        "Integer" | "Int" => ("i32".to_string(), "number".to_string()),
        "Boolean" | "Bool" => ("bool".to_string(), "boolean".to_string()),
        "Float" | "Double" => ("f64".to_string(), "number".to_string()),
        "DateTime" => ("DateTime".to_string(), "string".to_string()),
        _ => ("String".to_string(), "string".to_string()),
    }
}

fn analyze_api(api: &crate::parser::Api) -> Result<AnalyzedApi> {
    let mut routes = Vec::new();
    
    for route in &api.routes {
        let handler_fn = format!("{}_{}", route.handler, api.entity.to_lowercase());
        routes.push(AnalyzedRoute {
            method: route.method.clone(),
            path: route.path.clone(),
            handler: route.handler.clone(),
            handler_fn,
        });
    }
    
    Ok(AnalyzedApi {
        entity: api.entity.clone(),
        routes,
        query: api.query.as_ref().map(|query| AnalyzedQueryPolicy {
            page_size: query.page_size,
            max_page_size: query.max_page_size,
            sort_field: query.sort_field.clone(),
            descending: query.sort_direction == "desc",
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;

    #[test]
    fn rejects_api_for_unknown_entity() {
        let app = parser::parse_source(
            "entity Customer {\nfield name: String\n}\napi Missing {\nGET /missing -> list\n}",
        )
        .unwrap();
        let error = analyze(app).err().expect("analysis should fail");
        assert!(error.to_string().contains("unknown entity"));
    }

    #[test]
    fn rejects_unknown_field_types() {
        let app = parser::parse_source("entity Customer {\nfield name: Mystery\n}").unwrap();
        let error = analyze(app).err().expect("analysis should fail");
        assert!(error.to_string().contains("unsupported AXL type"));
    }


    #[test]
    fn rejects_invalid_query_policy() {
        let app = parser::parse_source("entity Customer {\nfield name: String\n}\napi Customer {\nquery page 50 max 20 sort missing sideways\nGET /customers -> list\n}").unwrap();
        let error = analyze(app).err().expect("analysis should fail");
        assert!(error.to_string().contains("cannot exceed"));
    }

    #[test]
    fn rejects_query_sorting_by_unknown_field() {
        let app = parser::parse_source("entity Customer {\nfield name: String\n}\napi Customer {\nquery page 20 max 100 sort missing asc\nGET /customers -> list\n}").unwrap();
        let error = analyze(app).err().expect("analysis should fail");
        assert!(error.to_string().contains("unknown field"));
    }
}
