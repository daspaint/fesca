// fesca/data_analyst/src/bin/run_partsupp.rs
// Usage:
//   cargo run --bin run_partsupp -- [path_to_binary_folder] [SQL]
// Examples:
//   cargo run --bin run_partsupp
//   cargo run --bin run_partsupp -- /full/path/to/partsupp 'SELECT AVG(supply_cost) FROM partsupp WHERE part_key = 1'

use anyhow::{Result, Context};
use serde::Deserialize;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;
use std::env;
use std::collections::HashMap;

// import the row_circuit functions (must exist in your crate)
use crate::row_circuit::{ build_row_update_circuit, run_row_circuit_over_rows };

#[derive(Debug, Deserialize)]
struct RawSchema {
    columns: Vec<RawColumn>,
    row_count: Option<usize>,
}
#[derive(Debug, Deserialize)]
struct RawColumn {
    name: String,
    type_hint: Option<String>,
}

#[derive(Debug, Clone)]
enum ColType {
    UnsignedInt64,
    Float64,
    FixedString(usize),
}

fn parse_type_hint(s: Option<&str>) -> ColType {
    let s = s.unwrap_or("");
    if s.contains("UnsignedInt") {
        ColType::UnsignedInt64
    } else if s.contains("Float(") || s.contains("FloatType") {
        ColType::Float64
    } else if s.contains("String(") || s.contains("StringType") {
        // extract max_chars
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
        ColType::FixedString(128)
    }
}

fn read_u64_le(slice: &[u8]) -> Result<u64> {
    if slice.len() < 8 { anyhow::bail!("expected 8 bytes for u64") }
    let mut arr = [0u8;8];
    arr.copy_from_slice(&slice[0..8]);
    Ok(u64::from_le_bytes(arr))
}
fn read_f64_le(slice: &[u8]) -> Result<f64> {
    if slice.len() < 8 { anyhow::bail!("expected 8 bytes for f64") }
    let mut arr = [0u8;8];
    arr.copy_from_slice(&slice[0..8]);
    Ok(f64::from_le_bytes(arr))
}

/// Utility: encode integer to LSB-first bool vector of given width
fn u64_to_bits_lsb(mut v: u64, width: usize) -> Vec<bool> {
    let mut bits = vec![false; width];
    for i in 0..width {
        bits[i] = (v & 1) == 1;
        v >>= 1;
    }
    bits
}

/// Utility: encode signed/int or f64 to integer *cents* bitvector if needed
fn i64_to_bits_lsb(mut v: i64, width: usize) -> Vec<bool> {
    let mut bits = vec![false; width];
    let mut u = v as u64;
    for i in 0..width {
        bits[i] = (u & 1) == 1;
        u >>= 1;
    }
    bits
}

/// Convert bytes (ASCII) into bit vector LSB-first (byte order preserved).
/// Produces len*8 bits.
fn bytes_to_bits_lsb(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len()*8);
    // For each byte we append its 8 bits LSB first
    for &b in bytes.iter() {
        for i in 0..8 {
            bits.push(((b >> i) & 1) == 1);
        }
    }
    bits
}

