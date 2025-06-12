use helpers::read_config::read_config;
use data_owner::read_csv_data;

fn main() {
    let role = read_config("config.txt", "role").unwrap_or_else(|| "None".to_string());
    println!("FESCA is here with role: {}", role);

    match role.as_str() {
        "data_owner" => {
            println!("Initializing data owner node...");
            if let Err(e) = read_csv_data("data/airtravel.csv") {
                eprintln!("Error reading CSV data: {}", e);
                std::process::exit(1);
            }
        }
        "computing_node" => {
            println!("Initializing computing node...");
        }
        "data_analyst" => {
            println!("Initializing data analyst node...");
        }
        _ => {
            eprintln!("Error: Invalid role '{}'. Must be one of: data_owner, computing_node, data_analyst", role);
            std::process::exit(1);
        }
    }
}
