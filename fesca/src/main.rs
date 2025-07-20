/*
The main entry point for the FESCA framework.
This file sets up the command-line interface and starts the appropriate role based on user input.
Example usage:
    cargo run -- --role DataOwner
 */
use clap::{Parser, ValueEnum};
use log::{error, info};
use std::error::Error;
use std::process;

use data_owner::run_data_owner;
use data_analyst::run as run_analyst;

#[derive(Clone, ValueEnum, Debug)]
#[clap(rename_all = "snake_case")]
enum Role {
    DataOwner,
    DataAnalyst,
    ComputingNode,
}

// CLI arguments
#[derive(Parser)]
struct Cli {
    #[arg(value_enum)]
    role: Role,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    info!("Starting FESCA framework...");

    let args = Cli::parse();

    match args.role {
        Role::DataOwner => {
            info!("Running as Data Owner...");
            if let Err(e) = run_data_owner() {
                error!("Error running data owner: {}", e);
                process::exit(1);
            }
        }
        Role::DataAnalyst => {
            info!("Running as Data Analyst...");
            if let Err(e) = run_data_analyst() {
                error!("Error running data analyst: {}", e);
                process::exit(1);
            }
        }
        Role::ComputingNode => {
            info!("Running as Computing Node...");
            // TODO: implement `computing_node::run()` and uncomment:
            // if let Err(e) = run_computing_node() {
            //     error!("Error running computing node: {}", e);
            //     process::exit(1);
            // }
            unimplemented!("computing_node not implemented yet");
        }
    }

    Ok(())
}
