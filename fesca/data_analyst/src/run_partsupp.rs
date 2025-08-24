// data_analyst/src/run_partsupp.rs
// Callable from data_analyst::run()
// Reads binary folder (schema + blob), reconstructs rows, builds per-row circuit and evaluates it locally.

use anyhow::{Result, Context};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::PathBuf;

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

/// Local in-module cell representation for reconstructed data
#[derive(Debug, Clone)]
enum CellValue {
    Int(i64),
    Float(f64),
    Str(String),
}

fn parse_type_hint(s: Option<&str>) -> ColType {
    let s = s.unwrap_or("");
    if s.contains("UnsignedInt") {
        ColType::UnsignedInt64
    } else if s.contains("Float(") || s.contains("FloatType") {
        ColType::Float64
    } else if s.contains("String(") || s.contains("StringType") {
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

fn u64_to_bits_lsb(mut v: u64, width: usize) -> Vec<bool> {
    let mut bits = vec![false; width];
    for i in 0..width {
        bits[i] = (v & 1) == 1;
        v >>= 1;
    }
    bits
}

fn i64_to_bits_lsb(v: i64, width: usize) -> Vec<bool> {
    u64_to_bits_lsb(v as u64, width)
}

fn bytes_to_bits_lsb(bytes: &[u8]) -> Vec<bool> {
    let mut bits = Vec::with_capacity(bytes.len()*8);
    for &b in bytes.iter() {
        for i in 0..8 {
            bits.push(((b >> i) & 1) == 1);
        }
    }
    bits
}

fn bits_le_to_u128(bits: &[bool]) -> u128 {
    let mut acc = 0u128;
    for (i, &b) in bits.iter().enumerate() {
        if b { acc |= 1u128 << i; }
    }
    acc
}

/// Public entrypoint: read the `folder` (if None use default path in crate), parse `sql` (if None use SQL from lib.rs),
/// reconstruct rows from binary, build per-row circuit and evaluate locally.
/// Returns Result<()>
pub fn run_partsupp(folder: Option<PathBuf>, sql: Option<&str>) -> Result<()> {
    // default folder relative to crate root
    let default_folder = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src").join("binary_data").join("owner_001").join("partsupp");
    let folder = folder.unwrap_or(default_folder);

    let sql = match sql {
        Some(s) => s.to_string(),
        None => {
            // fallback: use the SQL string you kept in lib.rs earlier
            // If lib.rs defines a SQL constant, call it or copy here; for safety we use a default that matches the partsupp table
            "SELECT AVG(supply_cost) FROM partsupp WHERE part_key = 1".to_string()
        }
    };

    info!("Reading binary table at folder = {}", folder.display());
    info!("Processing SQL query = {}", sql);

    // locate schema.json and binary blob
    let mut schema_file: Option<PathBuf> = None;
    let mut bin_file: Option<PathBuf> = None;
    for entry in read_dir(&folder).with_context(|| format!("reading folder {:?}", folder))? {
        let e = entry?;
        let p = e.path();
        if p.is_file() {
            if p.extension().and_then(|s| s.to_str()).map(|s| s.eq_ignore_ascii_case("json")).unwrap_or(false) {
                schema_file = Some(p.clone());
            } else {
                if bin_file.is_none() {
                    bin_file = Some(p.clone());
                }
            }
        }
    }
    let schema_file = schema_file.ok_or_else(|| anyhow::anyhow!("schema JSON not found in folder {:?}", folder))?;
    let bin_file = bin_file.ok_or_else(|| anyhow::anyhow!("binary data file not found in folder {:?}", folder))?;

    // parse schema.json
    let mut s = String::new();
    File::open(&schema_file).with_context(|| format!("opening schema {:?}", schema_file))?.read_to_string(&mut s)?;
    let raw: RawSchema = serde_json::from_str(&s).context("parsing schema json")?;
    let row_count = raw.row_count.unwrap_or(0usize);
    let col_types: Vec<ColType> = raw.columns.iter().map(|c| parse_type_hint(c.type_hint.as_deref())).collect();
    let col_names: Vec<String> = raw.columns.iter().map(|c| c.name.clone()).collect();

    // col sizes and row_size
    let col_sizes: Vec<usize> = col_types.iter().map(|ct| match ct {
        ColType::UnsignedInt64 => 8usize,
        ColType::Float64 => 8usize,
        ColType::FixedString(n) => *n,
    }).collect();
    let row_size: usize = col_sizes.iter().sum();

    // read blob
    let mut blob = Vec::new();
    File::open(&bin_file).with_context(|| format!("opening binary {:?}", bin_file))?.read_to_end(&mut blob)?;
    if blob.len() < row_count * row_size {
        error!("warning: blob length {} smaller than expected {} (row_count*row_size). continuing best-effort", blob.len(), row_count * row_size);
    }

    // reconstruct rows as Vec<HashMap<String, CellValue>>
    let mut rows: Vec<HashMap<String, CellValue>> = Vec::new();
    for r in 0..row_count {
        let base = r * row_size;
        if base + row_size > blob.len() {
            error!("stopping early: row {} would go past blob end", r);
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
                ColType::FixedString(_len) => {
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

    info!("Reconstructed {} rows from {:?}", rows.len(), bin_file);

    // parse SQL into logical plan
    let plan = crate::sql_to_logical::sql_to_logical_plan(&sql)
        .with_context(|| "parsing SQL into logical plan")?;

    // Extract pattern: Aggregate(AVG) with Filter(Eq) and inner Scan
    // returns (aggr_col_name, pred_col_name, pred_literal_expr)
    let (aggr_col_name, pred_col_name, pred_literal_expr) = match &plan {
        crate::logical_plan::LogicalPlan::Aggregate { input, aggr_exprs, .. } => {
            if aggr_exprs.len() != 1 {
                anyhow::bail!("only single AVG aggregate supported");
            }
            // check it's AVG
            let (func, expr, _alias) = &aggr_exprs[0];
            match func {
                crate::logical_plan::AggregateFunc::Avg => {}
                other => anyhow::bail!("only AVG supported"),
            }
            // aggregate expression must be ColumnName or Column(index)
            let aggr_col = match expr {
                crate::logical_plan::Expr::ColumnName(n) => n.clone(),
                crate::logical_plan::Expr::Column(idx) => {
                    let idx = *idx;
                    if idx >= col_names.len() { anyhow::bail!("aggregate column index {} out of range", idx) }
                    col_names[idx].clone()
                }
                _ => anyhow::bail!("unsupported aggregate expr: {:?}", expr),
            };

            // now extract predicate from input (expect Filter{predicate, input:Scan})
            match &**input {
                crate::logical_plan::LogicalPlan::Filter { predicate, input: _ } => {
                    // predicate must be BinaryOp Eq with column & literal
                    match predicate {
                        crate::logical_plan::Expr::BinaryOp { op, left, right } => {
                            match op {
                                crate::logical_plan::BinaryOperator::Eq => {}
                                _ => anyhow::bail!("only equality predicate supported"),
                            }
                            // find (colname, literal)
                            let (colname, lit_expr) = match (&**left, &**right) {
                                (crate::logical_plan::Expr::ColumnName(n), other_lit) => (n.clone(), other_lit.clone()),
                                (crate::logical_plan::Expr::Column(idx), other_lit) => {
                                    let idx = *idx;
                                    if idx >= col_names.len() { anyhow::bail!("predicate column idx out of range"); }
                                    (col_names[idx].clone(), other_lit.clone())
                                }
                                (other_lit, crate::logical_plan::Expr::ColumnName(n)) => (n.clone(), other_lit.clone()),
                                (other_lit, crate::logical_plan::Expr::Column(idx)) => {
                                    let idx = *idx;
                                    if idx >= col_names.len() { anyhow::bail!("predicate column idx out of range"); }
                                    (col_names[idx].clone(), other_lit.clone())
                                }
                                (l,r) => anyhow::bail!("unsupported predicate forms: {:?} {:?}", l, r),
                            };
                            (aggr_col, colname, lit_expr)
                        }
                        other => anyhow::bail!("unsupported predicate: {:?}", other),
                    }
                }
                _ => anyhow::bail!("expected Filter over Scan as input to Aggregate"),
            }
        }
        _ => anyhow::bail!("unsupported logical plan: expected Aggregate(...)"),
    };

    info!("Plan -> AVG column: {}, predicate: {} == {:?}", aggr_col_name, pred_col_name, pred_literal_expr);

    // decide bit widths for sum/salary/count
    let SUM_W = 96usize;
    let SAL_W = 56usize;
    let CNT_W = 32usize;

    // build constant bits for predicate literal
    let const_bits: Vec<bool> = match &pred_literal_expr {
        crate::logical_plan::Expr::LiteralInt(v) => {
            let u = *v as i64 as u64;
            u64_to_bits_lsb(u, 64)
        }
        crate::logical_plan::Expr::LiteralString(s) => {
            let b = s.as_bytes();
            bytes_to_bits_lsb(b)
        }
        other => anyhow::bail!("unsupported predicate literal {:?}", other),
    };

    // prepare row_bit_maps expected by row circuit: keys "salary", "dept", "const_dept"
    let mut row_bit_maps: Vec<HashMap<String, Vec<bool>>> = Vec::new();
    for row in rows.iter() {
        // get aggregate value
        let aggr_val = row.get(&aggr_col_name).ok_or_else(|| anyhow::anyhow!("aggregate column {} missing in row", aggr_col_name))?;
        // get predicate column value
        let pred_val = row.get(&pred_col_name).ok_or_else(|| anyhow::anyhow!("predicate column {} missing in row", pred_col_name))?;

        // salary bits: if float -> cents; if int -> direct
        let salary_bits: Vec<bool> = match aggr_val {
            CellValue::Int(v) => u64_to_bits_lsb(*v as u64, SAL_W),
            CellValue::Float(f) => {
                // convert to cents (round)
                let cents = (f * 100.0).round() as i64;
                u64_to_bits_lsb(cents as u64, SAL_W)
            }
            CellValue::Str(_) => {
                // unsupported: treat as zero
                vec![false; SAL_W]
            }
        };

        // pred bits sized to const_bits.len()
        let pred_bits: Vec<bool> = match pred_val {
            CellValue::Int(v) => {
                let mut b = u64_to_bits_lsb(*v as u64, const_bits.len());
                b.resize(const_bits.len(), false);
                b
            }
            CellValue::Float(f) => {
                let cents = (f * 100.0).round() as i64;
                let mut b = u64_to_bits_lsb(cents as u64, const_bits.len());
                b.resize(const_bits.len(), false);
                b
            }
            CellValue::Str(s) => {
                let mut b = bytes_to_bits_lsb(s.as_bytes());
                b.resize(const_bits.len(), false);
                b
            }
        };

        let mut m = HashMap::new();
        m.insert("salary".to_string(), salary_bits);
        m.insert("dept".to_string(), pred_bits);
        m.insert("const_dept".to_string(), const_bits.clone());
        row_bit_maps.push(m);
    }

    info!("Prepared {} row bit-maps", row_bit_maps.len());

    // build template circuit once
    let spec = build_row_update_circuit(SUM_W, SAL_W, CNT_W, const_bits.len());

    // run per-row circuit over rows
    let (sum_bits, count_bits) = run_row_circuit_over_rows(&spec, &row_bit_maps, SUM_W, CNT_W)
        .context("running per-row circuit")?;

    // decode and print
    let sum_val = bits_le_to_u128(&sum_bits);
    let count_val = bits_le_to_u128(&count_bits);

    println!("RESULT: SUM (raw int) = {}", sum_val);
    println!("RESULT: COUNT = {}", count_val);
    if count_val > 0 {
        println!("AVG ~= {:.6}", (sum_val as f64) / (count_val as f64));
    }

    let sum_bits_str: String = sum_bits.iter().map(|b| if *b { '1' } else { '0' }).collect();
    println!("SUM bits (LSB-first): {}", sum_bits_str);

    Ok(())
}
