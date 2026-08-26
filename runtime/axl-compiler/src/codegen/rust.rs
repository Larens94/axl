use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("src/handlers"))?;
    fs::create_dir_all(output.join("src/models"))?;
    
    generate_cargo_toml(app, output)?;
    generate_main_rs(app, output)?;
    
    // Generate mod.rs for handlers
    let mut handlers_mod = String::new();
    for api in &app.apis {
        let entity_lower = api.entity.to_lowercase();
        handlers_mod.push_str(&format!("pub mod {};\n", entity_lower));
    }
    fs::write(output.join("src/handlers/mod.rs"), handlers_mod)?;
    
    // Generate mod.rs for models
    let mut models_mod = String::new();
    for entity in &app.entities {
        let entity_lower = entity.name.to_lowercase();
        models_mod.push_str(&format!("pub mod {};\n", entity_lower));
    }
    fs::write(output.join("src/models/mod.rs"), models_mod)?;
    
    for entity in &app.entities {
        generate_model(entity, output)?;
    }
    
    for api in &app.apis {
        generate_handler(api, &app.entities, output)?;
    }
    
    if app.auth.is_some() {
        generate_auth(app, output)?;
    }
    
    generate_env(output)?;
    
    Ok(())
}

fn generate_cargo_toml(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let name = app.name.to_lowercase().replace(" ", "-");
    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = {{ version = "0.8", features = ["macros"] }}
tokio = {{ version = "1", features = ["full"] }}
tower = "0.5"
tower-http = {{ version = "0.6", features = ["cors", "compression-gzip", "trace"] }}
sea-orm = {{ version = "1.1", features = ["runtime-tokio-rustls", "sqlx-sqlite", "macros"] }}
serde = {{ version = "1", features = ["derive"] }}
serde_json = "1"
jsonwebtoken = "9"
argon2 = "0.5"
rand = "0.8"
chrono = {{ version = "0.4", features = ["serde"] }}
anyhow = "1"
thiserror = "2"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"
"#
    );
    
    fs::write(output.join("Cargo.toml"), content)?;
    Ok(())
}

fn generate_main_rs(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let mut routes = String::new();
    
    for api in &app.apis {
        let entity_lower = api.entity.to_lowercase();
        let base_path = api.routes.iter()
            .map(|route| route.path.strip_suffix("/:id").unwrap_or(&route.path))
            .min_by_key(|path| path.len())
            .unwrap_or("/api");
        routes.push_str(&format!(
            "        .nest(\"{}\", handlers::{}::routes())\n",
            base_path, entity_lower
        ));
    }
    
    let mut sql_tables = app.entities.iter().map(create_table_sql).collect::<Vec<_>>().join("\n");
    let (auth_module, auth_route, protected_setup, protected_merge) = if app.auth.is_some() {
        sql_tables.push_str("\nCREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, name TEXT NOT NULL, role TEXT NOT NULL DEFAULT 'user', created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP);");
        let setup = format!("    let protected = Router::new()\n{routes}        .route_layer(axum::middleware::from_fn_with_state(state.clone(), auth::require_auth))\n        .with_state(state.clone());\n");
        ("mod auth;", "        .nest(\"/api/auth\", auth::routes())\n", setup, "        .merge(protected)\n".to_string())
    } else {
        ("", "", String::new(), routes.clone())
    };
    let seed_sql = create_seed_sql(app);
    
    let content = format!(
        r#"use axum::{{routing::get, Router}};
use sea_orm::{{Database, DatabaseConnection}};
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{{layer::SubscriberExt, util::SubscriberInitExt}};

mod handlers;
mod models;
{auth_module}

#[derive(Clone)]
pub struct AppState {{
    pub db: DatabaseConnection,
    pub jwt_secret: String,
}}

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    dotenvy::dotenv().ok();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://app.db?mode=rwc".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| anyhow::anyhow!("JWT_SECRET must be configured"))?;
    let db = Database::connect(&database_url).await?;
    
    create_tables(&db).await?;
    seed_demo_data(&db).await?;
    
    let state = AppState {{ db, jwt_secret }};
{protected_setup}
    let app = Router::new()
        .route("/api/health", get(|| async {{ "OK" }}))
{auth_route}
{protected_merge}
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state);
    
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{{}}", port);
    tracing::info!("Server listening on {{}}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}}

async fn create_tables(db: &DatabaseConnection) -> anyhow::Result<()> {{
    use sea_orm::ConnectionTrait;
    
    db.execute(sea_orm::Statement::from_string(
        sea_orm::DatabaseBackend::Sqlite,
        "{sql_tables}".to_string(),
    )).await?;
    
    Ok(())
}}

async fn seed_demo_data(db: &DatabaseConnection) -> anyhow::Result<()> {{
    use sea_orm::ConnectionTrait;
    let statements = [{seed_sql}];
    for sql in statements {{
        db.execute(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Sqlite, sql.to_string())).await?;
    }}
    Ok(())
}}
"#
    );
    
    fs::write(output.join("src/main.rs"), content)?;
    Ok(())
}

