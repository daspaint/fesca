// Share Receiver Server
// =====================
// gRPC server implementation for receiving binary table shares from data owners

use anyhow::Result;
use std::path::Path;
use tonic::{transport::Server, Request, Response, Status};

// Include the generated protobuf code
pub mod share_service {
    tonic::include_proto!("share_service");
}

use share_service::{
    share_service_server::{ShareService, ShareServiceServer},
    SendTableSharesRequest, SendTableSharesResponse,
};

use super::storage::BinaryShareStorage;

/// gRPC service implementation for receiving table shares
#[derive(Debug)]
pub struct ShareReceiver {
    storage: BinaryShareStorage,
}

impl ShareReceiver {
    pub fn new(storage_base_path: String) -> Self {
        Self {
            storage: BinaryShareStorage::new(storage_base_path),
        }
    }
}

#[tonic::async_trait]
impl ShareService for ShareReceiver {
    /// Receive binary table shares from a data owner and store them as binary files
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

        println!("Computing node received binary shares from: {} ({})", 
                 data_owner.owner_name, data_owner.owner_id);
        println!("Table: {} (ID: {}), Party: {}", 
                 schema.table_name, schema.table_id, party_data.party_id);
        println!("Rows received: {}", party_data.rows.len());

        // Store the binary data using the storage module
        match self.storage.store_binary_shares(party_data, schema, data_owner).await {
            Ok(files_created) => {
                let success_msg = format!("Successfully stored binary shares. Files: {:?}", files_created);
                println!("{}", success_msg);
                
                Ok(Response::new(SendTableSharesResponse {
                    success: true,
                    message: success_msg,
                    storage_path: self.storage.get_storage_path(data_owner, schema),
                }))
            }
            Err(e) => {
                let error_msg = format!("Failed to store binary shares: {}", e);
                eprintln!("{}", error_msg);
                
                Ok(Response::new(SendTableSharesResponse {
                    success: false,
                    message: error_msg,
                    storage_path: String::new(),
                }))
            }
        }
    }
}

/// Start the share receiver server
pub async fn start_server(port: u16, storage_path: String) -> Result<()> {
    // Create storage directory if it doesn't exist
    if !Path::new(&storage_path).exists() {
        println!("Creating storage directory: {}", storage_path);
        std::fs::create_dir_all(&storage_path)?;
    }

    let addr = format!("0.0.0.0:{}", port).parse()?;
    let share_receiver = ShareReceiver::new(storage_path.clone());

    println!("Starting computing node gRPC server on {}", addr);
    println!("Binary shares will be stored in: {}", storage_path);

    Server::builder()
        .add_service(ShareServiceServer::new(share_receiver))
        .serve(addr)
        .await?;

    Ok(())
} 