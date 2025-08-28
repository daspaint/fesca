// data_analyst/src/grpc_client.rs
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::PathBuf;
use crate::table_schema::Schema;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub owner_dir: PathBuf,
    pub table_dir: PathBuf,
    pub table_name: String,
    pub row_count: u64,
    pub schema: Schema,
}

#[derive(Debug, Deserialize)]
struct ComputingNodes {
    node0_url: Option<String>,
    node1_url: Option<String>,
    node2_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct DataOwnerConfig {
    computing_nodes: ComputingNodes,
}

/// Public synchronous function used by data_analyst::run().
/// Internally runs a small tokio runtime to perform async gRPC RPCs.
pub fn find_table(table_name: &str, column_name: &str) -> Result<TableInfo> {
    // Try gRPC discovery first using the Data Owner config file
    let config_path = std::env::var("DATA_OWNER_CONFIG")
        .unwrap_or_else(|_| "../data_owner/config_data_owner.json".into());

    let mut tried_grpc = false;
    if let Ok(cfg_text) = fs::read_to_string(&config_path) {
        if let Ok(cfg) = serde_json::from_str::<DataOwnerConfig>(&cfg_text) {
            // Collect node URLs (skip None/empty)
            let mut node_urls = Vec::new();
            for opt in [&cfg.computing_nodes.node0_url, &cfg.computing_nodes.node1_url, &cfg.computing_nodes.node2_url] {
                if let Some(u) = opt {
                    if !u.trim().is_empty() {
                        node_urls.push(u.clone());
                    }
                }
            }

            if !node_urls.is_empty() {
                tried_grpc = true;
                match run_grpc_find_table(&node_urls, table_name, column_name) {
                    Ok(table_info) => return Ok(table_info),
                    Err(e) => {
                        // log and fall back to filesystem lookup below
                        log::warn!("gRPC find_table attempts failed: {}", e);
                    }
                }
            }
        } else {
            log::warn!("Could not parse Data Owner config at '{}', falling back to filesystem", config_path);
        }
    } else {
        log::info!("Data Owner config '{}' not found; falling back to filesystem lookup", config_path);
    }

    // Fallback: original filesystem lookup (unchanged)
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let pattern = format!("{}/fesca_shares/owner_*/{}$", home.display(), table_name);

    let mut matches = Vec::new();
    for entry in glob::glob(&pattern).context("Failed to glob fesca_shares pattern")? {
        if let Ok(path) = entry {
            matches.push(path);
        }
    }

    if matches.is_empty() {
        bail!("Check the spelling of the table. Are you sure data_owner splitted the shares?");
    }

    for table_dir in matches {
        let schema_path = table_dir.join("schema.json");
        if !schema_path.exists() {
            continue;
        }
        let data = fs::read_to_string(&schema_path)
            .with_context(|| format!("Failed to read schema.json at {}", schema_path.display()))?;
        let schema: Schema = serde_json::from_str(&data)
            .with_context(|| format!("Failed to parse schema.json at {}", schema_path.display()))?;

        if schema.table_name != table_name {
            continue;
        }
        let found = schema.columns.iter().any(|c| c.name == column_name);
        if !found {
            continue;
        }

        return Ok(TableInfo {
            owner_dir: table_dir.parent().unwrap_or_else(|| table_dir.as_path()).to_path_buf(),
            table_dir: table_dir.clone(),
            table_name: table_name.to_string(),
            row_count: schema.row_count,
            schema,
        });
    }

    bail!("Check the spelling of the table. Are you sure data_owner splitted the shares?");
}

/// Runs async gRPC calls in a tokio runtime. Returns first positive reply or Err.
fn run_grpc_find_table(node_urls: &[String], table_name: &str, column_name: &str) -> Result<TableInfo> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        for url in node_urls {
            log::info!("Attempting FindTable RPC to {}", url);

            // Use the generated find_table client (data_analyst crate compiled the proto into crate::find_table)
            let client_res = crate::find_table::table_lookup_client::TableLookupClient::connect(url.clone()).await;
            let mut client = match client_res {
                Ok(c) => c,
                Err(e) => {
                    log::warn!("Connect to {} failed: {}", url, e);
                    continue;
                }
            };

            let req = tonic::Request::new(crate::find_table::FindTableRequest {
                table_name: table_name.to_string(),
                column_name: column_name.to_string(),
            });

            let call = client.find_table(req);
            match tokio::time::timeout(std::time::Duration::from_secs(5), call).await {
                Ok(Ok(response)) => {
                    let inner = response.into_inner();
                    if inner.found {
                        // parse schema_json into local Schema
                        let schema: Schema = serde_json::from_str(&inner.schema_json)
                            .map_err(|e| anyhow::anyhow!("Failed to parse schema_json from node {}: {}", url, e))?;

                        let table_dir = PathBuf::from(&inner.table_dir);
                        let owner_dir = table_dir.parent().unwrap_or_else(|| table_dir.as_path()).to_path_buf();

                        return Ok(TableInfo {
                            owner_dir,
                            table_dir,
                            table_name: table_name.to_string(),
                            row_count: inner.row_count,
                            schema,
                        });
                    } else {
                        log::info!("Node {} reports not found", url);
                    }
                }
                Ok(Err(e)) => {
                    log::warn!("FindTable RPC to {} failed: {}", url, e);
                }
                Err(_) => {
                    log::warn!("FindTable RPC to {} timed out", url);
                }
            }
        }

        Err(anyhow::anyhow!("No computing nodes reported hosting the table"))
    })
}
