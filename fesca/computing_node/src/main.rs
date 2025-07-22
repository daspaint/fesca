// Computing Node Main Executable
// ===============================
// This executable runs a computing node that:
// 1. Receives binary share data from data owners via gRPC
// 2. Stores the shares for later computation processing
// 3. (Future) Performs secure multi-party computations

use computing_node::start_server;
use std::env;

#[tokio::main]
async fn main() {
    println!("=== FESCA Computing Node ===");
    
    // Get port from environment variable or use default
    let port = env::var("GRPC_PORT")
        .unwrap_or_else(|_| "50051".to_string())
        .parse::<u16>()
        .unwrap_or(50051);
    
    // Get storage path from environment or use default
    let home_dir = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let storage_path = env::var("STORAGE_PATH")
        .unwrap_or_else(|_| format!("{}/fesca_shares", home_dir));
    
    println!("Starting computing node server...");
    println!("Port: {}", port);
    println!("Storage: {}", storage_path);
    
    if let Err(e) = start_server(port, storage_path).await {
        eprintln!("Error starting computing node server: {}", e);
        std::process::exit(1);
    }
} 