pub mod types;
pub mod load;
pub mod encode;
pub mod sharing;

#[cfg(test)]
mod tests;

use std::error::Error;
use std::fs::File;
use std::path::Path;

pub fn read_csv_data(file_path: &str) -> Result<(), Box<dyn Error>> {
    // Check if file exists
    if !Path::new(file_path).exists() {
        return Err(format!("File not found: {}", file_path).into());
    }

    // Create a CSV reader
    let file = File::open(file_path)?;
    let mut rdr = csv::Reader::from_reader(file);
    
    // Get headers
    let headers = rdr.headers()?.clone();
    
    println!("\nFirst 10 entries of CSV data:");
    println!("=============================");
    
    // Print headers
    for header in headers.iter() {
        print!("{:<8}", header);
    }
    println!();
    
    // Print separator
    println!("{}", "-".repeat(32));
    
    // Print first 10 records
    for (i, result) in rdr.records().enumerate() {
        if i >= 10 { break; }  // Only show first 10 entries
        
        let record = result?;
        for field in record.iter() {
            print!("{:<8}", field);
        }
        println!();
    }
    
    Ok(())
}