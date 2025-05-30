use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn read_config(path: &str, name: &str) -> Option<String> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(line) = line {
            println!("{:?}", line);
            let trimmed_line = line.trim(); // trim whole line first

            if let Some((key, value)) = trimmed_line.split_once(':') {
                if key.trim() == name {
                    return Some(value.trim().to_string());
                }
            }
        }
    }

    None
}