fn create_seed_sql(app: &AnalyzedApp) -> String {
    app.seeds.iter().filter_map(|seed| {
        let entity = app.entities.iter().find(|entity| entity.name == seed.entity)?;
        let columns = seed.values.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ");
        let values = seed.values.iter().map(|(name, value)| {
            let field = entity.fields.iter().find(|field| field.name == *name);
            sql_literal(field, value)
        }).collect::<Vec<_>>().join(", ");
        let (identity_name, identity_value) = seed.values.first()?;
        let identity_field = entity.fields.iter().find(|field| field.name == *identity_name);
        let identity = sql_literal(identity_field, identity_value);
        Some(format!("\"INSERT INTO {} ({}) SELECT {} WHERE NOT EXISTS (SELECT 1 FROM {} WHERE {} = {});\"", entity.table_name, columns, values, entity.table_name, identity_name, identity))
    }).collect::<Vec<_>>().join(",\n        ")
}

fn sql_literal(field: Option<&crate::analyzer::AnalyzedField>, value: &str) -> String {
    if field.is_some_and(|field| matches!(field.rust_type.as_str(), "i32" | "f64" | "bool")) {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "''"))
    }
}

fn create_table_sql(entity: &crate::analyzer::AnalyzedEntity) -> String {
    let columns = entity.fields.iter().map(|field| {
        let sql_type = match field.rust_type.as_str() {
            "i32" => "INTEGER",
            "bool" => "BOOLEAN",
            "f64" => "REAL",
            "DateTime" | "chrono::NaiveDateTime" => "DATETIME",
            _ => "TEXT",
        };
        let primary = if field.is_primary_key { " PRIMARY KEY AUTOINCREMENT" } else { "" };
        let nullable = if field.optional || field.is_primary_key { "" } else { " NOT NULL" };
        let default = field.default.as_ref().map(|value| {
            if field.rust_type == "String" {
                format!(" DEFAULT '{}'", value.replace('\'', "''"))
            } else {
                format!(" DEFAULT {value}")
            }
        }).unwrap_or_default();
        format!("            {} {sql_type}{primary}{nullable}{default}", field.name)
    }).collect::<Vec<_>>().join(",\n");
    format!("CREATE TABLE IF NOT EXISTS {} (\n{}\n);", entity.table_name, columns)
}

fn generate_model(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let table_name = &entity.table_name;
    
    let mut fields = String::new();
    let mut create_fields = String::new();
    let mut update_fields = String::new();
    let mut response_fields = String::new();
    let mut response_from_fields = String::new();
    
    for field in &entity.fields {
        let optional = if field.optional { "Option<" } else { "" };
        let close = if field.optional { ">" } else { "" };
        let rust_name = rust_identifier(&field.name);
        
        // Skip id field - it's added explicitly with primary_key annotation
        if field.name != "id" {
            fields.push_str(&format!(
                "    pub {}: {}{}{},\n",
                rust_name, optional, field.rust_type, close
            ));
        }
        
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            let create_optional = field.optional || field.default.is_some();
            let create_wrapper = if create_optional { "Option<" } else { "" };
            let create_close = if create_optional { ">" } else { "" };
            create_fields.push_str(&format!("    pub {rust_name}: {create_wrapper}{}{create_close},\n", field.rust_type));
            update_fields.push_str(&format!("    pub {rust_name}: Option<{}>,\n", field.rust_type));
            response_fields.push_str(&format!(
                "    pub {}: {}{}{},\n",
                rust_name, optional, field.rust_type, close
            ));
            response_from_fields.push_str(&format!(
                "            {}: model.{},\n",
                rust_name, rust_name
            ));
        }
    }
    
    response_fields.insert_str(0, "    pub id: i32,\n");
    response_fields.push_str("    pub created_at: String,\n");
    
    let content = format!(
        r#"use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{table_name}")]
pub struct Model {{
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
{fields}}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}

#[derive(Debug, Deserialize)]
pub struct CreateInput {{
{create_fields}}}

#[derive(Debug, Deserialize)]
pub struct UpdateInput {{
{update_fields}}}

#[derive(Debug, Serialize)]
pub struct Response {{
{response_fields}}}

impl From<Model> for Response {{
    fn from(model: Model) -> Self {{
        Self {{
            id: model.id,
{response_from_fields}            created_at: model.created_at.to_string(),
        }}
    }}
}}
"#
    );
    
    fs::write(output.join(format!("src/models/{}.rs", entity_lower)), content)?;
    Ok(())
}

