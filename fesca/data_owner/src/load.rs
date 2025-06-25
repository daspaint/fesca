use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};
use csv::Reader;
use crate::types::TableSchema;
use serde_json;

/// Reads the value for a given key from a config file (key: value per line).
fn read_config_value(path: &str, name: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        if let Ok(line) = line {
            let trimmed_line = line.trim();
            if let Some((key, value)) = trimmed_line.split_once(':') {
                if key.trim() == name {
                    return Some(value.trim().to_string());
                }
            }
        }
    }
    None
}

/// Loads the CSV data and its corresponding JSON schema based on the data_path in the config file.
pub fn load_csv_and_schema_from_config(config_path: &str) -> Result<(Vec<Vec<String>>, TableSchema), Box<dyn std::error::Error>> {
    // Get the data_path from config
    let data_path = read_config_value(config_path, "data_path")
        .ok_or_else(|| format!("Failed to read 'data_path' from config"))?;

    // Load CSV data
    let mut rdr = Reader::from_path(&data_path)?;
    let mut records = Vec::new();
    for result in rdr.records() {
        let record = result?;
        records.push(record.iter().map(|s| s.to_string()).collect());
    }

    // Build the schema path (same directory, same name, .json extension)
    let csv_path = Path::new(&data_path);
    let schema_path = csv_path.with_extension("json");

    // Load the schema
    let schema_file = File::open(&schema_path)?;
    let schema: TableSchema = serde_json::from_reader(schema_file)?;

    Ok((records, schema))
} 