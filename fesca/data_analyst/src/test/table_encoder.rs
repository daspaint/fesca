// Table Encoding Module
// =====================
// This module handles reading TBL files, parsing JSON schemas, and generating
// binary encodings of complete tables for secure multi-party computation.

use std::fs;
use log::info;

mod types;
use types::{Charset, ColumnType, TableSchema};

mod encode;
use encode::encode_value;

/// Returns the fixed bit width for a given column type.
/// Note: This returns the theoretical bit width defined by the type,
/// not the actual bits used by specific values.
fn get_column_type_bits(column_type: &ColumnType) -> usize {
    match column_type {
        ColumnType::Boolean => 1,
        ColumnType::UnsignedInt => 32,
        ColumnType::Float => 64,
        ColumnType::String { max_chars, charset } => {
            let bits_per_char = match charset {
                Charset::Ascii => 7,
                Charset::Utf8 => 8,
            };
            max_chars * bits_per_char
        }
    }
}

/// Main function to run table encoding from command line.
/// 
/// Reads a TBL file, parses it according to a JSON schema, and generates binary encoding.
/// This function combines TBL file parsing, schema loading, and binary encoding into
/// a single operation. It reads pipe-delimited data from the TBL file, validates it
/// against the schema, and encodes each value according to its column type.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logger
    env_logger::init();
    
    // Use the test files in the same directory
    let tbl_path = "src/test/partsupp.tbl";
    let schema_path = "src/test/partsupp.json";

    // Read and parse the schema
    let schema_content = fs::read_to_string(schema_path)?;
    let schema: TableSchema = serde_json::from_str(&schema_content)?;
    
    // Log column bit information at the beginning
    info!("Starting encoding for table '{}' with {} columns:", schema.table_name, schema.columns.len());
    for (idx, column) in schema.columns.iter().enumerate() {
        let bits_per_column = get_column_type_bits(&column.type_hint);
        info!("Column {}: '{}' - {} bits per value", idx, column.name, bits_per_column);
    }
    
    // Read and parse the TBL file
    let tbl_content = fs::read_to_string(tbl_path)?;
    let mut encoded_rows = Vec::new();
    
    // Process each line in the TBL file
    for line in tbl_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Split by pipe character and trim whitespace
        let values: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        
        // Skip incomplete rows (last row might be incomplete)
        if values.len() < schema.columns.len() {
            continue;
        }
        
        // Encode each column value
        let mut encoded_row = Vec::new();
        for (column, value) in schema.columns.iter().zip(values.iter()) {
            let encoded_value = encode_value(value, column);
            encoded_row.push(encoded_value);
        }
        
        encoded_rows.push(encoded_row);
    }
    
    // Log total bits per row (calculate from actual encoded data)
    if !encoded_rows.is_empty() {
        let total_bits: usize = encoded_rows[0].iter().map(|bv| bv.len()).sum();
        info!("Total bits per row: {}", total_bits);
    }
    
    info!("Encoding completed successfully!");

    //TODO: Here you can use the encoded rows for further reading.
    info!("Encoded rows: {:?}", encoded_rows);
    
    Ok(())
}
