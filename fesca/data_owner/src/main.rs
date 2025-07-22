// Data Owner Main Executable
use data_owner::run_data_owner;

#[tokio::main]
async fn main() {
    println!("=== FESCA Data Owner ===");
    println!("Starting data owner process...");
    
    match run_data_owner().await {
        Ok(()) => {
            println!("Data owner process completed successfully!");
        }
        Err(e) => {
            eprintln!("Error in data owner process: {}", e);
            std::process::exit(1);
        }
    }
} 