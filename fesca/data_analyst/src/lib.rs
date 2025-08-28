mod parser;
// mod logical_to_circuits;
mod circuit_builder;
mod grpc_client;

use anyhow::Result;
use log::info;
use parser::extract_execution_plan;

/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    // Parse SQL -> ExecutionPlan. Improvement idea: accept queries from CLI.
    let sql = "SELECT SUM(supply_cost) FROM partsupp";

    // extract_execution_plan now returns (table_name, column_name, agg_name)
    let (table_name, column_name, agg_name) = extract_execution_plan(sql)?;
    info!("Execution details -> table: {}, column: {}, agg: {}", table_name, column_name, agg_name);

    // Validate that the requested table/column exist via gRPC call to computing node (filesystem stub)
    let table_info = grpc_client::find_table(&table_name, &column_name)
        .map_err(|e| anyhow::anyhow!("Failed to find table via computing node: {}", e))?;

    // TODO: compile logical plan to circuit
    // let circuit = logical_to_circuits::compile_to_circuit(&logical, 5, 2);
    // info!("Circuit wire_count = {}", circuit.wire_count);

    info!("Found table: {} with {} rows", table_info.table_name, table_info.row_count);

    Ok(())
}
