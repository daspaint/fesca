// Data Owner Module
// =================
// This module handles the data owner's role.
// The data owner is responsible for:
// 1. Loading CSV data and corresponding JSON schema
// 2. Encoding data values into bit vectors
// 3. Creating secret shares using 3-party replicated secret sharing
// 4. Distributing shares to computing nodes

pub mod types;
pub mod load;
pub mod encode;
pub mod sharing;

#[cfg(test)]
mod tests;

use std::error::Error;
use std::fs::File;
use std::path::Path;
use anyhow::Result;

use crate::load::load_csv_and_schema_from_config;
use crate::encode::encode_value;
use crate::types::{SharedTableOutput, SharedPartyData, SharedRow, ColumnType};
use crate::sharing::{share_bit_string, share_string_with_encoding};

/// Main entry point for the data owner role.

pub fn run_data_owner() -> Result<()> {
    println!("Initializing data owner node...");
    
    // Step 1: Load CSV data and corresponding JSON schema
    let (records, schema) = match load_csv_and_schema_from_config("config.txt") {
        Ok((records, schema)) => {
            println!("Loaded {} records and schema for table '{}'.", records.len(), schema.table_name);
            (records, schema)
        },
        Err(e) => {
            eprintln!("Error loading data or schema: {e}");
            std::process::exit(1);
        }
    };

    // Step 2: Initialize random number generator for secret sharing
    let mut rng = rand::thread_rng();
    
    // Step 3: Initialize storage for each party's shares
    let mut party0_rows = Vec::new();
    let mut party1_rows = Vec::new();
    let mut party2_rows = Vec::new();

    println!("Encoding and sharing records...");
    let mut processed_rows = 0;
    
    // Step 4: Process each record in the CSV data
    for (row_idx, record) in records.iter().enumerate() {
        // Initialize share containers for this row
        let mut row_shares0 = Vec::new();
        let mut row_shares1 = Vec::new();
        let mut row_shares2 = Vec::new();
        
        // Step 5: Process each field in the record
        for (field, col_desc) in record.iter().zip(&schema.columns) {
            match &col_desc.type_hint {
                // Special handling for strings - encode and share in one step
                ColumnType::String { max_chars, charset } => {
                    let (s0, s1, s2) = share_string_with_encoding(field, charset, *max_chars, &mut rng);
                    row_shares0.push(s0);
                    row_shares1.push(s1);
                    row_shares2.push(s2);
                },
                // For other types: first encode to bits, then share the bits
                _ => {
                    let bits = encode_value(field, col_desc);
                    let (s0, s1, s2) = share_bit_string(&bits, &mut rng);
                    row_shares0.push(s0);
                    row_shares1.push(s1);
                    row_shares2.push(s2);
                }
            }
        }
        
        // Step 6: Store the shares for each party
        party0_rows.push(SharedRow { entries: row_shares0 });
        party1_rows.push(SharedRow { entries: row_shares1 });
        party2_rows.push(SharedRow { entries: row_shares2 });
        
        // Progress tracking
        processed_rows += 1;
        if processed_rows % 1000 == 0 {
            println!("Processed {} rows...", processed_rows);
        }
        
        // Debug output for first few rows
        if row_idx < 2 {
            println!("Row {row_idx} shared: first field = {:?}", record.get(0));
        }
    }
    
    println!("All records encoded and shared. Total rows processed: {}", processed_rows);

    // Step 7: Package the shares into the final output structure
    let party0_data = SharedPartyData {
        party_id: 0,
        table_id: schema.table_id,
        rows: party0_rows,
    };
    let party1_data = SharedPartyData {
        party_id: 1,
        table_id: schema.table_id,
        rows: party1_rows,
    };
    let party2_data = SharedPartyData {
        party_id: 2,
        table_id: schema.table_id,
        rows: party2_rows,
    };
    
    let shared_table = SharedTableOutput {
        party0_data,
        party1_data,
        party2_data,
        schema,
    };
    
    println!("Shared table generated. Example: party0 first row: {:?}", shared_table.party0_data.rows.get(0));
    
    // TODO: In a real implementation, this would distribute the shares to computing nodes
    
    Ok(())
}

/// Legacy function for reading and displaying CSV data.
/// 
/// This function is kept for backward compatibility and debugging purposes.
/// It simply reads a CSV file and displays the first 10 entries.
/// 
/// # Arguments
/// * `file_path` - Path to the CSV file to read
/// 
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success or error information
pub fn read_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    // Create a CSV reader
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    // Get headers
    let headers = rdr.headers()?.clone();
    
    println!("\nFirst 10 entries of CSV data:");
    println!("=============================");
    
    // Print headers
    for header in headers.iter() {
        print!("{:<8}", header);
    }
    println!();
    
    // Print separator
    println!("{}", "-".repeat(32));
    
    // Print first 10 records
    for (i, result) in rdr.records().enumerate() {
        if i >= 10 { break; }  // Only show first 10 entries
        
        let record = result?;
        for field in record.iter() {
            print!("{:<8}", field);
        }
        println!();
    }
    
    Ok(())
}