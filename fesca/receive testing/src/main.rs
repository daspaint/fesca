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
        let filename = format!("{}/shares_{}.json", storage_path, timestamp);
        
        // Create a storage structure with metadata
        let storage_data = ShareStorage {
            data_owner: DataOwnerStorage {
                owner_id: data_owner.owner_id.clone(),
                owner_name: data_owner.owner_name.clone(),
                timestamp: data_owner.timestamp,
            },
            schema: SchemaStorage {
                table_name: schema.table_name.clone(),
                table_id: schema.table_id,
                row_count: schema.row_count,
                columns: schema.columns.iter().map(|col| ColumnStorage {
                    name: col.name.clone(),
                    type_hint: format!("{:?}", col.type_hint), // Simplified for storage
                }).collect(),
            },
            party_data: PartyDataStorage {
                party_id: party_data.party_id,
                table_id: party_data.table_id,
                row_count: party_data.rows.len() as u32,
                // Note: In a real implementation, you would store the actual bit shares
                // For testing purposes, we're just storing metadata
                shares_stored: true,
            },
            storage_metadata: StorageMetadata {
                stored_at: Utc::now().to_rfc3339(),
                storage_path: filename.clone(),
            },
        };

        // Write to file
        match serde_json::to_string_pretty(&storage_data) {
            Ok(json_data) => {
                if let Err(e) = fs::write(&filename, json_data) {
                    let error_msg = format!("Failed to write shares to file: {}", e);
                    eprintln!("{}", error_msg);
                    return Ok(Response::new(SendTableSharesResponse {
                        success: false,
                        message: error_msg,
                        storage_path: String::new(),
                    }));
                }
            },
            Err(e) => {
                let error_msg = format!("Failed to serialize shares: {}", e);
                eprintln!("{}", error_msg);
                return Ok(Response::new(SendTableSharesResponse {
                    success: false,
                    message: error_msg,
                    storage_path: String::new(),
                }));
            }
        }

        println!("Successfully stored shares at: {}", filename);

        Ok(Response::new(SendTableSharesResponse {
            success: true,
            message: format!("Shares stored successfully for table '{}' from owner '{}'", 
                           schema.table_name, data_owner.owner_name),
            storage_path: filename,
        }))
    }
}

/// Storage structures for persisting share data
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ShareStorage {
    data_owner: DataOwnerStorage,
    schema: SchemaStorage,
    party_data: PartyDataStorage,
    storage_metadata: StorageMetadata,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DataOwnerStorage {
    owner_id: String,
    owner_name: String,
    timestamp: i64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SchemaStorage {
    table_name: String,
    table_id: u32,
    row_count: u32,
    columns: Vec<ColumnStorage>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ColumnStorage {
    name: String,
    type_hint: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PartyDataStorage {
    party_id: u32,
    table_id: u32,
    row_count: u32,
    shares_stored: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct StorageMetadata {
    stored_at: String,
    storage_path: String,
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