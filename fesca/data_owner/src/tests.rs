use super::types::{ColumnDescriptor, ColumnType, Charset, SharedTableOutput, SharedPartyData, SharedRow, SharedBitString};
use super::encode::encode_value;
use super::sharing::{
    share_bit_string, share_string_with_encoding
};
use super::load::load_csv_and_schema_from_config;
use rand::rng;
use rand::Rng;

// === Reconstruction functions for testing ===
fn reconstruct_bit_string(share0: &SharedBitString, share1: &SharedBitString, share2: &SharedBitString) -> Vec<bool> {
    let mut reconstructed_bits = Vec::new();
    for i in 0..share0.bits.len() {
        let bit0 = &share0.bits[i];
        let bit1 = &share1.bits[i];
        let bit2 = &share2.bits[i];
        let reconstructed_bit = bit0.share_a ^ bit1.share_a ^ bit2.share_a;
        reconstructed_bits.push(reconstructed_bit);
    }
    reconstructed_bits
}

fn reconstruct_string_with_encoding(share0: &SharedBitString, share1: &SharedBitString, share2: &SharedBitString, charset: &Charset, max_chars: usize) -> String {
    let bits = reconstruct_bit_string(share0, share1, share2);
    let mut result = String::new();
    let bits_per_char = match charset {
        Charset::Ascii => 7,
        Charset::Utf8 => 8,
        Charset::Custom { bits_per_char } => *bits_per_char,
    };
    for i in 0..max_chars {
        let start_bit = i * bits_per_char;
        let end_bit = start_bit + bits_per_char;
        if start_bit >= bits.len() {
            break;
        }
        let char_bits = &bits[start_bit..end_bit.min(bits.len())];
        let mut char_value = 0u32;
        for (bit_idx, &bit) in char_bits.iter().enumerate() {
            if bit {
                char_value |= 1 << bit_idx;
            }
        }
        if char_value == 0 {
            break; // End of string
        }
        result.push(char::from_u32(char_value).unwrap_or('\0'));
    }
    result
}

fn reconstruct_bool(share0: &SharedBitString, share1: &SharedBitString, share2: &SharedBitString) -> bool {
    let bits = reconstruct_bit_string(share0, share1, share2);
    bits.first().copied().unwrap_or(false)
}

fn reconstruct_u32(share0: &SharedBitString, share1: &SharedBitString, share2: &SharedBitString) -> u32 {
    let bits = reconstruct_bit_string(share0, share1, share2);
    let mut result = 0u32;
    for (i, &bit) in bits.iter().take(32).enumerate() {
        if bit {
            result |= 1 << i;
        }
    }
    result
}

fn reconstruct_f64(share0: &SharedBitString, share1: &SharedBitString, share2: &SharedBitString) -> f64 {
    let bits = reconstruct_bit_string(share0, share1, share2);
    let mut bytes = [0u8; 8];
    for (i, &bit) in bits.iter().take(64).enumerate() {
        let byte_idx = i / 8;
        let bit_idx = i % 8;
        if byte_idx < 8 && bit {
            bytes[byte_idx] |= 1 << bit_idx;
        }
    }
    f64::from_le_bytes(bytes)
}

