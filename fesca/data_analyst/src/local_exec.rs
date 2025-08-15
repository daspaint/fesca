/*
This is a minimal "DBMS-like" local system for reading SQL queries and transforming them into logical plans.
This was built for testing and optimizing the SQL query compilation pipeline.
It supports basic SQL features like SELECT, WHERE, GROUP BY, and aggregates.
Loads CSV files as tables, resolves column names to indices at execution time,
and executes logical plans against a local catalog of tables.
 */
use anyhow::{bail, Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

use crate::logical_plan::{
    AggregateFunc, BinaryOperator, Expr as LPExpr, LogicalPlan,
};

#[derive(Debug, Clone)]
pub enum Cell {
    Int(i64),
    Str(String),
}

#[derive(Debug, Clone)]
pub struct TableData {
    pub name: String,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>, // rows[row_idx][col_idx]
}

#[derive(Default)]
pub struct Catalog {
    tables: HashMap<String, TableData>,
}

impl Catalog {
    pub fn new() -> Self { Self { tables: HashMap::new() } }

    /// Register a CSV file as a table (header required).
    pub fn register_csv<P: AsRef<Path>>(&mut self, table_name: &str, path: P) -> Result<()> {
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .has_headers(true)
            .from_reader(File::open(&path)
                .with_context(|| format!("opening CSV {:?}", path.as_ref()))?);

        let headers = rdr
            .headers()
            .context("reading CSV headers")?
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let mut rows = Vec::new();
        for rec in rdr.records() {
            let r = rec?;
            let mut row = Vec::with_capacity(headers.len());
            for field in r.iter() {
                if let Ok(v) = field.parse::<i64>() {
                    row.push(Cell::Int(v));
                } else {
                    row.push(Cell::Str(field.to_string()));
                }
            }
            // pad short rows (if any) with empty strings
            while row.len() < headers.len() { row.push(Cell::Str(String::new())); }
            rows.push(row);
        }

        let t = TableData { name: table_name.to_string(), columns: headers, rows };
        self.tables.insert(table_name.to_string(), t);
        Ok(())
    }

    pub fn get(&self, table_name: &str) -> Option<&TableData> {
        self.tables.get(table_name)
    }
}

/// Execute a logical plan against the local catalog.
/// For simplicity this returns either a scalar (for aggregates)
/// or a vector of rows (for projection).
#[derive(Debug)]
pub enum ExecResult {
    /// For SELECT with aggregates: one row with 1..N scalar cells
    Row(Vec<Cell>),
    /// For projection without aggregates
    Rows { columns: Vec<String>, rows: Vec<Vec<Cell>> },
}

pub fn execute(plan: &LogicalPlan, catalog: &Catalog) -> Result<ExecResult> {
    match plan {
        LogicalPlan::Scan { table_name, .. } => {
            let t = catalog.get(table_name)
                .ok_or_else(|| anyhow::anyhow!("unknown table {}", table_name))?;
            Ok(ExecResult::Rows { columns: t.columns.clone(), rows: t.rows.clone() })
        }

        LogicalPlan::Filter { input, predicate } => {
            let res = execute(input, catalog)?;
            match res {
                ExecResult::Rows { columns, rows } => {
                    // keep rows where predicate(row) == true
                    let mut out = Vec::new();
                    for row in rows {
                        if eval_bool(predicate, &columns, &row)? {
                            out.push(row);
                        }
                    }
                    Ok(ExecResult::Rows { columns, rows: out })
                }
                _ => bail!("Filter expects row set"),
            }
        }

        LogicalPlan::Project { input, exprs } => {
            let res = execute(input, catalog)?;
            match res {
                ExecResult::Rows { columns, rows } => {
                    // create new column names
                    let out_cols = exprs.iter().map(|(_e, alias)| {
                        alias.clone().unwrap_or_else(|| "expr".to_string())
                    }).collect::<Vec<_>>();

                    let mut out_rows = Vec::with_capacity(rows.len());
                    for row in rows.iter() {
                        let mut new_row = Vec::with_capacity(exprs.len());
                        for (e, _alias) in exprs {
                            new_row.push(eval_expr(e, &columns, row)?);
                        }
                        out_rows.push(new_row);
                    }
                    Ok(ExecResult::Rows { columns: out_cols, rows: out_rows })
                }
                _ => bail!("Project expects row set"),
            }
        }

        LogicalPlan::Aggregate { input, group_exprs, aggr_exprs } => {
            if !group_exprs.is_empty() {
                bail!("GROUP BY not implemented in this minimal executor");
            }

            let res = execute(input, catalog)?;
            let (columns, rows) = match res {
                ExecResult::Rows { columns, rows } => (columns, rows),
                _ => bail!("Aggregate expects row set"),
            };

            // Only a single final row with the aggregate results
            let mut out = Vec::with_capacity(aggr_exprs.len());
            for (func, expr, _alias) in aggr_exprs {
                match func {
                    AggregateFunc::Avg => {
                        // avg over numeric cells
                        let mut sum: i128 = 0;
                        let mut cnt: i128 = 0;
                        for row in rows.iter() {
                            if let Cell::Int(v) = eval_expr(expr, &columns, row)? {
                                sum += v as i128;
                                cnt += 1;
                            }
                        }
                        if cnt == 0 {
                            out.push(Cell::Int(0)); // or Str("NaN")
                        } else {
                            out.push(Cell::Int((sum / cnt) as i64));
                        }
                    }
                    AggregateFunc::Sum => {
                        let mut sum: i128 = 0;
                        for row in rows.iter() {
                            if let Cell::Int(v) = eval_expr(expr, &columns, row)? {
                                sum += v as i128;
                            }
                        }
                        out.push(Cell::Int(sum as i64));
                    }
                    AggregateFunc::Count => {
                        out.push(Cell::Int(rows.len() as i64));
                    }
                }
            }
            Ok(ExecResult::Row(out))
        }
    }
}

fn eval_bool(expr: &LPExpr, cols: &[String], row: &[Cell]) -> Result<bool> {
    match eval_expr(expr, cols, row)? {
        Cell::Int(v) => Ok(v != 0),
        Cell::Str(s) => Ok(!s.is_empty()),
    }
}

fn eval_expr(expr: &LPExpr, cols: &[String], row: &[Cell]) -> Result<Cell> {
    match expr {
        LPExpr::ColumnName(name) => {
            let idx = cols.iter().position(|c| c == name)
                .ok_or_else(|| anyhow::anyhow!("unknown column {}", name))?;
            Ok(row[idx].clone())
        }
        LPExpr::Column(idx) => Ok(row[*idx].clone()), // if you still use it elsewhere
        LPExpr::LiteralInt(v) => Ok(Cell::Int(*v as i64)),
        LPExpr::LiteralString(s) => Ok(Cell::Str(s.clone())),
        LPExpr::BinaryOp { op, left, right } => {
            match (eval_expr(left, cols, row)?, eval_expr(right, cols, row)?) {
                (Cell::Int(a), Cell::Int(b)) => match op {
                    BinaryOperator::Plus => Ok(Cell::Int(a + b)),
                    BinaryOperator::Eq => Ok(Cell::Int((a == b) as i64)),
                    BinaryOperator::And => Ok(Cell::Int(((a != 0) && (b != 0)) as i64)),
                },
                (Cell::Str(a), Cell::Str(b)) => match op {
                    BinaryOperator::Eq => Ok(Cell::Int((a == b) as i64)),
                    _ => bail!("unsupported string op {:?}", op),
                },
                (Cell::Int(a), Cell::Str(b)) | (Cell::Str(b), Cell::Int(a)) => {
                    match op {
                        BinaryOperator::Eq => Ok(Cell::Int((a.to_string() == b) as i64)),
                        _ => bail!("type mismatch for {:?}", op),
                    }
                }
            }
        }
    }
}
