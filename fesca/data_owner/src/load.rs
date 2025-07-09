// Data Loading Module
// ===================
// This module handles loading CSV data and corresponding JSON schema files
// based on configuration settings. It provides utilities for:
// 1. Reading configuration files with key-value pairs
// 2. Loading CSV data from specified paths
// 3. Loading corresponding JSON schema files
// 4. Combining data and schema for processing

use std::fs::File;
use std::path::Path;
use std::io::{BufRead, BufReader};
use csv::Reader;
use crate::types::TableSchema;
use serde_json;

/// Reads a configuration value from a key-value configuration file.
/// 
/// The configuration file format is expected to be:
/// ```
/// key1: value1
/// key2: value2
/// ```
/// 
/// # Arguments
/// * `path` - Path to the configuration file
/// * `name` - The key name to look for
/// 
/// # Returns
/// * `Option<String>` - The value if found, None otherwise
/// 
/// # Example
/// ```
/// let data_path = read_config_value("config.txt", "data_path");
/// ```
fn read_config_value(path: &str, name: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);
    
    // Parse each line looking for the key-value pair
    for line in reader.lines() {
        if let Ok(line) = line {
            let trimmed_line = line.trim();
            
            // Skip empty lines and comments
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                continue;
            }
            
            // Split on first colon to separate key from value
            if let Some((key, value)) = trimmed_line.split_once(':') {
                if key.trim() == name {
                    return Some(value.trim().to_string());
                }
            }
        }
    }
    None
}

/// Loads CSV data and its corresponding JSON schema from configuration.
/// 
/// This function performs the following steps:
/// 1. Reads the 'data_path' from the configuration file
/// 2. Loads the CSV data from the specified path
/// 3. Constructs the schema path by replacing .csv with .json
/// 4. Loads and parses the JSON schema file
/// 5. Returns both data and schema for processing
/// 
/// # Arguments
/// * `config_path` - Path to the configuration file containing data_path
/// 
/// # Returns
/// * `Result<(Vec<Vec<String>>, TableSchema), Box<dyn std::error::Error>>` - 
///   A tuple containing the CSV records and parsed schema, or an error
/// 
/// # File Structure Expected
/// - CSV file: Contains the actual data rows
/// - JSON file: Contains schema with same name as CSV but .json extension
/// 
/// # Example
/// ```
/// let (records, schema) = load_csv_and_schema_from_config("config.txt")?;
/// println!("Loaded {} records for table '{}'", records.len(), schema.table_name);
/// ```
pub fn load_csv_and_schema_from_config(config_path: &str) -> Result<(Vec<Vec<String>>, TableSchema), Box<dyn std::error::Error>> {
    // Step 1: Get the data_path from configuration
    let data_path = read_config_value(config_path, "data_path")
        .ok_or_else(|| format!("Failed to read 'data_path' from config file: {}", config_path))?;

    // Step 2: Load CSV data from the specified path
    let mut rdr = Reader::from_path(&data_path)?;
    let mut records = Vec::new();
    
    // Read all records from the CSV file
    for result in rdr.records() {
        let record = result?;
        // Convert each field to String and collect into a vector
        records.push(record.iter().map(|s| s.to_string()).collect());
    }

    // Step 3: Construct the schema file path
    // Schema file should have the same name as CSV but with .json extension
    let csv_path = Path::new(&data_path);
    let schema_path = csv_path.with_extension("json");

    // Step 4: Load and parse the JSON schema file
    let schema_file = File::open(&schema_path)
        .map_err(|e| format!("Failed to open schema file '{}': {}", schema_path.display(), e))?;
    
    let schema: TableSchema = serde_json::from_reader(schema_file)
        .map_err(|e| format!("Failed to parse schema file '{}': {}", schema_path.display(), e))?;

    // Step 5: Return both data and schema
    Ok((records, schema))
} 