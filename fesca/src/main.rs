use helpers::read_config::read_config;

fn main() {
    let role = read_config("config.txt", "role").unwrap_or_else(|| "None".to_string());
    println!("FESCA is here {:?}", role);
}
