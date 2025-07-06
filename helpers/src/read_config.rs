use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn read_config(path: &str, name: &str) -> Option<String> {
    // Try to open the file
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    // Read line by line
    for line in reader.lines() {
        if let Ok(line) = line {
            let trimmed_line = line.trim();

            // Split once at the first colon
            if let Some((key, value)) = trimmed_line.split_once(':') {
                if key.trim() == name {
                    return Some(value.trim().to_string());
                }
            }
        }
    }

    None
}
