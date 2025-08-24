mod logical_plan;
mod sql_to_logical;

mod logical_to_circuits;
mod circuit_builder;

mod local_exec;
mod binary_exec;
mod row_circuit;

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
    /*
    Fully working code for local plaintext SQL engine
     */
    // let csv_path = "data_analyst/src/employees.csv";
    // // Parse SQL -> LogicalPlan. Returns AST. Improvement idea: accept queries from CLI.
    // let sql = "SELECT AVG(salary) FROM employees WHERE dept = 'R&D'";

    // info!("Registering CSV as a table...");
    // let mut cat = Catalog::new();
    // cat.register_csv("employees", csv_path)?;

    // let logical = sql_to_logical_plan(sql)?;
    // info!("LogicalPlan: {:#?}", logical);

    /*
    Fully working code for translating phPlan into circuit, uncomment when sql optimizing is done
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

    run_partsupp::run_partsupp(None, None)?;

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
