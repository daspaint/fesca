// computing_node/src/grpc_server.rs
/* 
gRPC server implementation for receiving table shares from data owners and communicating
with data analyst.
*/
use anyhow::Result;
use std::path::Path;
use tonic::{transport::Server};
use log::{info, error};

pub mod find_table {
    tonic::include_proto!("find_table");
}

pub mod share_service {
    tonic::include_proto!("share_service");
}

// Use the proto modules generated at crate root (lib.rs includes them)
use crate::grpc_server::share_service;
use crate::grpc_server::share_service::share_service_server::{ShareService, ShareServiceServer};
use crate::grpc_server::share_service::{SendTableSharesRequest, SendTableSharesResponse};

// key-exchange service helper
use crate::key_exchange::key_exchange_server::create_key_exchange_service;

// new find-table server impl (implementation file placed at src/find_table_service.rs)
use crate::grpc_server::find_table_service::TableLookupService;
use crate::grpc_server::find_table::table_lookup_server::TableLookupServer;

// Storage module (kept under receiving_shares::storage)
use crate::receiving_shares::storage::BinaryShareStorage;
use crate::key_exchange::correlated_randomness::ComputingNodeConfig;

use tonic::{Request, Response, Status};

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

        info!("Computing node received binary shares from: {} ({})", 
                 data_owner.owner_name, data_owner.owner_id);
        info!("Table: {} (ID: {}), Party: {}", 
                 schema.table_name, schema.table_id, party_data.party_id);
        info!("Rows received: {}", party_data.rows.len());

        // Store the binary data using the storage module
        match self.storage.store_binary_shares(party_data, schema, data_owner).await {
            Ok(files_created) => {
                let success_msg = format!("Successfully stored binary shares. Files: {:?}", files_created);
                info!("{}", success_msg);
                
                Ok(Response::new(SendTableSharesResponse {
                    success: true,
                    message: success_msg,
                    storage_path: self.storage.get_storage_path(data_owner, schema),
                }))
            }
            Err(e) => {
                let error_msg = format!("Failed to store binary shares: {}", e);
                error!("{}", error_msg);
                
                Ok(Response::new(SendTableSharesResponse {
                    success: false,
                    message: error_msg,
                    storage_path: String::new(),
                }))
            }
        }
    }
}

/// Start the share receiver server with key exchange service
pub async fn start_server(port: u16, storage_path: String) -> Result<()> {
    // Create storage directory if it doesn't exist
    if !Path::new(&storage_path).exists() {
        info!("Creating storage directory: {}", storage_path);
        std::fs::create_dir_all(&storage_path)?;
    }

    let addr = format!("0.0.0.0:{}", port).parse()?;
    let share_receiver = ShareReceiver::new(storage_path.clone());

    // Load configuration for key exchange service
    let config = match ComputingNodeConfig::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load computing node configuration: {}", e);
            // Create a default config if loading fails
            ComputingNodeConfig {
                computation_urls: crate::key_exchange::correlated_randomness::ComputationUrls {
                    url1: String::new(),
                    url2: String::new(),
                },
                node_id: "default_node".to_string(),
                storage_path: storage_path.clone(),
                key_1: String::new(),
                key_2: String::new(),
            }
        }
    };

    // Create key exchange service
    let key_exchange_service = create_key_exchange_service(config);

    // base_dir used by the new TableLookupService is the same storage_path (converted to PathBuf)
    let base_dir = std::path::PathBuf::from(storage_path.clone());
    let find_table_svc = TableLookupServer::new(TableLookupService::new(base_dir));

    info!("Starting computing node gRPC server on {}", addr);
    info!("Binary shares will be stored in: {}", storage_path);
    info!("Key exchange service enabled for correlated randomness");
    info!("Find Table service enabled for table lookups");

    Server::builder()
        .add_service(ShareServiceServer::new(share_receiver))
        .add_service(key_exchange_service)
        .add_service(find_table_svc)
        .serve(addr)
        .await?;

    Ok(())
}