/// Test function to verify that a single value can be shared and reconstructed correctly
fn test_single_value_sharing() {
    println!("Testing single value sharing...");
    
    let mut rng = rng();
    
    // Test boolean values
    let test_bools = vec!["true", "false", "1", "0"];
    for bool_str in test_bools {
        let col_desc = ColumnDescriptor {
            name: "test_bool".to_string(),
            type_hint: ColumnType::Boolean,
        };
        
        let bits = encode_value(bool_str, &col_desc);
        let (share0, share1, share2) = share_bit_string(&bits, &mut rng);
        let reconstructed_bits = reconstruct_bit_string(&share0, &share1, &share2);
        let reconstructed_bool = reconstruct_bool(&share0, &share1, &share2);
        
        let expected_bool = bool_str.trim().to_lowercase() == "true" || bool_str.trim() == "1";
        assert_eq!(bits, reconstructed_bits, "Boolean bit reconstruction failed for {}", bool_str);
        assert_eq!(expected_bool, reconstructed_bool, "Boolean value reconstruction failed for {}", bool_str);
    }
    
    // Test unsigned integers
    let test_ints = vec!["0", "42", "1000", "4294967295"]; // u32 max
    for int_str in test_ints {
        let col_desc = ColumnDescriptor {
            name: "test_int".to_string(),
            type_hint: ColumnType::UnsignedInt,
        };
        
        let bits = encode_value(int_str, &col_desc);
        let (share0, share1, share2) = share_bit_string(&bits, &mut rng);
        let reconstructed_bits = reconstruct_bit_string(&share0, &share1, &share2);
        let reconstructed_int = reconstruct_u32(&share0, &share1, &share2);
        
        let expected_int: u32 = int_str.parse().unwrap();
        assert_eq!(bits, reconstructed_bits, "Integer bit reconstruction failed for {}", int_str);
        assert_eq!(expected_int, reconstructed_int, "Integer value reconstruction failed for {}", int_str);
    }
    
    // Test float values
    let test_floats = vec!["0.0", "3.14", "-42.5", "1.23456789"];
    for float_str in test_floats {
        let col_desc = ColumnDescriptor {
            name: "test_float".to_string(),
            type_hint: ColumnType::Float { bit_width: 64 },
        };
        
        let bits = encode_value(float_str, &col_desc);
        let (share0, share1, share2) = share_bit_string(&bits, &mut rng);
        let reconstructed_bits = reconstruct_bit_string(&share0, &share1, &share2);
        let reconstructed_float = reconstruct_f64(&share0, &share1, &share2);
        
        let expected_float: f64 = float_str.parse().unwrap();
        assert_eq!(bits, reconstructed_bits, "Float bit reconstruction failed for {}", float_str);
        assert!((expected_float - reconstructed_float).abs() < f64::EPSILON, 
                "Float value reconstruction failed for {}", float_str);
    }
    
    // Test string values
    let test_strings = vec!["hello", "world", "test", ""];
    for string_str in test_strings {
        let col_desc = ColumnDescriptor {
            name: "test_string".to_string(),
            type_hint: ColumnType::String { 
                max_chars: 10, 
                charset: Charset::Ascii 
            },
        };
        
        let (share0, share1, share2) = share_string_with_encoding(string_str, &Charset::Ascii, 10, &mut rng);
        let reconstructed_string = reconstruct_string_with_encoding(&share0, &share1, &share2, &Charset::Ascii, 10);
        
        // Remove null characters from the end
        let clean_reconstructed = reconstructed_string.trim_matches('\0');
        assert_eq!(string_str, clean_reconstructed, "String reconstruction failed for '{}'", string_str);
    }
    
    println!("✓ Single value sharing tests passed!");
}

