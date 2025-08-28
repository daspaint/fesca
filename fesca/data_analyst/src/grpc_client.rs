/*
Sends a request from Data Analyst to a Computing Node to find if a table with a given column exists.
This is a filesystem-based stub: looks up ~/fesca_shares/owner_x/<table_name>$
*/
use anyhow::{bail, Context, Result};
use glob::glob;
use std::fs;
use std::path::PathBuf;
use table_schema::Schema; 

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub owner_dir: PathBuf,
    pub table_dir: PathBuf,
    pub table_name: String,
    pub row_count: u64,
    pub schema: Schema,
}

pub fn find_table(table_name: &str, column_name: &str) -> Result<TableInfo> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let pattern = format!("{}/fesca_shares/owner_*/{}$", home.display(), table_name);

    let mut matches = Vec::new();
    for entry in glob(&pattern).context("Failed to glob fesca_shares pattern")? {
        if let Ok(path) = entry {
            matches.push(path);
        }
    }

    if matches.is_empty() {
        bail!("Check the spelling of the table. Are you sure data_owner splitted the shares?");
    }

    // Try to find a match where both schema.json exists and contains the column
    for table_dir in matches {
        let schema_path = table_dir.join("schema.json");
        if !schema_path.exists() {
            continue;
        }
        let data = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema.json at {}", schema_path.display()))?;
        let schema: Schema = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse schema.json at {}", schema_path.display()))?;

        // validate table name matches
        if schema.table_name != table_name {
            continue;
        }

        // validate column exists
        let found = schema.columns.iter().any(|c| c.name == column_name);
        if !found {
            continue;
        }

        return Ok(TableInfo {
            owner_dir: table_dir.parent().unwrap_or_else(|| table_dir.as_path()).to_path_buf(),
            table_dir: table_dir.clone(),
            table_name: table_name.to_string(),
            row_count: schema.row_count,
            schema,
        });
    }

    // If we get here, none of the matched table dirs had the column
    bail!("Check the spelling of the table. Are you sure data_owner splitted the shares?");
}
