// Key Exchange Server for Correlated Randomness
// ==============================================
// gRPC server implementation for exchanging keys between computing nodes

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use log::{info, warn};

// Include the generated protobuf code
pub mod key_exchange_service {
    tonic::include_proto!("key_exchange_service");
}

use key_exchange_service::{
    key_exchange_service_server::{KeyExchangeService, KeyExchangeServiceServer},
    SendKeyRequest, SendKeyResponse,
    RequestKeyRequest, RequestKeyResponse,
};

use super::correlated_randomness::ComputingNodeConfig;

/// In-memory storage for received keys
type KeyStorage = Arc<Mutex<HashMap<String, String>>>;

/// gRPC service implementation for key exchange
#[derive(Debug)]
pub struct KeyExchangeReceiver {
    /// Storage for keys received from other nodes
    /// Key format: "{sender_node_id}:{key_type}" -> key_value
    key_storage: KeyStorage,
    /// Our node configuration
    config: ComputingNodeConfig,
}

impl KeyExchangeReceiver {
    pub fn new(config: ComputingNodeConfig) -> Self {
        Self {
            key_storage: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
}

#[tonic::async_trait]
impl KeyExchangeService for KeyExchangeReceiver {
    /// Receive a key from another computing node
    async fn send_key(
        &self,
        request: Request<SendKeyRequest>,
    ) -> Result<Response<SendKeyResponse>, Status> {
        let req = request.into_inner();
        
        info!("Received key from node: {} (type: {})", req.sender_node_id, req.key_type);
        
        // Validate the key (should be 64 hex characters for 256 bits)
        if req.key.len() != 64 {
            let error_msg = format!("Invalid key length: expected 64 hex chars, got {}", req.key.len());
            warn!("{}", error_msg);
            return Ok(Response::new(SendKeyResponse {
                success: false,
                message: error_msg,
            }));
        }
        
        // Validate that it's a valid hex string
        if !req.key.chars().all(|c| c.is_ascii_hexdigit()) {
            let error_msg = "Invalid key format: must be hexadecimal".to_string();
            warn!("{}", error_msg);
            return Ok(Response::new(SendKeyResponse {
                success: false,
                message: error_msg,
            }));
        }
        
        // Store the key
        let storage_key = format!("{}:{}", req.sender_node_id, req.key_type);
        {
            let mut storage = self.key_storage.lock().await;
            storage.insert(storage_key, req.key.clone());
        }
        
        info!("Successfully stored key from node {} (type: {})", req.sender_node_id, req.key_type);
        
        Ok(Response::new(SendKeyResponse {
            success: true,
            message: "Key received and stored successfully".to_string(),
        }))
    }
    
    /// Provide a key to another computing node that requests it
    async fn request_key(
        &self,
        request: Request<RequestKeyRequest>,
    ) -> Result<Response<RequestKeyResponse>, Status> {
        let req = request.into_inner();
        
        info!("Key request from node: {} (type: {})", req.requester_node_id, req.key_type);
        
        // Determine which key to provide based on the request type
        let key_to_provide = match req.key_type.as_str() {
            "key_1" => {
                // If they want our key_1, provide it
                &self.config.key_1
            }
            "key_2" => {
                // If they want key_2, they should get it from our storage 
                // (if another node sent it to us)
                let storage_key = format!("{}:key_1", req.requester_node_id);
                let storage = self.key_storage.lock().await;
                if let Some(stored_key) = storage.get(&storage_key) {
                    return Ok(Response::new(RequestKeyResponse {
                        success: true,
                        message: "Key found in storage".to_string(),
                        key: stored_key.clone(),
                    }));
                } else {
                    // No key available yet
                    return Ok(Response::new(RequestKeyResponse {
                        success: true,
                        message: "Key not available yet".to_string(),
                        key: String::new(),
                    }));
                }
            }
            _ => {
                let error_msg = format!("Unknown key type: {}", req.key_type);
                warn!("{}", error_msg);
                return Ok(Response::new(RequestKeyResponse {
                    success: false,
                    message: error_msg,
                    key: String::new(),
                }));
            }
        };
        
        if key_to_provide.is_empty() {
            Ok(Response::new(RequestKeyResponse {
                success: true,
                message: "Key not available yet".to_string(),
                key: String::new(),
            }))
        } else {
            info!("Providing {} to node {}", req.key_type, req.requester_node_id);
            Ok(Response::new(RequestKeyResponse {
                success: true,
                message: "Key provided successfully".to_string(),
                key: key_to_provide.clone(),
            }))
        }
    }
}

/// Create the KeyExchangeService server
pub fn create_key_exchange_service(config: ComputingNodeConfig) -> KeyExchangeServiceServer<KeyExchangeReceiver> {
    let key_exchange_receiver = KeyExchangeReceiver::new(config);
    KeyExchangeServiceServer::new(key_exchange_receiver)
}