fn rust_identifier(name: &str) -> String {
    const KEYWORDS: &[&str] = &[
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false",
        "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut",
        "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "async", "await", "dyn",
    ];
    if KEYWORDS.contains(&name) { format!("r#{name}") } else { name.to_string() }
}

fn sea_orm_column(name: &str) -> String {
    name.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
        })
        .collect()
}

fn generate_handler(api: &crate::analyzer::AnalyzedApi, entities: &[crate::analyzer::AnalyzedEntity], output: &Path) -> Result<()> {
    let entity_lower = api.entity.to_lowercase();
    let entity = entities.iter().find(|e| e.name.to_lowercase() == entity_lower);

    let mut create_assignments = String::new();
    let mut update_assignments = String::new();
    let (axum_query_import, sea_query_imports, query_struct, list_signature, list_query) = if let Some(query) = &api.query {
        let column = sea_orm_column(&query.sort_field);
        let ordering = if query.descending { "order_by_desc" } else { "order_by_asc" };
        (
            ", Query",
            ", QuerySelect, PaginatorTrait",
            "#[derive(Debug, serde::Deserialize)]\nstruct ListQuery { page: Option<u64>, per_page: Option<u64> }\n",
            ", Query(params): Query<ListQuery>".to_string(),
            format!("    let page = params.page.unwrap_or(1).max(1);\n    let per_page = params.per_page.unwrap_or({}).clamp(1, {});\n    let total = Entity::find().count(&state.db).await.unwrap_or(0);\n    let items = Entity::find()\n        .{}(Column::{})\n        .offset((page - 1) * per_page)\n        .limit(per_page)\n        .all(&state.db)\n        .await\n        .unwrap_or_default();", query.page_size, query.max_page_size, ordering, column),
        )
    } else {
        (
            "",
            "",
            "",
            String::new(),
            "    let items = Entity::find()\n        .order_by_asc(Column::Id)\n        .all(&state.db)\n        .await\n        .unwrap_or_default();\n    let total = items.len() as u64;".to_string(),
        )
    };
    if let Some(entity) = entity {
        for field in &entity.fields {
            if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
                let name = rust_identifier(&field.name);
                if field.default.is_some() {
                    let value = if field.optional { "Set(Some(v))" } else { "Set(v)" };
                    create_assignments.push_str(&format!(
                        "    if let Some(v) = payload.{name} {{ active.{name} = {value}; }}\n"
                    ));
                } else {
                    create_assignments.push_str(&format!(
                        "    active.{name} = Set(payload.{name});\n"
                    ));
                }
                let update_value = if field.optional { "Set(Some(v))" } else { "Set(v)" };
                update_assignments.push_str(&format!(
                    "            if let Some(v) = payload.{name} {{ active.{name} = {update_value}; }}\n"
                ));
            }
        }
    }
    
    let content = format!(
        r#"use axum::{{extract::{{Path, State{axum_query_import}}}, routing::get, Json, Router}};
use sea_orm::{{EntityTrait, ActiveModelTrait, ModelTrait, Set, QueryOrder, IntoActiveModel{sea_query_imports}}};

use crate::models::{entity_lower}::*;
use crate::AppState;
{query_struct}

pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(list).post(create))
        .route("/{{id}}", get(show).put(update).delete(delete_one))
}}

