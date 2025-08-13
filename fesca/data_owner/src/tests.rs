// Tests for the data_owner module
// ================================
// This module contains tests for the secret sharing and reconstruction functionality.
// It verifies that data can be correctly shared into 3 parties and then reconstructed
// to match the original data.

use crate::types::{BitVector, BinaryPartyData, ColumnType, Charset};
use crate::config::load_data_and_config;
use crate::run_table_sharing;
use rand::{thread_rng, Rng};
use log::info;


/// Reconstructs the original bit vector from the three party shares
/// 
/// In 3-party replicated secret sharing, each party holds 2 out of 3 shares.
/// Party 0: (a, b), Party 1: (b, c), Party 2: (a, c)
/// Original bit = a ⊕ b ⊕ c
fn reconstruct_bitvector_from_shares(
    party0: &BinaryPartyData,
    party1: &BinaryPartyData, 
    party2: &BinaryPartyData,
    row_idx: usize,
    bit_offset: u32,
    bit_length: u32,
) -> BitVector {
    let mut reconstructed = BitVector::new();
    
    // Get the byte arrays from each party for the specified row
    let row0 = &party0.rows[row_idx];
    let row1 = &party1.rows[row_idx];
    let row2 = &party2.rows[row_idx];
    
    // Convert byte arrays back to bit vectors
    let bitvec_a0 = bytes_to_bitvector(&row0.bitstring_a);
    let bitvec_b0 = bytes_to_bitvector(&row0.bitstring_b);
    let _bitvec_b1 = bytes_to_bitvector(&row1.bitstring_a); // Party 1's first share is 'b'
    let bitvec_c1 = bytes_to_bitvector(&row1.bitstring_b); // Party 1's second share is 'c'
    let _bitvec_a2 = bytes_to_bitvector(&row2.bitstring_a); // Party 2's first share is 'a'
    let _bitvec_c2 = bytes_to_bitvector(&row2.bitstring_b); // Party 2's second share is 'c'
    
    // Verify consistency: a0 should equal a2, b0 should equal b1, c1 should equal c2
    // For the requested bit range, reconstruct each bit
    for i in 0..bit_length {
        let bit_pos = (bit_offset + i) as usize;
        
        // Get the three shares for this bit position
        let a = bitvec_a0.get(bit_pos).map(|b| *b).unwrap_or(false);
        let b = bitvec_b0.get(bit_pos).map(|b| *b).unwrap_or(false);  
        let c = bitvec_c1.get(bit_pos).map(|b| *b).unwrap_or(false);
        
        // Reconstruct: original = a ⊕ b ⊕ c
        let original_bit = a ^ b ^ c;
        reconstructed.push(original_bit);
    }
    
    reconstructed
}

/// Converts byte array back to BitVector (reverses the bytes_to_vector conversion)
fn bytes_to_bitvector(bytes: &[u8]) -> BitVector {
    let mut bitvec = BitVector::new();
    
    for &byte in bytes {
        for i in 0..8 {
            let bit = (byte >> i) & 1 == 1;
            bitvec.push(bit);
        }
    }
    
    bitvec
}

/// Decodes a BitVector back to the original value based on the column type
fn decode_bitvector(bits: &BitVector, column_type: &ColumnType) -> String {
    match column_type {
        ColumnType::Boolean => {
            let bit = bits.get(0).map(|b| *b).unwrap_or(false);
            bit.to_string()
        },
        ColumnType::UnsignedInt => {
            let mut value = 0u32;
            for (i, bit) in bits.iter().enumerate().take(32) {
                if *bit {
                    value |= 1 << i;
                }
            }
            value.to_string()
        },
        ColumnType::Float => {
            let mut value = 0u64;
            for (i, bit) in bits.iter().enumerate().take(64) {
                if *bit {
                    value |= 1 << i;
                }
            }
            let float_val = f64::from_bits(value);
            format!("{:.2}", float_val)
        },
        ColumnType::String { max_chars, charset } => {
            let bits_per_char = match charset {
                Charset::Ascii => 7,
                Charset::Utf8 => 8,
            };
            
            let mut result = String::new();
            for char_idx in 0..*max_chars {
                let start_bit = char_idx * bits_per_char;
                let mut char_value = 0u32;
                
                for bit_idx in 0..bits_per_char {
                    let bit_pos = start_bit + bit_idx;
                    if let Some(bit) = bits.get(bit_pos) {
                        if *bit {
                            char_value |= 1 << bit_idx;
                        }
                    }
                }
                
                if char_value == 0 {
                    break; // Stop at null character
                }
                
                if let Some(c) = char::from_u32(char_value) {
                    result.push(c);
                }
            }
            result
        }
    }
}

