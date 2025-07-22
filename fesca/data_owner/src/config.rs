// Configuration Module
// ====================
// This module handles all configuration file reading for the data owner.
// It provides utilities for:
// 1. Reading the unified data owner configuration file
// 2. Loading TBL data and corresponding JSON schema files

use std::fs::File;
use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use std::io::Read;
use crate::types::TableSchema;
use serde_json;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComputingNodes {
    pub node0_url: String,
    pub node1_url: String,
    pub node2_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataOwnerInfo {
    pub owner_id: String,
    pub owner_name: String,
}

/// Unified configuration structure for data owner
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DataOwnerConfig {
    pub computing_nodes: ComputingNodes,
    pub data_owner: DataOwnerInfo,
    pub data_path: String,
}

impl ComputingNodes {
    /// Get node URLs as an array for easier iteration
    pub fn as_array(&self) -> [String; 3] {
        [
            self.node0_url.clone(),
            self.node1_url.clone(),
            self.node2_url.clone(),
        ]
    }
}

/// Loads the unified data owner configuration
/// 
/// # Arguments
/// * `config_path` - Path to the unified config file (e.g., "config_data_owner.json")
/// 
/// # Returns
/// * `Result<DataOwnerConfig>` - Parsed configuration or error
pub fn load_data_owner_config(config_path: &str) -> Result<DataOwnerConfig> {
    let file = File::open(config_path)?;
    let config: DataOwnerConfig = serde_json::from_reader(file)?;
    Ok(config)
}

/// Loads TBL data and its corresponding JSON schema from unified configuration.
/// 
/// # Arguments
/// * `config_path` - Path to the unified configuration file
///
/// # File Structure Expected
/// - TBL file: Contains the actual data rows with pipe-separated values
/// - JSON file: Contains schema with same name as TBL but .json extension
pub fn load_data_and_config(config_path: &str) -> Result<(Vec<Vec<String>>, TableSchema, DataOwnerConfig), Box<dyn std::error::Error>> {
    // Step 1: Load the unified configuration
    let config = load_data_owner_config(config_path)?;

    // Step 2: Load TBL data from the configured path
    let mut file = File::open(&config.data_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut records = Vec::new();
    
    // Read all records from the TBL file (pipe-separated values)
    for line in contents.lines() {
        let line = line.trim();
        if !line.is_empty() {
            // Split by pipe and remove trailing empty field if present
            let mut fields: Vec<String> = line.split('|').map(|s| s.to_string()).collect();
            // Remove the last empty field if it exists (common in TBL format)
            if fields.last().map_or(false, |s| s.is_empty()) {
                fields.pop();
            }
            records.push(fields);
        }
    }

    // Step 3: Construct the schema file path
    // Schema file should have the same name as TBL but with .json extension
    let tbl_path = Path::new(&config.data_path);
    let schema_path = tbl_path.with_extension("json");

    // Step 4: Load and parse the JSON schema file
    let schema_file = File::open(&schema_path)
        .map_err(|e| format!("Failed to open schema file '{}': {}", schema_path.display(), e))?;
    
    let schema: TableSchema = serde_json::from_reader(schema_file)
        .map_err(|e| format!("Failed to parse schema file '{}': {}", schema_path.display(), e))?;

    // Step 5: Return data, schema, and config
    Ok((records, schema, config))
} 