async fn list(State(state): State<AppState>{list_signature}) -> Json<serde_json::Value> {{
{list_query}
    let response: Vec<Response> = items.into_iter().map(Response::from).collect();
    Json(serde_json::json!({{ "data": response, "total": total }}))
}}

async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    match Entity::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => Json(serde_json::json!({{ "data": Response::from(item) }})),
        Ok(None) => Json(serde_json::json!({{ "error": "Not found" }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateInput>,
) -> Json<serde_json::Value> {{
    let mut active = <ActiveModel as Default>::default();
{create_assignments}
    
    match active.insert(&state.db).await {{
        Ok(item) => Json(serde_json::json!({{ "data": Response::from(item) }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateInput>,
) -> Json<serde_json::Value> {{
    match Entity::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => {{
            let mut active = item.into_active_model();
{update_assignments}            match active.update(&state.db).await {{
                Ok(updated) => Json(serde_json::json!({{ "data": Response::from(updated) }})),
                Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
            }}
        }}
        Ok(None) => Json(serde_json::json!({{ "error": "Not found" }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn delete_one(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    match Entity::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => {{
            match item.delete(&state.db).await {{
                Ok(_) => Json(serde_json::json!({{ "message": "Deleted successfully" }})),
                Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
            }}
        }}
        Ok(None) => Json(serde_json::json!({{ "error": "Not found" }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}
"#
    );
    
    fs::write(output.join(format!("src/handlers/{}.rs", entity_lower)), content)?;
    Ok(())
}

fn generate_auth(_app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = r#"use axum::{extract::State, routing::post, Json, Router, http::{Request, StatusCode}, middleware::Next, response::Response};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, Validation, Header, EncodingKey};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: i32,
    email: String,
    role: String,
    exp: usize,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/login", post(login)).route("/register", post(register))
}

pub async fn require_auth(
    State(state): State<AppState>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let token = request.headers().get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    decode::<Claims>(token, &DecodingKey::from_secret(state.jwt_secret.as_bytes()), &Validation::new(Algorithm::HS256))
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    Ok(next.run(request).await)
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let email = payload.email.trim().to_lowercase();
    let row = state.db.query_one(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT id, email, password_hash, name, role FROM users WHERE email = ?",
        [email.clone().into()],
    )).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::UNAUTHORIZED)?;
    let hash: String = row.try_get("", "password_hash").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let parsed = PasswordHash::new(&hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Argon2::default().verify_password(payload.password.as_bytes(), &parsed).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let id: i32 = row.try_get("", "id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let name: String = row.try_get("", "name").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let role: String = row.try_get("", "role").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = create_jwt(id, &email, &role, &state.jwt_secret)?;
    Ok(Json(AuthResponse { token, user: serde_json::json!({ "id": id, "email": email, "name": name, "role": role }) }))
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    if payload.password.len() < 8 || !payload.email.contains('@') || payload.name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    let email = payload.email.trim().to_lowercase();
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.to_string();
    state.db.execute(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "INSERT INTO users (email, password_hash, name) VALUES (?, ?, ?)",
        [email.clone().into(), hash.into(), payload.name.trim().to_string().into()],
    )).await.map_err(|error| if error.to_string().contains("UNIQUE") { StatusCode::CONFLICT } else { StatusCode::INTERNAL_SERVER_ERROR })?;
    let row = state.db.query_one(Statement::from_sql_and_values(
        DatabaseBackend::Sqlite,
        "SELECT id FROM users WHERE email = ?",
        [email.clone().into()],
    )).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let id: i32 = row.try_get("", "id").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let token = create_jwt(id, &email, "user", &state.jwt_secret)?;
    Ok(Json(AuthResponse { token, user: serde_json::json!({ "id": id, "email": email, "name": payload.name, "role": "user" }) }))
}

fn create_jwt(user_id: i32, email: &str, role: &str, secret: &str) -> Result<String, StatusCode> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .unwrap()
        .timestamp() as usize;
    
    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        role: role.to_string(),
        exp: expiration,
    };
    
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
"#;
    
    fs::write(output.join("src/auth.rs"), content)?;
    Ok(())
}

fn generate_env(output: &Path) -> Result<()> {
    let content = r#"DATABASE_URL=sqlite://app.db?mode=rwc
JWT_SECRET=replace-with-a-random-secret
PORT=3000
RUST_LOG=debug
"#;
    
    fs::write(output.join(".env.example"), content)?;
    Ok(())
}
