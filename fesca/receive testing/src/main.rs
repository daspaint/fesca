// Receive Testing Server
// ======================
// This server receives table shares from data owners and stores them persistently
// in the home directory, organized by table name and data owner.

use anyhow::Result;
use std::fs;
use std::path::Path;
use tonic::{transport::Server, Request, Response, Status};
use chrono::Utc;

// Include the generated protobuf code
pub mod share_service {
    tonic::include_proto!("share_service");
}

use share_service::{
    share_service_server::{ShareService, ShareServiceServer},
    SendTableSharesRequest, SendTableSharesResponse,
};

/// gRPC service implementation for receiving table shares
#[derive(Debug, Default)]
pub struct ShareServiceImpl {}

#[tonic::async_trait]
impl ShareService for ShareServiceImpl {
    /// Receive table shares from a data owner and store them persistently
    async fn send_table_shares(
        &self,
        request: Request<SendTableSharesRequest>,
    ) -> Result<Response<SendTableSharesResponse>, Status> {
        let req = request.into_inner();
        
        // Extract data owner and table information
        let data_owner = req.data_owner.as_ref()
            .ok_or_else(|| Status::invalid_argument("Missing data owner information"))?;
        let schema = req.schema.as_ref()
            .ok_or_else(|| Status::invalid_argument("Missing table schema"))?;
        let party_data = req.party_data.as_ref()
            .ok_or_else(|| Status::invalid_argument("Missing party data"))?;

        println!("Received shares from data owner: {} ({})", 
                 data_owner.owner_name, data_owner.owner_id);
        println!("Table: {} (ID: {}), Party: {}", 
                 schema.table_name, schema.table_id, party_data.party_id);
        println!("Rows received: {}", party_data.rows.len());

        // Create storage path based on table name and data owner (using home directory)
        let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let storage_path = format!("{}/fesca_shares/{}/{}/party_{}", 
                                  home_dir,
                                  data_owner.owner_id, 
                                  schema.table_name, 
                                  party_data.party_id);

        // Create directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&storage_path) {
            let error_msg = format!("Failed to create storage directory: {}", e);
            eprintln!("{}", error_msg);
            return Ok(Response::new(SendTableSharesResponse {
                success: false,
                message: error_msg,
                storage_path: String::new(),
            }));
        }

        // Store the shares as JSON
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        
        // Store schema in a separate file
        let schema_filename = format!("{}/schema_{}.json", storage_path, timestamp);
        let schema_data = serde_json::json!({
            "table_name": schema.table_name,
            "table_id": schema.table_id,
            "row_count": schema.row_count,
            "columns": schema.columns.iter().map(|col| {
                serde_json::json!({
                    "name": col.name,
                    "type_hint": format!("{:?}", col.type_hint)
                })
            }).collect::<Vec<_>>()
        });

        // Store complete SharedPartyData in a separate file
        let party_data_filename = format!("{}/party_data_{}.json", storage_path, timestamp);
        let party_data_json = serde_json::json!({
            "party_id": party_data.party_id,
            "table_id": party_data.table_id,
            "rows": party_data.rows.iter().map(|row| {
                serde_json::json!({
                    "entries": row.entries.iter().map(|entry| {
                        serde_json::json!({
                            "bits": entry.bits.iter().map(|bit| {
                                serde_json::json!({
                                    "share_a": bit.share_a,
                                    "share_b": bit.share_b
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        // Create metadata file with references
        let metadata_filename = format!("{}/metadata_{}.json", storage_path, timestamp);
        let metadata = serde_json::json!({
            "data_owner": {
                "owner_id": data_owner.owner_id,
                "owner_name": data_owner.owner_name,
                "timestamp": data_owner.timestamp
            },
            "storage_metadata": {
                "stored_at": Utc::now().to_rfc3339(),
                "schema_file": schema_filename.clone(),
                "party_data_file": party_data_filename.clone(),
                "storage_path": storage_path.clone()
            }
        });

        // Write all three files
        let files_to_write = vec![
            (&schema_filename, &schema_data),
            (&party_data_filename, &party_data_json),
            (&metadata_filename, &metadata)
        ];

        for (filename, data) in files_to_write {
            match serde_json::to_string_pretty(data) {
                Ok(json_data) => {
                    if let Err(e) = fs::write(filename, json_data) {
                        let error_msg = format!("Failed to write file '{}': {}", filename, e);
                        eprintln!("{}", error_msg);
                        return Ok(Response::new(SendTableSharesResponse {
                            success: false,
                            message: error_msg,
                            storage_path: String::new(),
                        }));
                    }
                },
                Err(e) => {
                    let error_msg = format!("Failed to serialize data for '{}': {}", filename, e);
                    eprintln!("{}", error_msg);
                    return Ok(Response::new(SendTableSharesResponse {
                        success: false,
                        message: error_msg,
                        storage_path: String::new(),
                    }));
                }
            }
        }

        println!("Successfully stored:");
        println!("  Schema: {}", schema_filename);
        println!("  Party Data: {}", party_data_filename);
        println!("  Metadata: {}", metadata_filename);

        Ok(Response::new(SendTableSharesResponse {
            success: true,
            message: format!("Complete shares stored successfully for table '{}' from owner '{}' (party {})", 
                           schema.table_name, data_owner.owner_name, party_data.party_id),
            storage_path: storage_path,
        }))
    }
}



/// Main function to start the gRPC server
#[tokio::main]
async fn main() -> Result<()> {
    // Create ~/fesca_shares directory if it doesn't exist
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let base_dir = format!("{}/fesca_shares", home_dir);
    if !Path::new(&base_dir).exists() {
        println!("Creating base storage directory: {}", base_dir);
        fs::create_dir_all(&base_dir)?;
    }

    let addr = "0.0.0.0:50051".parse()?;
    let share_service = ShareServiceImpl::default();

    println!("Starting gRPC server on {}", addr);
    println!("Shares will be stored in: {}", base_dir);

    Server::builder()
        .add_service(ShareServiceServer::new(share_service))
        .serve(addr)
        .await?;

    Ok(())
} 