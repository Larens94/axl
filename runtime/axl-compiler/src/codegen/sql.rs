use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;
    
    for (i, entity) in app.entities.iter().enumerate() {
        let migration_name = format!("m20240101_{:06}_create_{}s", i + 1, entity.name.to_lowercase());
        generate_migration(entity, output, &migration_name)?;
    }
    
    Ok(())
}

fn generate_migration(
    entity: &crate::analyzer::AnalyzedEntity,
    output: &Path,
    migration_name: &str,
) -> Result<()> {
    let mut columns = String::new();
    
    for (i, field) in entity.fields.iter().enumerate() {
        let sql_type = match field.rust_type.as_str() {
            "i32" => "INTEGER",
            "String" => "TEXT",
            "bool" => "BOOLEAN",
            "f64" => "REAL",
            "DateTime" => "DATETIME",
            _ => "TEXT",
        };
        
        let nullable = if field.optional { "" } else { " NOT NULL" };
        let default = if let Some(ref d) = field.default {
            format!(" DEFAULT {}", d)
        } else {
            String::new()
        };
        
        let pk = if field.is_primary_key { " PRIMARY KEY AUTOINCREMENT" } else { "" };
        
        columns.push_str(&format!(
            "                    .col(ColumnDef::new({}::{}).{}{}{}{})\n",
            entity.name.to_uppercase(),
            field.name.to_uppercase().replace("_", "_"),
            sql_type.to_lowercase(),
            nullable,
            default,
            pk
        ));
        
        if i < entity.fields.len() - 1 {
            columns.push_str("\n");
        }
    }
    
    let content = format!(r#"use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {{
    fn name(&self) -> &str {{
        "{migration_name}"
    }}
}}

#[async_trait::async_trait]
impl MigrationTrait for Migration {{
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .create_table(
                Table::create()
                    .table({entity_name}::Table)
                    .if_not_exists()
{columns}
                    .to_owned(),
            )
            .await
    }}

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{
        manager
            .drop_table(Table::drop().table({entity_name}::Table).to_owned())
            .await
    }}
}}

#[derive(Iden)]
pub enum {entity_name} {{
    Table,
{iden_fields}}}
"#,
        migration_name = migration_name,
        entity_name = entity.name,
        columns = columns,
        iden_fields = entity.fields.iter()
            .map(|f| format!("    {}", f.name.to_uppercase().replace("_", "_")))
            .collect::<Vec<_>>()
            .join(",\n")
    );
    
    fs::write(output.join(format!("{}.rs", migration_name)), content)?;
    Ok(())
}
