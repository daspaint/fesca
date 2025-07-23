// Data owner: loads TBL data, creates 3-party secret shares, distributes via gRPC.

pub mod types;
pub mod encode;
pub mod sharing;
pub mod config;
pub mod grpc_client;

#[cfg(test)]
mod tests;

use anyhow::Result;
use log::{info, error};

use crate::config::load_data_and_config;
use crate::encode::encode_value;
use crate::types::{ColumnType, BinaryPartyData, BinaryRow, Charset};
use crate::sharing::share_bit_vector;
use crate::grpc_client::ShareClient;

/// Loads data, creates 3-party secret shares, and distributes to computing nodes.
/// This function is called by the main FESCA entry point.
pub fn run_data_owner() -> Result<()> {
    // Run the async operation using tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_data_owner_async())
}

/// Internal async implementation of data owner functionality
async fn run_data_owner_async() -> Result<()> {

    // Step 1: Load TBL data, schema, and configuration from unified config file
    let config_path = if std::path::Path::new("config_data_owner.json").exists() {
        "config_data_owner.json"
    } else {
        "../data_owner/config_data_owner.json"
    };
    
    let (records, schema, config) = match load_data_and_config(config_path) {
        Ok((records, schema, config)) => {
            info!("Loaded {} records and schema for table '{}'.", records.len(), schema.table_name);
            info!("Loaded data owner configuration");
            (records, schema, config)
        },
        Err(e) => {
            error!("Error loading data, schema, or configuration: {e}");
            std::process::exit(1);
        }
    };

    // Step 2: Initialize random number generator for secret sharing
    let mut rng = rand::thread_rng();
    
    // Step 3: Calculate column bit sizes for binary format
    let mut column_bit_sizes = Vec::new();
    for col_desc in &schema.columns {
        let bit_size = match &col_desc.type_hint {
            ColumnType::Boolean => 1,
            ColumnType::UnsignedInt => 32,
            ColumnType::Float => 64,
            ColumnType::String { max_chars, charset } => {
                let bits_per_char = match charset {
                    Charset::Ascii => 7,
                    Charset::Utf8 => 8,
                };
                (*max_chars as u32) * bits_per_char
            }
        };
        column_bit_sizes.push(bit_size);
    }
    
    // Step 4: Initialize binary party data structures directly
    let mut binary_party0 = BinaryPartyData {
        party_id: 0,
        table_id: schema.table_id,
        rows: Vec::new(),
    };
    let mut binary_party1 = BinaryPartyData {
        party_id: 1,
        table_id: schema.table_id,
        rows: Vec::new(),
    };
    let mut binary_party2 = BinaryPartyData {
        party_id: 2,
        table_id: schema.table_id,
        rows: Vec::new(),
    };

    info!("Encoding, sharing, and converting to binary format...");
    let mut processed_rows = 0;
    
    // Step 5: Process each record in the TBL data, generating binary data directly
    for (row_idx, record) in records.iter().enumerate() {
        // Initialize binary data containers for each party
        let mut bitstring_a0 = Vec::new();
        let mut bitstring_b0 = Vec::new();
        let mut bitstring_a1 = Vec::new();
        let mut bitstring_b1 = Vec::new();
        let mut bitstring_a2 = Vec::new();
        let mut bitstring_b2 = Vec::new();
        let mut column_bit_offsets = Vec::new();
        let mut current_offset = 0u32;
        
        // Step 6: Process each field in the record
        for (col_idx, (field, col_desc)) in record.iter().zip(&schema.columns).enumerate() {
            column_bit_offsets.push(current_offset);
            
            // Encode all types uniformly using encode_value, then share the bits
            let bits = encode_value(field, col_desc);
            let ((a0_bytes, b0_bytes), (a1_bytes, b1_bytes), (a2_bytes, b2_bytes)) = share_bit_vector(&bits, &mut rng);
            
            // Append the byte shares directly to each party's bitstrings
            bitstring_a0.extend_from_slice(&a0_bytes);
            bitstring_b0.extend_from_slice(&b0_bytes);
            bitstring_a1.extend_from_slice(&a1_bytes);
            bitstring_b1.extend_from_slice(&b1_bytes);
            bitstring_a2.extend_from_slice(&a2_bytes);
            bitstring_b2.extend_from_slice(&b2_bytes);
            
            current_offset += column_bit_sizes[col_idx];
        }
        
        // Step 7: Create BinaryRow objects and add to each party's data
        let binary_row0 = BinaryRow {
            bitstring_a: bitstring_a0,
            bitstring_b: bitstring_b0,
            column_bit_offsets: column_bit_offsets.clone(),
            column_bit_lengths: column_bit_sizes.clone(),
        };
        let binary_row1 = BinaryRow {
            bitstring_a: bitstring_a1,
            bitstring_b: bitstring_b1,
            column_bit_offsets: column_bit_offsets.clone(),
            column_bit_lengths: column_bit_sizes.clone(),
        };
        let binary_row2 = BinaryRow {
            bitstring_a: bitstring_a2,
            bitstring_b: bitstring_b2,
            column_bit_offsets: column_bit_offsets.clone(),
            column_bit_lengths: column_bit_sizes.clone(),
        };
        
        binary_party0.rows.push(binary_row0);
        binary_party1.rows.push(binary_row1);
        binary_party2.rows.push(binary_row2);
        
        // Progress tracking
        processed_rows += 1;
        if processed_rows % 1000 == 0 {
            info!("Processed {} rows...", processed_rows);
        }
        
        // Debug output for first few rows
        if row_idx < 2 {
            info!("Row {row_idx} shared: first field = {:?}", record.get(0));
        }
    }
    
    info!("All records encoded, shared, and converted to binary. Total rows processed: {}", processed_rows);
    
    info!("Binary data ready for transmission. Party 0 has {} rows with {} bytes per row", 
             binary_party0.rows.len(),
             binary_party0.rows.get(0).map(|r| r.bitstring_a.len() + r.bitstring_b.len()).unwrap_or(0));
    
    // Step 8: Send individual party data to computing nodes via gRPC
    info!("Sending shares to computing nodes...");
    
    let client = ShareClient::new(config.data_owner);
    let node_urls = config.computing_nodes.as_array();
    
    // Send binary data to each computing node using the new binary format
    match client.send_binary_table_shares(
        &schema,
        &[binary_party0.clone(), binary_party1.clone(), binary_party2.clone()],
        &node_urls,
    ).await {
        Ok(responses) => {
            for (i, response) in responses.iter().enumerate() {
                info!("Node {} response: success={}, message={}, path={}", 
                         i, response.success, response.message, response.storage_path);
            }
        },
        Err(e) => {
            error!("Error sending data to computing nodes: {}", e);
            // Continue execution for now, but log the error
        }
    }
    
    info!("Data sharing completed successfully!");
    info!("Party data sizes - Binary format:");
    info!("  Party 0: {} rows, {} bytes per row", binary_party0.rows.len(), 
             binary_party0.rows.get(0).map(|r| r.bitstring_a.len() + r.bitstring_b.len()).unwrap_or(0));
    info!("  Party 1: {} rows, {} bytes per row", binary_party1.rows.len(),
             binary_party1.rows.get(0).map(|r| r.bitstring_a.len() + r.bitstring_b.len()).unwrap_or(0));
    info!("  Party 2: {} rows, {} bytes per row", binary_party2.rows.len(),
             binary_party2.rows.get(0).map(|r| r.bitstring_a.len() + r.bitstring_b.len()).unwrap_or(0));
    
    Ok(())
}