#[tokio::test]
async fn test_reconstruct_random_element() {
    info!("Starting test_reconstruct_random_element...");
    
    // Run the table sharing to get the shares
    info!("Running table sharing...");
    let (party0, party1, party2, schema, _config) = run_table_sharing().await
        .expect("Failed to run table sharing");
    
    info!("Table sharing completed successfully!");
    info!("Schema: {} columns, Party 0: {} rows", schema.columns.len(), party0.rows.len());
    
    // Load the original data for comparison
    let (records, _, _) = load_data_and_config("config_data_owner.json")
        .expect("Failed to load original data");
    
    // Choose a random row and column
    let mut rng = thread_rng();
    let row_idx = rng.gen_range(0..records.len());
    let col_idx = rng.gen_range(0..schema.columns.len());
    
    info!("Testing reconstruction of row {} column {}", row_idx, col_idx);
    
    // Get the original value from the loaded data
    let original_value = &records[row_idx][col_idx];
    let column_desc = &schema.columns[col_idx];
    
    // Find the bit offset and length for this column
    let bit_offset = party0.rows[row_idx].column_bit_offsets[col_idx];
    let bit_length = party0.rows[row_idx].column_bit_lengths[col_idx];
    
    info!("Original value: '{}', Column: {} ({})", 
             original_value, column_desc.name, format!("{:?}", column_desc.type_hint));
    
    // Reconstruct the bit vector for this column
    info!("Reconstructing bits for bit_offset={}, bit_length={}", bit_offset, bit_length);
    let reconstructed_bits = reconstruct_bitvector_from_shares(
        &party0, &party1, &party2, row_idx, bit_offset, bit_length
    );
    
    info!("Reconstructed {} bits: {:?}", reconstructed_bits.len(), 
          reconstructed_bits.iter().take(10).collect::<Vec<_>>());
    
    // Decode the reconstructed bits back to the original value
    let reconstructed_value = decode_bitvector(&reconstructed_bits, &column_desc.type_hint);
    
    info!("Reconstructed value: '{}'", reconstructed_value);
    
    // Compare the values (with some tolerance for floating point)
    match &column_desc.type_hint {
        ColumnType::Float => {
            let original_float: f64 = original_value.parse()
                .unwrap_or_else(|e| panic!("Failed to parse original float '{}': {}", original_value, e));
            let reconstructed_float: f64 = reconstructed_value.parse()
                .unwrap_or_else(|e| panic!("Failed to parse reconstructed float '{}': {}", reconstructed_value, e));
            let difference = (original_float - reconstructed_float).abs();
            assert!(difference < 0.01, 
                   "FLOAT MISMATCH:\n  Original:      {}\n  Reconstructed: {}\n  Difference:    {}\n  Tolerance:     0.01\n  Row: {}, Column: {} ({})\n  Bit offset: {}, length: {}", 
                   original_float, reconstructed_float, difference, row_idx, col_idx, column_desc.name, bit_offset, bit_length);
        },
        _ => {
            let orig_trimmed = original_value.trim();
            let recon_trimmed = reconstructed_value.trim();
            assert_eq!(orig_trimmed, recon_trimmed,
                      "VALUE MISMATCH:\n  Original:      '{}' (len: {})\n  Reconstructed: '{}' (len: {})\n  Row: {}, Column: {} ({})\n  Type: {:?}\n  Bit offset: {}, length: {}\n  Original bytes: {:?}\n  Reconstructed bytes: {:?}", 
                      orig_trimmed, orig_trimmed.len(), recon_trimmed, recon_trimmed.len(), 
                      row_idx, col_idx, column_desc.name, column_desc.type_hint, bit_offset, bit_length,
                      orig_trimmed.as_bytes(), recon_trimmed.as_bytes());
        }
    }
    
    info!("Element reconstruction test passed!");
}

