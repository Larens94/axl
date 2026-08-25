use anyhow::Result;
use crate::parser::AxlApp;

pub struct AnalyzedApp {
    pub name: String,
    pub entities: Vec<AnalyzedEntity>,
    pub apis: Vec<AnalyzedApi>,
    pub auth: Option<AnalyzedAuth>,
    pub ui: Vec<AnalyzedUi>,
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
    let mut analyzed = AnalyzedApp {
        name: app.name,
        entities: Vec::new(),
        apis: Vec::new(),
        auth: None,
        ui: Vec::new(),
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
    
    Ok(analyzed)
}

fn analyze_entity(entity: &crate::parser::Entity) -> Result<AnalyzedEntity> {
    let table_name = format!("{}s", entity.name.to_lowercase());
    
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
        rust_type: "DateTime".to_string(),
        ts_type: "string".to_string(),
        optional: false,
        default: Some("CURRENT_TIMESTAMP".to_string()),
        is_primary_key: false,
        is_foreign_key: false,
    });
    
    fields.push(AnalyzedField {
        name: "updated_at".to_string(),
        field_type: "DateTime".to_string(),
        rust_type: "DateTime".to_string(),
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
    })
}
