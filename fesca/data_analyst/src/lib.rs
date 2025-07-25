mod logical_plan;
mod sql_to_logical;
mod logical_to_circuits;
mod circuit_builder;

use anyhow::{Result, bail};
use log::info;
// use logical_plan::{Expr as LPExpr, BinaryOperator, LogicalPlan, AggregateFunc};
use logical_to_circuits::compile_to_circuit;
use sql_to_logical::sql_to_logical_plan;

// use sqlparser::dialect::GenericDialect;
// use sqlparser::parser::Parser;
// use sqlparser::ast::{
//     Statement, Query, SetExpr, SelectItem, TableWithJoins, TableFactor,
//     Expr as AstExpr, Value as AstValue, BinaryOperator as AstOp,
//     Function as AstFunction, FunctionArg, FunctionArgExpr
// };


/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    // Parse SQL -> LogicalPlan. Returns AST. Improvement idea: accept queries from CLI.
    let sql = "SELECT AVG(salary) FROM employees WHERE dept = 'R&D'";
    let logical = sql_to_logical_plan(sql)?;
    info!("LogicalPlan: {:#?}", logical);

    // Build circuit for e.g. 5 rows × 2 columns. Improvement idea: read table size dynamically from existing dataset.
    let circuit = compile_to_circuit(&logical, 5, 2);
    info!("Circuit wire_count = {}", circuit.wire_count);
    info!("Circuit gates count = {}", circuit.gates.len());
    info!("Circuit outputs = {:?}", circuit.outputs);

    //log the circuit structure
    info!("Circuit gates: {:#?}", circuit.gates);

    // Log each gate
    for g in &circuit.gates {
        info!("Gate: {:?}", g);
    }
    Ok(())
}
