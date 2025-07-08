use helpers::read_config::read_config;
use data_owner::run_data_owner;
use anyhow::Result;

fn main() -> Result<()> {
    let role = read_config("config.txt", "role").unwrap_or_else(|| "None".to_string());
    println!("FESCA is here with role: {}", role);

    match role.as_str() {
        "data_owner" => {
            run_data_owner()?;
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
    Ok(())
}
