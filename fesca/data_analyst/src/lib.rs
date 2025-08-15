mod logical_plan;
mod sql_to_logical;

mod logical_to_circuits;
mod circuit_builder;

mod local_exec;

use anyhow::{Result, bail};
use log::info;
// use logical_plan::{Expr as LPExpr, BinaryOperator, LogicalPlan, AggregateFunc};
use logical_to_circuits::compile_to_circuit;
use sql_to_logical::sql_to_logical_plan;
use local_exec::{Catalog, execute, ExecResult, Cell}; // TODO: comment or delete when sql optimizing is done

// use sqlparser::dialect::GenericDialect;
// use sqlparser::parser::Parser;
// use sqlparser::ast::{
//     Statement, Query, SetExpr, SelectItem, TableWithJoins, TableFactor,
//     Expr as AstExpr, Value as AstValue, BinaryOperator as AstOp,
//     Function as AstFunction, FunctionArg, FunctionArgExpr
// };


/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    let csv_path = "fesca/data_analyst/src/employees.csv";
    // Parse SQL -> LogicalPlan. Returns AST. Improvement idea: accept queries from CLI.
    let sql = "SELECT AVG(salary) FROM employees WHERE dept = 'R&D'";

    info!("Registering CSV as a table...");
    let mut cat = Catalog::new();
    cat.register_csv("employees", csv_path)?;

    let logical = sql_to_logical_plan(sql)?;
    info!("LogicalPlan: {:#?}", logical);

    /*
    Fully working code, uncomment when sql optimizing is done
     */
    // // Build circuit for e.g. 5 rows × 2 columns. Improvement idea: read table size dynamically from existing dataset.
    // // Better: estimate how big is the row, and apply the same function to each row.
    // let circuit = compile_to_circuit(&logical, 5, 2);
    // info!("Circuit wire_count = {}", circuit.wire_count);
    // info!("Circuit gates count = {}", circuit.gates.len());
    // info!("Circuit outputs = {:?}", circuit.outputs);

    // //log the circuit structure
    // info!("Circuit gates: {:#?}", circuit.gates);

    // // Log each gate
    // for g in &circuit.gates {
    //     info!("Gate: {:?}", g);
    // }

    // Execute the sql query on our local "dbms"
    let result = execute(&logical, &cat)?;
    info!("Execution complete");

    // Print result of sql query
    match result {
        ExecResult::Row(cells) => {
            println!("{}", display_row(&cells));
        }
        ExecResult::Rows { columns, rows } => {
            println!("{}", columns.join(","));
            for r in rows {
                println!("{}", display_row(&r));
            }
        }
    }

    Ok(())
}

/*
Displays the test table data in a human-readable format.
 */
fn display_row(row: &[Cell]) -> String {
    row.iter()
        .map(|c| match c {
            Cell::Int(v) => v.to_string(),
            Cell::Str(s) => s.clone(),
        })
        .collect::<Vec<_>>()
        .join(",")
}
