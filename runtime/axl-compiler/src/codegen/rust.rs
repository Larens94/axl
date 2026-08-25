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
        routes.push_str(&format!(
            "        .nest(\"/api/{}s\", handlers::{}::routes())\n",
            entity_lower, entity_lower
        ));
    }
    
    let sql_tables = r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            name TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'user',
            is_active BOOLEAN NOT NULL DEFAULT 1,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT,
            company TEXT,
            phone TEXT,
            status TEXT NOT NULL DEFAULT 'active',
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE IF NOT EXISTS leads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            company TEXT NOT NULL,
            contact TEXT NOT NULL,
            email TEXT,
            source TEXT,
            status TEXT NOT NULL DEFAULT 'warm',
            value INTEGER NOT NULL DEFAULT 0,
            score INTEGER NOT NULL DEFAULT 50,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE IF NOT EXISTS deals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            customer_id INTEGER,
            value INTEGER NOT NULL DEFAULT 0,
            stage TEXT NOT NULL DEFAULT 'discovery',
            probability INTEGER NOT NULL DEFAULT 50,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT,
            activity_type TEXT NOT NULL,
            related_type TEXT,
            related_id INTEGER,
            description TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
    "#;
    
    let content = format!(
        r#"use axum::{{routing::get, Router}};
use sea_orm::{{Database, DatabaseConnection}};
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{{layer::SubscriberExt, util::SubscriberInitExt}};

mod handlers;
mod models;

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
        .unwrap_or_else(|_| "secret-key-change-in-production".to_string());
    let db = Database::connect(&database_url).await?;
    
    create_tables(&db).await?;
    
    let state = AppState {{ db, jwt_secret }};
    
    let app = Router::new()
        .route("/api/health", get(|| async {{ "OK" }}))
{routes}
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
"#
    );
    
    fs::write(output.join("src/main.rs"), content)?;
    Ok(())
}

fn generate_model(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    let table_name = &entity.table_name;
    
    let mut fields = String::new();
    let mut create_fields = String::new();
    let mut response_fields = String::new();
    let mut response_from_fields = String::new();
    
    for field in &entity.fields {
        let optional = if field.optional { "Option<" } else { "" };
        let close = if field.optional { ">" } else { "" };
        
        // Skip id field - it's added explicitly with primary_key annotation
        if field.name != "id" {
            fields.push_str(&format!(
                "    pub {}: {}{}{},\n",
                field.name, optional, field.rust_type, close
            ));
        }
        
        if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
            create_fields.push_str(&format!(
                "    pub {}: {}{}{},\n",
                field.name, optional, field.rust_type, close
            ));
            response_fields.push_str(&format!(
                "    pub {}: {}{}{},\n",
                field.name, optional, field.rust_type, close
            ));
            response_from_fields.push_str(&format!(
                "            {}: model.{},\n",
                field.name, field.name
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
{create_fields}}}

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

fn generate_handler(api: &crate::analyzer::AnalyzedApi, entities: &[crate::analyzer::AnalyzedEntity], output: &Path) -> Result<()> {
    let entity_lower = api.entity.to_lowercase();
    let entity_upper = api.entity.chars().next().unwrap().to_uppercase().to_string() + &api.entity[1..];
    
    let entity = entities.iter().find(|e| e.name.to_lowercase() == entity_lower);
    
    let mut set_fields = String::new();
    if let Some(entity) = entity {
        for field in &entity.fields {
            if !field.is_primary_key && field.name != "created_at" && field.name != "updated_at" {
                set_fields.push_str(&format!(
                    "        if let Some(v) = payload.{name} {{ active.{name} = Set(v); }}\n",
                    name = field.name
                ));
            }
        }
    }
    
    let content = format!(
        r#"use axum::{{extract::{{Path, State}}, routing::{{get, post, put, delete}}, Json, Router}};
use sea_orm::{{EntityTrait, ActiveModelTrait, Set, QueryOrder, IntoActiveModel}};

use crate::models::{entity_lower}::*;
use crate::AppState;

pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(show).put(update).delete(delete_one))
}}

async fn list(State(state): State<AppState>) -> Json<serde_json::Value> {{
    let items = {entity_upper}::find()
        .order_by_asc(Column::Id)
        .all(&state.db)
        .await
        .unwrap_or_default();
    
    let response: Vec<Response> = items.into_iter().map(Response::from).collect();
    Json(serde_json::json!({{ "data": response }}))
}}

async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    match {entity_upper}::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => Json(serde_json::json!({{ "data": Response::from(item) }})),
        Ok(None) => Json(serde_json::json!({{ "error": "Not found" }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateInput>,
) -> Json<serde_json::Value> {{
    let new_item = ActiveModel {{
{set_fields}        ..Default::default()
    }};
    
    match new_item.insert(&state.db).await {{
        Ok(item) => Json(serde_json::json!({{ "data": Response::from(item) }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateInput>,
) -> Json<serde_json::Value> {{
    match {entity_upper}::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => {{
            let mut active = item.into_active_model();
{set_fields}            match active.update(&state.db).await {{
                Ok(updated) => Json(serde_json::json!({{ "data": Response::from(updated) }})),
                Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
            }}
        }}
        Ok(None) => Json(serde_json::json!({{ "error": "Not found" }})),
        Err(e) => Json(serde_json::json!({{ "error": e.to_string() }})),
    }}
}}

async fn delete_one(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    match {entity_upper}::find_by_id(id).one(&state.db).await {{
        Ok(Some(item)) => {{
            match {entity_upper}::delete(item.into_active_model()).exec(&state.db).await {{
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

fn generate_auth(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = r#"use axum::{extract::State, Json, http::StatusCode};
use serde::{Deserialize, Serialize};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::SaltString};
use rand::rngs::OsRng;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

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

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    Err(StatusCode::NOT_IMPLEMENTED)
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
JWT_SECRET=secret-key-change-in-production
PORT=3000
RUST_LOG=debug
"#;
    
    fs::write(output.join(".env"), content)?;
    Ok(())
}
