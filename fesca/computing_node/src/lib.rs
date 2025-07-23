pub mod helpers;
pub mod node;

// Receive module components
pub mod receive {
    pub mod server;
    pub mod storage;
}

use anyhow::Result;
use std::env;
use log::info;

// Re-export main functionality
pub use node::Node;
pub use receive::server::{ShareReceiver, start_server};
pub use receive::storage::BinaryShareStorage;

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
    
    start_server(port, storage_path).await
}