fn main() -> Result<()> {
    // default folder relative to crate root
    let default_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data_analyst").join("src").join("binary_data").join("owner_001").join("partsupp");

    let mut args = env::args().skip(1);
    let folder_arg = args.next();
    let sql_arg = args.next();

    let folder = folder_arg.map(PathBuf::from).unwrap_or(default_folder);
    let sql = sql_arg.unwrap_or_else(|| {
        // default query (adapt if you want another)
        "SELECT AVG(supply_cost) FROM partsupp WHERE part_key = 1".to_string()
    });

    println!("binary folder: {}", folder.display());
    println!("SQL: {}", sql);

    // 1) find schema json and binary file in folder
    let mut schema_file: Option<PathBuf> = None;
    let mut bin_file: Option<PathBuf> = None;
    for entry in read_dir(&folder).context("reading binary folder")? {
        let e = entry?;
        let p = e.path();
        if p.is_file() {
            if p.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("json")).unwrap_or(false) {
                schema_file = Some(p.clone());
            } else {
                // first non-json file considered binary
                if bin_file.is_none() { bin_file = Some(p.clone()); }
            }
        }
    }
    let schema_file = schema_file.ok_or_else(|| anyhow::anyhow!("schema json not found in folder"))?;
    let bin_file = bin_file.ok_or_else(|| anyhow::anyhow!("binary file not found in folder"))?;

    // 2) parse schema json
    let mut s = String::new();
    File::open(&schema_file).with_context(|| format!("opening schema {:?}", schema_file))?.read_to_string(&mut s)?;
    let raw: RawSchema = serde_json::from_str(&s).context("parsing schema json")?;
    let row_count = raw.row_count.unwrap_or(0);
    let col_types: Vec<ColType> = raw.columns.iter().map(|c| parse_type_hint(c.type_hint.as_deref())).collect();
    let col_names: Vec<String> = raw.columns.iter().map(|c| c.name.clone()).collect();

    // compute sizes and row_size
    let col_sizes: Vec<usize> = col_types.iter().map(|ct| match ct {
        ColType::UnsignedInt64 => 8usize,
        ColType::Float64 => 8usize,
        ColType::FixedString(n) => *n,
    }).collect();
    let row_size: usize = col_sizes.iter().sum();

    // 3) read blob
    let mut blob = Vec::new();
    File::open(&bin_file).with_context(|| format!("opening binary file {:?}", bin_file))?.read_to_end(&mut blob)?;
    if blob.len() < row_count * row_size {
        eprintln!("warning: blob length {} smaller than row_count*row_size {}. Will attempt best-effort.", blob.len(), row_count * row_size);
    }

    // 4) reconstruct rows into simple vector of maps: Vec<HashMap<colname, cell_bytes_or_number>>
    // For numeric types we will store as i64 or f64 in this intermediate structure
    #[derive(Debug, Clone)]
    enum CellValue {
        Int(i64),
        Float(f64),
        Str(String),
    }

    let mut rows: Vec<HashMap<String, CellValue>> = Vec::new();
    for r in 0..row_count {
        let base = r * row_size;
        if base + row_size > blob.len() {
            eprintln!("stopping early: row {} would go past blob end", r);
            break;
        }
        let mut offset = 0usize;
        let mut rowmap = HashMap::new();
        for (ci, ct) in col_types.iter().enumerate() {
            let sz = col_sizes[ci];
            let start = base + offset;
            let end = start + sz;
            let slice = &blob[start..end];
            let name = col_names[ci].clone();
            match ct {
                ColType::UnsignedInt64 => {
                    let v = read_u64_le(slice)?;
                    rowmap.insert(name, CellValue::Int(v as i64));
                }
                ColType::Float64 => {
                    let f = read_f64_le(slice)?;
                    rowmap.insert(name, CellValue::Float(f));
                }
                ColType::FixedString(len) => {
                    let s = if let Some(pos) = slice.iter().position(|&b| b == 0) {
                        String::from_utf8_lossy(&slice[..pos]).to_string()
                    } else {
                        String::from_utf8_lossy(slice).to_string()
                    };
                    rowmap.insert(name, CellValue::Str(s));
                }
            }
            offset += sz;
        }
        rows.push(rowmap);
    }

    println!("Reconstructed {} rows (loaded {})", rows.len(), bin_file.display());

    // 5) parse SQL into logical plan (uses your existing sql_to_logical)
    let plan = crate::sql_to_logical::sql_to_logical_plan(&sql)
        .with_context(|| format!("parsing SQL"))?;

    // 6) For now we support pattern: SELECT AVG(<aggr_col>) FROM <table> WHERE <pred_col> = <literal>
    // So extract aggr column name and predicate (only simple Eq supported)
    use sqlparser::ast::{Statement, Query, SetExpr, Expr as AstExpr, Value as AstValue, SelectItem};
    let (aggr_col, pred_col, pred_value) = match &plan {
        crate::logical_plan::LogicalPlan::Aggregate { input, aggr_exprs, .. } => {
            // extract aggregate column name from aggr_exprs (assume single avg and that expr is Column(idx) or ColumnName)
            if aggr_exprs.len() != 1 {
                anyhow::bail!("only single-aggregate AVG supported by this runner");
            }
            // aggr_exprs: Vec<(AggregateFunc, Expr, Option<String>)>
            let (func, expr, _alias) = &aggr_exprs[0];
            match func {
                crate::logical_plan::AggregateFunc::Avg => { /* ok */ }
                other => anyhow::bail!("only AVG aggregate supported (found {:?})", other),
            }
            // expr should be LPExpr::ColumnName or LPExpr::Column
            let aggr_col_name = match expr {
                crate::logical_plan::Expr::ColumnName(n) => n.clone(),
                crate::logical_plan::Expr::Column(idx) => {
                    // map numeric index to name using schema
                    let idx = *idx;
                    if idx >= col_names.len() { anyhow::bail!("aggregate column index {} out of range", idx); }
                    col_names[idx].clone()
                }
                _ => anyhow::bail!("unsupported aggregate expression {:?}", expr),
            };

            // input should be Filter { predicate, input: Scan{table} }
            match &**input {
                crate::logical_plan::LogicalPlan::Filter { predicate, input: inner } => {
                    // extract predicate column and literal
                    match predicate {
                        crate::logical_plan::Expr::BinaryOp { op, left, right } => {
                            // we only support Eq
                            match op {
                                crate::logical_plan::BinaryOperator::Eq => {}
                                _ => anyhow::bail!("only equality predicates supported"),
                            }
                            // left should be column, right literal or vice versa
                            let (colname, lit) = match (&**left, &**right) {
                                (crate::logical_plan::Expr::ColumnName(n), crate::logical_plan::Expr::LiteralString(s)) => (n.clone(), crate::logical_plan::Expr::LiteralString(s.clone())),
                                (crate::logical_plan::Expr::ColumnName(n), crate::logical_plan::Expr::LiteralInt(v)) => (n.clone(), crate::logical_plan::Expr::LiteralInt(*v)),
                                (crate::logical_plan::Expr::Column(idx), crate::logical_plan::Expr::LiteralInt(v)) => {
                                    let idx = *idx;
                                    if idx >= col_names.len() { anyhow::bail!("predicate column index {} out of range", idx); }
                                    (col_names[idx].clone(), crate::logical_plan::Expr::LiteralInt(*v))
                                }
                                (crate::logical_plan::Expr::LiteralString(s), crate::logical_plan::Expr::ColumnName(n)) => (n.clone(), crate::logical_plan::Expr::LiteralString(s.clone())),
                                (crate::logical_plan::Expr::LiteralInt(v), crate::logical_plan::Expr::ColumnName(n)) => (n.clone(), crate::logical_plan::Expr::LiteralInt(*v)),
                                (l,r) => anyhow::bail!("unsupported predicate form: left={:?}, right={:?}", l, r),
                            };
                            (aggr_col_name, colname, lit)
                        }
                        other => anyhow::bail!("unsupported predicate expression {:?}", other),
                    }
                }
                _ => anyhow::bail!("only Filter over Scan supported as input to Aggregate for this runner"),
            }
        }
        other => anyhow::bail!("unsupported logical plan shape: expected Aggregate, got {:?}", other),
    };

    println!("Aggregate column: {}", aggr_col);
    println!("Predicate column: {}", pred_col);
    println!("Predicate value (AST expr): {:?}", pred_value);

    // 7) Prepare rows as bit-maps for the circuit:
    // decide bit widths
    let SUM_W = 80usize;   // widen to avoid overflow
    let SAL_W = 48usize;   // salary width (bits) for aggregate column
    let CNT_W = 32usize;   // count width
    // for predicate column we'll use a width determined by its type:
    // find predicate column index and type
    let pred_idx = col_names.iter().position(|n| n == &pred_col).ok_or_else(|| anyhow::anyhow!("predicate column not found in schema"))?;
    let pred_coltype = &col_types[pred_idx];

    // constant bits for predicate
    let const_bits = match pred_value {
        crate::logical_plan::Expr::LiteralInt(v) => {
            // integer constant -> bits LSB-first
            let u = *v as i64 as u64;
            u64_to_bits_lsb(u, 64) // use 64 bits (you can reduce)
        }
        crate::logical_plan::Expr::LiteralString(s) => {
            // encode string to bytes, then to bits
            let bytes = s.as_bytes();
            bytes_to_bits_lsb(bytes)
        }
        other => anyhow::bail!("unsupported predicate literal {:?}", other),
    };

    // prepare vector of row maps: each row map uses keys expected by row_circuit: "salary", "dept", "const_dept"
    let mut row_bit_maps: Vec<HashMap<String, Vec<bool>>> = Vec::new();
    for row in rows.iter() {
        // extract aggregate column value
        let aggr_val = row.get(&aggr_col).ok_or_else(|| anyhow::anyhow!("aggregate column missing in row"))?;
        // predicate column as CellValue
        let pred_val = row.get(&pred_col).ok_or_else(|| anyhow::anyhow!("predicate column missing in row"))?;

        // salary bits
        let salary_bits: Vec<bool> = match aggr_val {
            // if integer
            crate::local_exec::Cell::Int(v) => u64_to_bits_lsb(*v as u64, SAL_W),
            // if float stored as Int or Float we convert to cents / cast
            _ => {
                // fallback: try to string parse via debug -> 0
                vec![false; SAL_W]
            }
        };

        // predicate bits: for integers store as bits; for string store ASCII bits
        let pred_bits: Vec<bool> = match pred_val {
            crate::local_exec::Cell::Int(v) => u64_to_bits_lsb(*v as u64, const_bits.len()),
            crate::local_exec::Cell::Str(s) => {
                // encode string to bytes and to bits; pad/truncate to const_bits length
                let mut b = bytes_to_bits_lsb(s.as_bytes());
                b.resize(const_bits.len(), false);
                b
            }
            _ => vec![false; const_bits.len()],
        };

        // assemble map
        let mut map = HashMap::new();
        map.insert("salary".to_string(), salary_bits);
        map.insert("dept".to_string(), pred_bits);
        map.insert("const_dept".to_string(), const_bits.clone()); // same for all rows
        row_bit_maps.push(map);
    }

    // 8) Build the row template circuit
    println!("building row circuit template ...");
    let spec = build_row_update_circuit(SUM_W, SAL_W, CNT_W, const_bits.len());

    // 9) Run circuit template over rows
    println!("running per-row circuit over {} rows ...", row_bit_maps.len());
    let (sum_bits, count_bits) = run_row_circuit_over_rows(&spec, &row_bit_maps, SUM_W, CNT_W)
        .context("running row circuit")?;

    // decode outputs
    let sum_val = bits_le_to_u128(&sum_bits);
    let count_val = bits_le_to_u128(&count_bits);

    println!("RESULT: SUM (raw) = {}", sum_val);
    println!("RESULT: COUNT = {}", count_val);
    if count_val > 0 {
        println!("AVG = {:.6}", (sum_val as f64) / (count_val as f64));
    }

    // print bitstring for sum (LSB-first)
    let sum_bits_str: String = sum_bits.iter().map(|b| if *b {'1'} else {'0'}).collect();
    println!("SUM bits (LSB-first): {}", sum_bits_str);

    Ok(())
}

fn bits_le_to_u128(bits: &[bool]) -> u128 {
    let mut acc = 0u128;
    for (i, &b) in bits.iter().enumerate() {
        if b { acc |= 1u128 << i; }
    }
    acc
}
