use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    fs::create_dir_all(output.join("src/handlers"))?;
    fs::create_dir_all(output.join("src/models"))?;
    
    // Generate Cargo.toml
    generate_cargo_toml(app, output)?;
    
    // Generate main.rs
    generate_main_rs(app, output)?;
    
    // Generate models
    for entity in &app.entities {
        generate_model(entity, output)?;
    }
    
    // Generate handlers
    for api in &app.apis {
        generate_handler(api, output)?;
    }
    
    // Generate auth if present
    if app.auth.is_some() {
        generate_auth(app, output)?;
    }
    
    Ok(())
}

fn generate_cargo_toml(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = format!(r#"[package]
name = "{}"
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
chrono = {{ version = "0.4", features = ["serde"] }}
anyhow = "1"
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
dotenvy = "0.15"
"#, app.name.to_lowercase().replace(" ", "-"));
    
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
    
    let content = format!(r#"use axum::{{routing::get, Router}};
use sea_orm::{{Database, DatabaseConnection}};
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use tracing_subscriber::{{layer::SubscriberExt, util::SubscriberInitExt}};

mod handlers;
mod models;

#[derive(Clone)]
pub struct AppState {{
    pub db: DatabaseConnection,
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
    let db = Database::connect(&database_url).await?;
    
    let state = AppState {{ db }};
    
    let app = Router::new()
        .route("/api/health", get(|| async {{ "OK" }}))
{routes}
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .with_state(state);
    
    let addr = "0.0.0.0:3000";
    tracing::info!("Server listening on {{}}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}}
"#);
    
    fs::write(output.join("src/main.rs"), content)?;
    Ok(())
}

fn generate_model(entity: &crate::analyzer::AnalyzedEntity, output: &Path) -> Result<()> {
    let entity_lower = entity.name.to_lowercase();
    
    let mut fields = String::new();
    for field in &entity.fields {
        let optional = if field.optional { "Option<" } else { "" };
        let close = if field.optional { ">" } else { "" };
        fields.push_str(&format!(
            "    pub {}: {}{}{},\n",
            field.name, optional, field.rust_type, close
        ));
    }
    
    let content = format!(r#"use sea_orm::entity::prelude::*;
use serde::{{Deserialize, Serialize}};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "{}")]
pub struct Model {{
{fields}}}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {{}}

impl ActiveModelBehavior for ActiveModel {{}}

#[derive(Debug, Deserialize)]
pub struct CreateInput {{
    // TODO: Add fields
}}

#[derive(Debug, Deserialize)]
pub struct UpdateInput {{
    // TODO: Add fields
}}

#[derive(Debug, Serialize)]
pub struct Response {{
    // TODO: Add fields
}}
"#, entity.table_name);
    
    fs::write(output.join(format!("src/models/{}.rs", entity_lower)), content)?;
    Ok(())
}

fn generate_handler(api: &crate::analyzer::AnalyzedApi, output: &Path) -> Result<()> {
    let entity_lower = api.entity.to_lowercase();
    
    let content = format!(r#"use axum::{{extract::{{Path, State}}, routing::{{get, post, put, delete}}, Json, Router}};
use sea_orm::{{EntityTrait, ActiveModelTrait, Set}};

use crate::models::{{entity_lower}};
use crate::AppState;

pub fn routes() -> Router<AppState> {{
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(show).put(update).delete(delete_one))
}}

async fn list(State(state): State<AppState>) -> Json<serde_json::Value> {{
    // TODO: Implement list
    Json(serde_json::json!({{"message": "list {entity_lower}"}}))
}}

async fn show(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    // TODO: Implement show
    Json(serde_json::json!({{"message": "show {entity_lower}", "id": id}}))
}}

async fn create(State(state): State<AppState>, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {{
    // TODO: Implement create
    Json(serde_json::json!({{"message": "create {entity_lower}"}}))
}}

async fn update(State(state): State<AppState>, Path(id): Path<i32>, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {{
    // TODO: Implement update
    Json(serde_json::json!({{"message": "update {entity_lower}", "id": id}}))
}}

async fn delete_one(State(state): State<AppState>, Path(id): Path<i32>) -> Json<serde_json::Value> {{
    // TODO: Implement delete
    Json(serde_json::json!({{"message": "delete {entity_lower}", "id": id}}))
}}
"#);
    
    fs::write(output.join(format!("src/handlers/{}.rs", entity_lower)), content)?;
    Ok(())
}

fn generate_auth(app: &AnalyzedApp, output: &Path) -> Result<()> {
    let content = r#"use axum::{Json, http::StatusCode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
}

pub async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Implement login
    Err(StatusCode::NOT_IMPLEMENTED)
}

pub async fn register(Json(payload): Json<LoginRequest>) -> Result<Json<AuthResponse>, StatusCode> {
    // TODO: Implement register
    Err(StatusCode::NOT_IMPLEMENTED)
}
"#;
    
    fs::write(output.join("src/auth.rs"), content)?;
    Ok(())
}
