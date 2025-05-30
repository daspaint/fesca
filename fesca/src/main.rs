use helpers::read_config::read_config;

fn main() {
    let role = read_config("fesca/config.txt", "role:");
    println!("FESCA is here {:?}", role);
}
