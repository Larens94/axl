use std::fs;
use std::path::Path;
use anyhow::Result;
use crate::analyzer::AnalyzedApp;

pub fn generate(app: &AnalyzedApp, output: &Path) -> Result<()> {
    fs::create_dir_all(output)?;

    for (i, entity) in app.entities.iter().enumerate() {
        let migration_name = format!("m20240101_{:06}_create_{}", i + 1, entity.table_name);
        generate_migration(entity, output, &migration_name)?;
    }
    
    Ok(())
}

fn generate_migration(
    entity: &crate::analyzer::AnalyzedEntity,
    output: &Path,
    migration_name: &str,
) -> Result<()> {
    let columns = entity.fields.iter().map(|field| {
        let sql_type = match field.rust_type.as_str() {
            "i32" => "INTEGER",
            "String" => "TEXT",
            "bool" => "BOOLEAN",
            "f64" => "REAL",
            "DateTime" | "chrono::NaiveDateTime" => "DATETIME",
            _ => "TEXT",
        };
        
        let nullable = if field.optional || field.is_primary_key { "" } else { " NOT NULL" };
        let default = if let Some(ref d) = field.default {
            if field.rust_type == "String" {
                format!(" DEFAULT '{}'", d.replace('\'', "''"))
            } else {
                format!(" DEFAULT {d}")
            }
        } else {
            String::new()
        };
        let pk = if field.is_primary_key { " PRIMARY KEY AUTOINCREMENT" } else { "" };
        format!("    {} {sql_type}{pk}{nullable}{default}", field.name)
    }).collect::<Vec<_>>().join(",\n");

    let content = format!(
        "CREATE TABLE IF NOT EXISTS {} (\n{}\n);\n",
        entity.table_name, columns
    );

    fs::write(output.join(format!("{migration_name}.sql")), content)?;
    Ok(())
}
