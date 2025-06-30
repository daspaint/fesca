use anyhow::Result;
use dotenv::dotenv;
use env_logger;
use log::{error, info};
use std::{env, process};

use helpers::read_config::read_config;
use data_owner::read_csv_data;
// use computing_node::run as run_compute; TODO: uncomment when computing_node module is ready
use data_analyst::run as run_analyst;

fn main() -> Result<()>{

    // Initialize environment variables and logging
    dotenv().ok();
    env_logger::init();

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
            // TODO: implement computing_node::run() and uncomment:
            // computing_node::run()?;
            unimplemented!("computing_node not implemented yet");
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