/// Test function to verify that a complete row can be shared and reconstructed correctly
fn test_row_sharing() {
    println!("Testing row sharing...");
    
    let mut rng = rng();
    
    // Create a test row with different data types
    let test_row = vec!["true", "42", "3.14", "hello"];
    let schema_columns = vec![
        ColumnDescriptor {
            name: "bool_col".to_string(),
            type_hint: ColumnType::Boolean,
        },
        ColumnDescriptor {
            name: "int_col".to_string(),
            type_hint: ColumnType::UnsignedInt,
        },
        ColumnDescriptor {
            name: "float_col".to_string(),
            type_hint: ColumnType::Float { bit_width: 64 },
        },
        ColumnDescriptor {
            name: "string_col".to_string(),
            type_hint: ColumnType::String { 
                max_chars: 10, 
                charset: Charset::Ascii 
            },
        },
    ];
    
    // Share the row
    let mut row_shares0 = Vec::new();
    let mut row_shares1 = Vec::new();
    let mut row_shares2 = Vec::new();
    
    for (field, col_desc) in test_row.iter().zip(&schema_columns) {
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
    
    // Reconstruct the row
    let mut reconstructed_row = Vec::new();
    for (i, col_desc) in schema_columns.iter().enumerate() {
        let share0 = &row_shares0[i];
        let share1 = &row_shares1[i];
        let share2 = &row_shares2[i];
        
        let reconstructed_value = match &col_desc.type_hint {
            ColumnType::Boolean => {
                let bool_val = reconstruct_bool(share0, share1, share2);
                if bool_val { "true".to_string() } else { "false".to_string() }
            },
            ColumnType::UnsignedInt => {
                let int_val = reconstruct_u32(share0, share1, share2);
                int_val.to_string()
            },
            ColumnType::Float { .. } => {
                let float_val = reconstruct_f64(share0, share1, share2);
                float_val.to_string()
            },
            ColumnType::String { max_chars, charset } => {
                let string_val = reconstruct_string_with_encoding(share0, share1, share2, charset, *max_chars);
                string_val.trim_matches('\0').to_string()
            },
        };
        
        reconstructed_row.push(reconstructed_value);
    }
    
    // Compare original and reconstructed values
    for (original, reconstructed) in test_row.iter().zip(reconstructed_row.iter()) {
        match *original {
            "true" | "false" | "1" | "0" => {
                let orig_bool = original.trim().to_lowercase() == "true" || original.trim() == "1";
                let recon_bool = reconstructed.trim().to_lowercase() == "true" || reconstructed.trim() == "1";
                assert_eq!(orig_bool, recon_bool, "Boolean mismatch: {} vs {}", original, reconstructed);
            },
            _ => {
                // For numeric and string values, do direct comparison
                if original.parse::<f64>().is_ok() {
                    let orig_float: f64 = original.parse().unwrap();
                    let recon_float: f64 = reconstructed.parse().unwrap();
                    assert!((orig_float - recon_float).abs() < f64::EPSILON, 
                            "Float mismatch: {} vs {}", original, reconstructed);
                } else {
                    assert_eq!(original, reconstructed, "String mismatch: {} vs {}", original, reconstructed);
                }
            }
        }
    }
    
    println!("✓ Row sharing test passed!");
}

/// Test function to verify that a complete table can be shared and reconstructed correctly
fn test_table_sharing() {
    println!("Testing table sharing...");
    
    // Always use config_test.txt for test data
    match load_csv_and_schema_from_config("config_test.txt") {
        Ok((records, schema)) => {
            println!("Loaded {} records for testing", records.len());
            
            let mut rng = rng();
            let mut party0_rows = Vec::new();
            let mut party1_rows = Vec::new();
            let mut party2_rows = Vec::new();
            
            // Share the table
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
            }
            
            // Test reconstruction of first few rows
            let test_rows = std::cmp::min(5, records.len());
            for row_idx in 0..test_rows {
                let original_record = &records[row_idx];
                let party0_row = &party0_rows[row_idx];
                let party1_row = &party1_rows[row_idx];
                let party2_row = &party2_rows[row_idx];
                
                let mut reconstructed_record = Vec::new();
                
                for (col_idx, col_desc) in schema.columns.iter().enumerate() {
                    let share0 = &party0_row.entries[col_idx];
                    let share1 = &party1_row.entries[col_idx];
                    let share2 = &party2_row.entries[col_idx];
                    
                    let reconstructed_value = match &col_desc.type_hint {
                        ColumnType::Boolean => {
                            let bool_val = reconstruct_bool(share0, share1, share2);
                            if bool_val { "true".to_string() } else { "false".to_string() }
                        },
                        ColumnType::UnsignedInt => {
                            let int_val = reconstruct_u32(share0, share1, share2);
                            int_val.to_string()
                        },
                        ColumnType::Float { .. } => {
                            let float_val = reconstruct_f64(share0, share1, share2);
                            float_val.to_string()
                        },
                        ColumnType::String { max_chars, charset } => {
                            let string_val = reconstruct_string_with_encoding(share0, share1, share2, charset, *max_chars);
                            string_val.trim_matches('\0').to_string()
                        },
                    };
                    
                    reconstructed_record.push(reconstructed_value);
                }
                
                // Compare original and reconstructed records
                for (col_idx, (original, reconstructed)) in original_record.iter().zip(reconstructed_record.iter()).enumerate() {
                    let col_desc = &schema.columns[col_idx];
                    
                    match &col_desc.type_hint {
                        ColumnType::Boolean => {
                            let orig_bool = original.trim().to_lowercase() == "true" || original.trim() == "1";
                            let recon_bool = reconstructed.trim().to_lowercase() == "true" || reconstructed.trim() == "1";
                            assert_eq!(orig_bool, recon_bool, 
                                      "Row {} col {}: Boolean mismatch: {} vs {}", 
                                      row_idx, col_idx, original, reconstructed);
                        },
                        ColumnType::Float { .. } => {
                            if let (Ok(orig_float), Ok(recon_float)) = (original.parse::<f64>(), reconstructed.parse::<f64>()) {
                                assert!((orig_float - recon_float).abs() < f64::EPSILON, 
                                        "Row {} col {}: Float mismatch: {} vs {}", 
                                        row_idx, col_idx, original, reconstructed);
                            } else {
                                assert_eq!(original, reconstructed, 
                                          "Row {} col {}: String mismatch: {} vs {}", 
                                          row_idx, col_idx, original, reconstructed);
                            }
                        },
                        _ => {
                            assert_eq!(original, reconstructed, 
                                      "Row {} col {}: Value mismatch: {} vs {}", 
                                      row_idx, col_idx, original, reconstructed);
                        }
                    }
                }
                
                println!("✓ Row {} reconstruction verified", row_idx);
            }
            
            println!("✓ Table sharing test passed for {} rows!", test_rows);
        },
        Err(e) => {
            println!("⚠ Could not load CSV file for table testing: {}", e);
            println!("  This is expected if config_test.txt is not set up for data_owner role");
        }
    }
}

/// Main test function that runs all sharing tests
pub fn run_sharing_tests() {
    println!("Running sharing functionality tests...");
    println!("=====================================");
    
    test_single_value_sharing();
    test_row_sharing();
    test_table_sharing();
    
    println!("=====================================");
    println!("All sharing tests completed successfully!");
}

#[cfg(test)]
mod test_module {
    use super::*;
    
    #[test]
    fn test_sharing_functionality() {
        run_sharing_tests();
    }
} 