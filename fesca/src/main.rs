use helpers::read_config::read_config;
use data_owner::load::load_csv_and_schema_from_config;
use data_owner::encode::encode_value;
use data_owner::types::{SharedTableOutput, SharedPartyData, SharedRow, SharedBitString, TableSchema, ColumnType};
use data_owner::sharing::{ReplicatedShareable, share_bit_string, share_string_with_encoding};
use rand::rng;
use rand::Rng;
use anyhow::Result;

fn main() -> Result<()> {
    let role = read_config("config.txt", "role").unwrap_or_else(|| "None".to_string());
    println!("FESCA is here with role: {}", role);

    match role.as_str() {
        "data_owner" => {
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
            let mut rng = rng();
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
        }
        "computing_node" => {
            println!("Initializing computing node...");
        }
        "data_analyst" => {
            println!("Initializing data analyst node...");
        }
        _ => {
            eprintln!("Error: Invalid role '{}'. Must be one of: data_owner, computing_node, data_analyst", role);
            std::process::exit(1);
        }
    }
    Ok(())
}
