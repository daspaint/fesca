// data_analyst/src/binary_exec.rs
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Minimal JSON-deserializable schema shape (we only use a few fields)
#[derive(Debug, Deserialize)]
pub struct RawSchema {
    pub columns: Vec<RawColumn>,
    pub row_count: Option<usize>,
    // other fields we ignore
}

#[derive(Debug, Deserialize)]
pub struct RawColumn {
    pub name: String,
    pub type_hint: Option<String>,
}

/// simplified column types we support
#[derive(Debug, Clone)]
enum ColType {
    UnsignedInt64,
    Float64,
    FixedString(usize),
}

/// Simple read: find first json file in the folder, parse schema, find first non-json file as binary blob
pub fn execute_on_binary_folder<P: AsRef<Path>>(folder: P, sql: &str) -> Result<()> {
    let folder = folder.as_ref();
    if !folder.exists() {
        anyhow::bail!("binary folder {:?} does not exist", folder);
    }

    // 1) find schema JSON file (first .json)
    let mut schema_path: Option<PathBuf> = None;
    let mut bin_path: Option<PathBuf> = None;
    for entry in read_dir(folder).with_context(|| format!("reading folder {:?}", folder))? {
        let e = entry?;
        let p = e.path();
        if p.is_file() {
            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                if ext.eq_ignore_ascii_case("json") {
                    schema_path = Some(p.clone());
                    continue;
                }
            }
        }
    }
    if schema_path.is_none() {
        anyhow::bail!("no .json schema file found in {:?}", folder);
    }

    // find binary file (first file not .json)
    for entry in read_dir(folder).with_context(|| format!("reading folder {:?}", folder))? {
        let e = entry?;
        let p = e.path();
        if p.is_file() {
            let skip = p.extension().and_then(|s| s.to_str()).map(|ext| ext.eq_ignore_ascii_case("json")).unwrap_or(false);
            if !skip {
                bin_path = Some(p.clone());
                break;
            }
        }
    }
    let schema_path = schema_path.unwrap();
    let bin_path = bin_path.ok_or_else(|| anyhow::anyhow!("no binary data file found in {:?}", folder))?;

    // 2) parse schema.json
    let mut s = String::new();
    File::open(&schema_path).with_context(|| format!("opening schema {:?}", schema_path))?.read_to_string(&mut s)?;
    let raw: RawSchema = serde_json::from_str(&s).with_context(|| format!("parsing schema {:?}", schema_path))?;

    let row_count = raw.row_count.unwrap_or(0);
    let mut col_types: Vec<ColType> = Vec::with_capacity(raw.columns.len());
    for c in &raw.columns {
        let ct = parse_type_hint(c.type_hint.as_deref());
        col_types.push(ct);
    }
    // compute column sizes in bytes
    let col_sizes: Vec<usize> = col_types.iter().map(|ct| match ct {
        ColType::UnsignedInt64 => 8usize,
        ColType::Float64 => 8usize,
        ColType::FixedString(n) => *n,
    }).collect();

    let row_size: usize = col_sizes.iter().sum();
    // 3) read binary blob
    let mut blob = Vec::new();
    File::open(&bin_path).with_context(|| format!("opening binary file {:?}", bin_path))?.read_to_end(&mut blob)?;
    if blob.len() < row_count * row_size {
        log::warn!("binary blob length {} < expected row_count*row_size {}. We'll still try (maybe header or extra data)", blob.len(), row_count * row_size);
    }

    // 4) build catalog using your local_exec::Catalog type
    // convert rows into local_exec::TableData and register in Catalog
    let mut rows = Vec::with_capacity(row_count);
    for r in 0..row_count {
        let base = r * row_size;
        if base + row_size > blob.len() {
            log::warn!("row {} would read past end of blob (base {} needed {}) — stopping", r, base, base + row_size);
            break;
        }
        let mut offset = 0usize;
        let mut rowcells = Vec::with_capacity(col_types.len());
        for (ci, ct) in col_types.iter().enumerate() {
            let sz = col_sizes[ci];
            let start = base + offset;
            let end = start + sz;
            let slice = &blob[start..end];
            let cell = match ct {
                ColType::UnsignedInt64 => {
                    let v = read_u64_le(slice)?;
                    // convert to local_exec::Cell::Int(i64) (beware overflow)
                    crate::local_exec::Cell::Int(v as i64)
                }
                ColType::Float64 => {
                    let f = read_f64_le(slice)?;
                    // local exec expected Int or Str. We'll store numeric as Int(floor)
                    crate::local_exec::Cell::Int(f as i64)
                }
                ColType::FixedString(len) => {
                    // convert bytes -> string: trim trailing zeros and whitespace
                    let s = if let Some(pos) = slice.iter().position(|&b| b == 0) {
                        String::from_utf8_lossy(&slice[..pos]).to_string()
                    } else {
                        String::from_utf8_lossy(slice).to_string()
                    };
                    crate::local_exec::Cell::Str(s)
                }
            };
            rowcells.push(cell);
            offset += sz;
        }
        rows.push(rowcells);
    }

    // column names
    let col_names: Vec<String> = raw.columns.iter().map(|c| c.name.clone()).collect();

    // register into Catalog
    let mut catalog = crate::local_exec::Catalog::new();
    let table_name = folder.file_name().and_then(|s| s.to_str()).unwrap_or("table").to_string();
    let t = crate::local_exec::TableData { name: table_name.clone(), columns: col_names.clone(), rows: rows.clone() };
    catalog.insert_table(t);

    // 5) build logical plan from SQL
    let plan = crate::sql_to_logical::sql_to_logical_plan(sql)
        .with_context(|| format!("parsing SQL into logical plan: {}", sql))?;

    // 6) execute plan (local, cleartext)
    let res = crate::local_exec::execute(&plan, &catalog)
        .with_context(|| "executing logical plan on reconstructed table")?;

    // pretty print result and also print bitstring for numeric outputs
    match res {
        crate::local_exec::ExecResult::Row(cells) => {
            println!("Execution returned single row with {} cells:", cells.len());
            for (i, c) in cells.iter().enumerate() {
                match c {
                    crate::local_exec::Cell::Int(v) => {
                        println!("  [{}] Int = {}", i, v);
                        // print bits LSB-first up to 64 bits
                        let bits = int_to_bits_u128(*v as i128 as u128, 64);
                        let s: String = bits.iter().map(|b| if *b { '1' } else { '0' }).collect();
                        println!("       bits LSB->MSB (64): {}", s);
                    }
                    crate::local_exec::Cell::Str(sv) => {
                        println!("  [{}] Str = {:?}", i, sv);
                    }
                }
            }
        }
        crate::local_exec::ExecResult::Rows { columns, rows } => {
            println!("Execution returned {} rows and {} columns", rows.len(), columns.len());
            // print a few rows and bitstrings for numeric columns
            for (ridx, row) in rows.iter().enumerate() {
                println!("Row {}:", ridx);
                for (ci, cell) in row.iter().enumerate() {
                    match cell {
                        crate::local_exec::Cell::Int(v) => {
                            print!("  {}: {} ", columns.get(ci).unwrap_or(&format!("col{}",ci)), v);
                            let bits = int_to_bits_u128(*v as i128 as u128, 64);
                            let s: String = bits.iter().map(|b| if *b { '1' } else { '0' }).collect();
                            println!(" bits(64): {}", s);
                        }
                        crate::local_exec::Cell::Str(sv) => {
                            println!("  {}: {:?}", columns.get(ci).unwrap_or(&format!("col{}",ci)), sv);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

// tiny helpers

fn parse_type_hint(s: Option<&str>) -> ColType {
    let s = s.unwrap_or("");
    if s.contains("UnsignedInt") {
        ColType::UnsignedInt64
    } else if s.contains("Float(") || s.contains("FloatType") {
        ColType::Float64
    } else if s.contains("String(") || s.contains("StringType") {
        // extract digits after "max_chars"
        let mut max = 128usize;
        if let Some(idx) = s.find("max_chars") {
            let tail = &s[idx..];
            let digits: String = tail.chars().skip_while(|c| !c.is_digit(10)).take_while(|c| c.is_digit(10)).collect();
            if let Ok(n) = digits.parse::<usize>() {
                max = n;
            }
        }
        ColType::FixedString(max)
    } else {
        // default
        ColType::FixedString(128)
    }
}

fn read_u64_le(slice: &[u8]) -> Result<u64> {
    if slice.len() < 8 {
        anyhow::bail!("expected 8 bytes for u64, got {}", slice.len());
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&slice[0..8]);
    Ok(u64::from_le_bytes(arr))
}
fn read_f64_le(slice: &[u8]) -> Result<f64> {
    if slice.len() < 8 {
        anyhow::bail!("expected 8 bytes for f64, got {}", slice.len());
    }
    let mut arr = [0u8; 8];
    arr.copy_from_slice(&slice[0..8]);
    Ok(f64::from_le_bytes(arr))
}

fn int_to_bits_u128(mut v: u128, width: usize) -> Vec<bool> {
    let mut b = vec![false; width];
    for i in 0..width {
        b[i] = (v & 1u128) == 1u128;
        v >>= 1;
    }
    b
}
