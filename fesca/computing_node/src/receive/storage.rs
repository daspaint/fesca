// Binary Share Storage
// ====================
// Handles storing binary share data received from data owners

use anyhow::Result;
use std::fs;
use std::io::Write;

use super::server::share_service;

/// Handles storage of binary share data
#[derive(Debug)]
pub struct BinaryShareStorage {
    base_path: String,
}

impl BinaryShareStorage {
    pub fn new(base_path: String) -> Self {
        Self { base_path }
    }

    pub fn get_storage_path(
        &self, 
        data_owner: &share_service::DataOwnerInfo,
        schema: &share_service::TableSchema
    ) -> String {
        format!("{}/{}/{}", self.base_path, data_owner.owner_id, schema.table_name)
    }

    /// Store binary party data as optimized binary files
    pub async fn store_binary_shares(
        &self,
        party_data: &share_service::BinaryPartyData,
        schema: &share_service::TableSchema,
        data_owner: &share_service::DataOwnerInfo,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let storage_path = self.get_storage_path(data_owner, schema);
        
        // Create directory if it doesn't exist
        fs::create_dir_all(&storage_path)?;
        
        let mut files_created = Vec::new();

        // 1. Store the actual binary data
        let data_file = format!("{}/party{}_data.bin", storage_path, party_data.party_id);
        self.write_binary_data(&data_file, party_data).await?;
        files_created.push(data_file);

        // 2. Store schema information for reference
        let schema_file = format!("{}/schema.json", storage_path);
        self.write_schema_json(&schema_file, schema, data_owner).await?;
        files_created.push(schema_file);

        Ok(files_created)
    }

    /// Write the actual binary data (bitstrings) with simplified header
    async fn write_binary_data(
        &self,
        file_path: &str,
        party_data: &share_service::BinaryPartyData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::create(file_path)?;
        
        // Simplified binary header format for prototype:
        // [8 bytes] Magic number: "FESCASHR"
        // [4 bytes] Number of rows: u32
        // Then the actual row data follows...

        let magic = b"FESCASHR"; // 8 bytes
        file.write_all(magic)?;
        
        // Binary data format:
        // [4 bytes] Number of rows: u32
        // For each row:
        //   [4 bytes] Bitstring A length: u32
        //   [Variable] Bitstring A data: bytes
        //   [4 bytes] Bitstring B length: u32 
        //   [Variable] Bitstring B data: bytes
        //   [4 bytes] Number of column offsets: u32
        //   [Variable] Column bit offsets: u32 * count
        //   [4 bytes] Number of column lengths: u32
        //   [Variable] Column bit lengths: u32 * count

        file.write_all(&(party_data.rows.len() as u32).to_le_bytes())?;
        
        for row in &party_data.rows {
            // Write bitstring A
            file.write_all(&(row.bitstring_a.len() as u32).to_le_bytes())?;
            file.write_all(&row.bitstring_a)?;
            
            // Write bitstring B
            file.write_all(&(row.bitstring_b.len() as u32).to_le_bytes())?;
            file.write_all(&row.bitstring_b)?;
            
            // Write column offsets
            file.write_all(&(row.column_bit_offsets.len() as u32).to_le_bytes())?;
            for offset in &row.column_bit_offsets {
                file.write_all(&offset.to_le_bytes())?;
            }
            
            // Write column lengths
            file.write_all(&(row.column_bit_lengths.len() as u32).to_le_bytes())?;
            for length in &row.column_bit_lengths {
                file.write_all(&length.to_le_bytes())?;
            }
        }

        Ok(())
    }

    /// Write schema as JSON for human readability with data owner information
    async fn write_schema_json(
        &self,
        file_path: &str,
        schema: &share_service::TableSchema,
        data_owner: &share_service::DataOwnerInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let schema_data = serde_json::json!({
            "table_name": schema.table_name,
            "table_id": schema.table_id,
            "row_count": schema.row_count,
            "data_owner": {
                "owner_id": data_owner.owner_id,
                "owner_name": data_owner.owner_name
            },
            "columns": schema.columns.iter().map(|col| {
                serde_json::json!({
                    "name": col.name,
                    "type_hint": format!("{:?}", col.type_hint)
                })
            }).collect::<Vec<_>>()
        });

        fs::write(file_path, serde_json::to_string_pretty(&schema_data)?)?;
        Ok(())
    }
} 