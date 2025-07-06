mod parties;
mod grpc_service;
mod grpc_client;

use anyhow::Result;
use dotenv::dotenv;
use env_logger;
use log::{error, info};
use std::{env, process};

use helpers::read_config::read_config;
use data_owner::read_csv_data;
// use computing_node::run as run_compute; TODO: uncomment when computing_node module is ready
use data_analyst::run as run_analyst;

#[tokio::main]
async fn main() -> Result<()>{

    // Initialize environment variables and logging
    dotenv().ok();
    use env_logger::Env;
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let role = read_config("config.txt", "role").unwrap_or_else(|| "data_analyst".to_string());
    info!("FESCA is here with role: {}", role);

    match role.as_str() {
        "data_owner" => {
            info!("Running as Data Owner");
            let data_path = match read_config("config.txt", "data_path") {
                Some(path) => path,
                None => {
                    error!("Error: 'data_path' must be specified in config.txt");
                    std::process::exit(1);
                }
            };
            
            if let Err(e) = read_csv_data(&data_path) {
                error!("Error reading CSV data from {}: {}", data_path, e);
                std::process::exit(1);
            }
        }
        "computing_node" => {
            info!("Running as Compute Node");
            println!("Testausgabe aus main.rs!");
            
            // Paper-Protokoll Simulation (lokal)
            parties::simulate_paper_protocol();
            
            // gRPC Protokoll Simulation (Netzwerk)
            println!("\n{}", "=".repeat(50));
            println!("STARTE gRPC SIMULATION");
            println!("{}", "=".repeat(50));
            
            // Starte alle drei gRPC-Server in separaten Tasks
            let p1_server = tokio::spawn(grpc_service::run_party_1_server());
            let p2_server = tokio::spawn(grpc_service::run_party_2_server());
            let p3_server = tokio::spawn(grpc_service::run_party_3_server());
            
            // Warte kurz, damit die Server starten können
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            
            // Führe das gRPC-Protokoll aus
            match grpc_client::run_grpc_protocol().await {
                Ok(_) => println!("✅ gRPC Protokoll erfolgreich abgeschlossen"),
                Err(e) => println!("❌ gRPC Protokoll fehlgeschlagen: {}", e),
            }
            
            // Beende die Server (in der Praxis würden sie weiterlaufen)
            p1_server.abort();
            p2_server.abort();
            p3_server.abort();
            
            // TODO: implement computing_node::run() and uncomment:
            // computing_node::run()?;
            // unimplemented!("computing_node not implemented yet");
        }
        "data_analyst" => {
            info!("Running as Data Analyst");
            run_analyst()?; 
        }
        invalid => {
            error!(
                "Invalid role '{}'. Must be: data_owner, computing_node, or data_analyst",
                invalid
            );
            process::exit(1);
        }
    }

    Ok(())
}
