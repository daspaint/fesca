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
use rand::Rng;

use crate::load::load_csv_and_schema_from_config;
use crate::encode::encode_value;
use crate::types::{SharedTableOutput, SharedPartyData, SharedRow, SharedBitString, TableSchema, ColumnType};
use crate::sharing::{ReplicatedShareable, share_bit_string, share_string_with_encoding};

pub fn run_data_owner() -> Result<()> {
    println!("Initializing data owner node...");
    // Load CSV and schema
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

    // Encode and share
    let mut rng = rand::thread_rng();
    let mut party0_rows = Vec::new();
    let mut party1_rows = Vec::new();
    let mut party2_rows = Vec::new();

    println!("Encoding and sharing records...");
    let mut processed_rows = 0;
    for (row_idx, record) in records.iter().enumerate() {
        let mut row_shares0 = Vec::new();
        let mut row_shares1 = Vec::new();
        let mut row_shares2 = Vec::new();
        for (field, col_desc) in record.iter().zip(&schema.columns) {
            match &col_desc.type_hint {
                ColumnType::String { max_chars, charset } => {
                    let (s0, s1, s2) = share_string_with_encoding(field, charset, *max_chars, &mut rng);
                    row_shares0.push(s0);
                    row_shares1.push(s1);
                    row_shares2.push(s2);
                },
                _ => {
                    let bits = encode_value(field, col_desc);
                    let (s0, s1, s2) = share_bit_string(&bits, &mut rng);
                    row_shares0.push(s0);
                    row_shares1.push(s1);
                    row_shares2.push(s2);
                }
            }
        }
        party0_rows.push(SharedRow { entries: row_shares0 });
        party1_rows.push(SharedRow { entries: row_shares1 });
        party2_rows.push(SharedRow { entries: row_shares2 });
        
        processed_rows += 1;
        if processed_rows % 1000 == 0 {
            println!("Processed {} rows...", processed_rows);
        }
        
        if row_idx < 2 {
            println!("Row {row_idx} shared: first field = {:?}", record.get(0));
        }
    }
    println!("All records encoded and shared. Total rows processed: {}", processed_rows);

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
    
    Ok(())
}

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