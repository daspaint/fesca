// Receiving shares module components
pub mod receiving_shares {
    pub mod storage;
}

pub mod grpc_server;
// Find table server implementation
pub mod find_table_service;

// Key exchange utilities
pub mod key_exchange {
    pub mod correlated_randomness;
    pub mod key_exchange_server;
}

use anyhow::Result;
use std::env;
use log::{info, warn};

// Re-export main functionality

pub use grpc_server::{ShareReceiver, start_server};
pub use receiving_shares::storage::BinaryShareStorage;
pub use key_exchange::correlated_randomness::{generate_keys, ComputingNodeConfig};
pub use key_exchange::key_exchange_server::create_key_exchange_service;

/// Main entry point for computing node functionality.
/// This function is called by the main FESCA entry point.
pub fn run_computing_node() -> Result<()> {
    // Run the async operation using tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(run_computing_node_async())
}

/// Internal async implementation of computing node functionality
async fn run_computing_node_async() -> Result<()> {
    info!("=== FESCA Computing Node ===");
    
    // Get port from environment variable or use default
    let port = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .unwrap_or(50051);
    
    // Get storage path from environment or use default
    let home_dir = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let storage_path = env::var("STORAGE_PATH")
        .unwrap_or_else(|_| format!("{}/fesca_shares", home_dir));
    
    info!("Starting computing node server...");
    info!("Port: {}", port);
    info!("Storage: {}", storage_path);
    
    // Start the server in a background task
    let server_task = tokio::spawn(start_server(port, storage_path.clone()));
    
    // Give the server a moment to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Attempt key generation with retries after server startup
    info!("Attempting correlated randomness key generation...");
    
    // Try key generation with retries since nodes start manually with delays
    let max_retries = 5;
    let mut key_generation_success = false;
    
    for attempt in 1..=max_retries {
        info!("Key generation attempt {}/{}", attempt, max_retries);
        
        match generate_keys().await {
            Ok(()) => {
                info!("Key generation completed successfully!");
                key_generation_success = true;
                break;
            }
            Err(e) => {
                warn!("Key generation attempt {} failed: {}", attempt, e);
                if attempt < max_retries {
                    let delay = 10; // Wait 10 seconds between attempts
                    info!("Retrying in {} seconds... (other nodes might still be starting)", delay);
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                }
            }
        }
    }
    
    if !key_generation_success {
        warn!("Key generation failed after {} attempts", max_retries);
        info!("This is normal if other computing nodes aren't started yet");
        info!("Key generation will be retried when other nodes come online");
    }
    
    // Wait for the server to complete
    server_task.await?
}