#[tokio::test]
async fn test_reconstruct_random_row() {
    // Run the table sharing to get the shares
    let (party0, party1, party2, schema, _config) = run_table_sharing().await
        .expect("Failed to run table sharing");
    
    // Load the original data for comparison
    let (records, _, _) = load_data_and_config("config_data_owner.json")
        .expect("Failed to load original data");
    
    // Choose a random row
    let mut rng = thread_rng();
    let row_idx = rng.gen_range(0..records.len());
    
    info!("Testing reconstruction of entire row {}", row_idx);
    
    // Get the original row from the loaded data
    let original_row = &records[row_idx];
    
    info!("Original row: {:?}", original_row);
    
    // Reconstruct each column in the row
    let mut reconstructed_row = Vec::new();
    
    for (col_idx, column_desc) in schema.columns.iter().enumerate() {
        // Find the bit offset and length for this column
        let bit_offset = party0.rows[row_idx].column_bit_offsets[col_idx];
        let bit_length = party0.rows[row_idx].column_bit_lengths[col_idx];
        
        // Reconstruct the bit vector for this column
        let reconstructed_bits = reconstruct_bitvector_from_shares(
            &party0, &party1, &party2, row_idx, bit_offset, bit_length
        );
        
        // Decode the reconstructed bits back to the original value
        let reconstructed_value = decode_bitvector(&reconstructed_bits, &column_desc.type_hint);
        reconstructed_row.push(reconstructed_value);
    }
    
    info!("Reconstructed row: {:?}", reconstructed_row);
    
    // Compare each column in the row
    for (col_idx, (original, reconstructed)) in original_row.iter().zip(reconstructed_row.iter()).enumerate() {
        let column_desc = &schema.columns[col_idx];
        let bit_offset = party0.rows[row_idx].column_bit_offsets[col_idx];
        let bit_length = party0.rows[row_idx].column_bit_lengths[col_idx];
        
        match &column_desc.type_hint {
            ColumnType::Float => {
                let original_float: f64 = original.parse()
                    .unwrap_or_else(|e| panic!("Failed to parse original float '{}' in column {}: {}", original, col_idx, e));
                let reconstructed_float: f64 = reconstructed.parse()
                    .unwrap_or_else(|e| panic!("Failed to parse reconstructed float '{}' in column {}: {}", reconstructed, col_idx, e));
                let difference = (original_float - reconstructed_float).abs();
                assert!(difference < 0.01, 
                       "ROW FLOAT MISMATCH:\n  Row: {}, Column: {} ({})\n  Original:      {}\n  Reconstructed: {}\n  Difference:    {}\n  Tolerance:     0.01\n  Bit offset: {}, length: {}", 
                       row_idx, col_idx, column_desc.name, original_float, reconstructed_float, difference, bit_offset, bit_length);
            },
            _ => {
                let orig_trimmed = original.trim();
                let recon_trimmed = reconstructed.trim();
                assert_eq!(orig_trimmed, recon_trimmed,
                          "ROW VALUE MISMATCH:\n  Row: {}, Column: {} ({})\n  Original:      '{}' (len: {})\n  Reconstructed: '{}' (len: {})\n  Type: {:?}\n  Bit offset: {}, length: {}\n  Original bytes: {:?}\n  Reconstructed bytes: {:?}", 
                          row_idx, col_idx, column_desc.name, orig_trimmed, orig_trimmed.len(), recon_trimmed, recon_trimmed.len(), 
                          column_desc.type_hint, bit_offset, bit_length, orig_trimmed.as_bytes(), recon_trimmed.as_bytes());
            }
        }
    }
    
    info!("Row reconstruction test passed!");
}


