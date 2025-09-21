mod parser;
// mod logical_to_circuits;
mod circuit_builder;
mod grpc_client;
pub mod table_schema;

use anyhow::Result;
use log::info;
use parser::extract_execution_plan;

pub mod find_table {
    tonic::include_proto!("find_table");
}

/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    // Parse SQL -> ExecutionPlan. Improvement idea: accept queries from CLI.
    let sql = "SELECT SUM(supply_cost) FROM partsupp";

    // extract_execution_plan now returns (table_name, column_name, agg_name)
    let (table_name, column_name, agg_name) = extract_execution_plan(sql)?;
    info!("Parsed query -> table: {}, column: {}, agg: {}", table_name, column_name, agg_name);

    // Validate that the requested table/column exist via gRPC call to computing node (filesystem stub)
    let table_info = grpc_client::find_table(&table_name, &column_name)
        .map_err(|e| anyhow::anyhow!("Failed to find table via computing node: {}", e))?;

    info!("Found table: {} with {} rows", table_info.table_name, table_info.row_count);

    // --- AUTOMATIC COMPUTE REQUEST START ---
    // Determine node URLs from Data Owner config (same logic as in grpc_client::find_table)
    let config_path = std::env::var("DATA_OWNER_CONFIG")
        .unwrap_or_else(|_| "data_owner/config_data_owner.json".into());

    // Read node URLs (best-effort; if missing, fall back to local filesystem-only flow)
    let mut node_urls: Vec<String> = Vec::new();
    if let Ok(cfg_text) = std::fs::read_to_string(&config_path) {
        if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&cfg_text) {
            if let Some(nodes) = cfg.get("computing_nodes") {
                if let Some(u) = nodes.get("node0_url").and_then(|v| v.as_str()) { if !u.is_empty() { node_urls.push(u.to_string()); } }
                if let Some(u) = nodes.get("node1_url").and_then(|v| v.as_str()) { if !u.is_empty() { node_urls.push(u.to_string()); } }
                if let Some(u) = nodes.get("node2_url").and_then(|v| v.as_str()) { if !u.is_empty() { node_urls.push(u.to_string()); } }
            }
        }
    }

    if node_urls.is_empty() {
        log::warn!("No computing node URLs available in config; skipping remote compute orchestration.");
        return Ok(());
    }

    // Start compute orchestration automatically
    match grpc_client::start_compute_request(&node_urls, &table_name, &column_name, &agg_name, table_info.row_count) {
        Ok(result) => {
            info!("Final aggregation result: {}", result);
        }
        Err(e) => {
            log::error!("Compute orchestration failed: {}", e);
        }
    }

    Ok(())
}
