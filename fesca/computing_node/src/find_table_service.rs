// computing_node/src/find_table_service.rs
use tonic::{Request, Response, Status};
use std::path::PathBuf;
use glob::glob;
use std::fs;
use log::{info, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::find_table::table_lookup_server::TableLookup;
use crate::find_table::{
    FindTableRequest, FindTableResponse,
    ExtractAndForwardRequest, ExtractAndForwardResponse,
    ExtractAndComputeRequest, ExtractAndComputeResponse,
    ReceiveSharesRequest, ReceiveSharesResponse,
};

use crate::calculate::utils::{read_binary_party_data_bytes, deserialize_binary_party_data, extract_bits_as_u64};

#[derive(Clone)]
pub struct TableLookupService {
    pub base_dir: PathBuf,
    /// aggregator_store stores incoming ReceiveSharesRequest grouped by "table::column" key
    /// Used only when this node acts as aggregator.
    pub aggregator_store: Arc<Mutex<HashMap<String, Vec<ReceiveSharesRequest>>>>,
}

impl TableLookupService {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir, aggregator_store: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Helper: find a table directory that matches the given table_name and contains the column_name.
    /// Returns (table_dir, schema_json_string).
    fn find_local_table_dir_and_schema(&self, table_name: &str, column_name: &str) -> Result<(PathBuf, String), Status> {
        let pattern = format!("{}/owner_*/{}", self.base_dir.display(), table_name);
        for entry in glob(&pattern).map_err(|e| Status::internal(format!("glob error: {}", e)))? {
            let table_dir = match entry {
                Ok(p) => p,
                Err(_) => continue,
            };
            let schema_path = table_dir.join("schema.json");
            if !schema_path.exists() {
                continue;
            }
            let data = fs::read_to_string(&schema_path)
                .map_err(|e| Status::internal(format!("read schema.json error: {}", e)))?;
            let parsed: serde_json::Value = serde_json::from_str(&data)
                .map_err(|e| Status::internal(format!("parse schema.json error: {}", e)))?;
            let has_column = parsed.get("columns")
                .and_then(|v| v.as_array())
                .map(|cols| {
                    cols.iter().any(|c| {
                        c.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s == column_name)
                            .unwrap_or(false)
                    })
                })
                .unwrap_or(false);
            if !has_column {
                continue;
            }
            return Ok((table_dir, data));
        }
        Err(Status::not_found("table/column not found locally"))
    }

    /// Helper: given schema JSON string and a column_name return the column index.
    fn column_index_from_schema(schema_json: &str, column_name: &str) -> Result<usize, Status> {
        let parsed: serde_json::Value = serde_json::from_str(schema_json)
            .map_err(|e| Status::internal(format!("parse schema.json error: {}", e)))?;
        let column_index = parsed.get("columns")
            .and_then(|v| v.as_array())
            .and_then(|cols| {
                cols.iter().position(|c| {
                    c.get("name")
                        .and_then(|n| n.as_str())
                        .map(|nstr| nstr == column_name)
                        .unwrap_or(false)
                })
            })
            .ok_or_else(|| Status::not_found("column not found in schema"))?;
        Ok(column_index)
    }

    /// Helper: read the partyX_data.bin file bytes for the given party_id in table_dir.
    fn read_party_file_bytes(&self, table_dir: &PathBuf, party_id: u32) -> Result<Vec<u8>, Status> {
        let fname = format!("party{}_data.bin", party_id);
        let path = table_dir.join(fname);
        read_binary_party_data_bytes(&path).map_err(|e| Status::internal(format!("failed to read {}: {}", path.display(), e)))
    }

    /// Helper: given raw party bytes, extract per-row bitstring_a and bitstring_b and column offsets/lengths.
    /// Returns (as_vec, bs_vec, offsets_vec, lengths_vec).
    fn extract_column_bytes_from_party_bytes(
        &self,
        party_bytes: &[u8],
        column_index: usize,
    ) -> Result<(Vec<Vec<u8>>, Vec<Vec<u8>>, Vec<u32>, Vec<u32>), Status> {
        // Deserialize into BinaryPartyData using bincode or JSON.
        // This expects a local `crate::types::BinaryPartyData` with fields:
        //  - rows: Vec<BinaryRow>, where BinaryRow has bitstring_a: Vec<u8>, bitstring_b: Vec<u8>,
        //    column_bit_offsets: Vec<u32>, column_bit_lengths: Vec<u32>
        let party: crate::types::BinaryPartyData = deserialize_binary_party_data(party_bytes)
            .map_err(|e| Status::internal(format!("deserialize party bytes failed: {}", e)))?;

        let mut as_vec = Vec::with_capacity(party.rows.len());
        let mut bs_vec = Vec::with_capacity(party.rows.len());
        let mut offsets = Vec::new();
        let mut lengths = Vec::new();

        for row in party.rows.iter() {
            as_vec.push(row.bitstring_a.clone());
            bs_vec.push(row.bitstring_b.clone());
        }
        if let Some(first) = party.rows.get(0) {
            offsets = first.column_bit_offsets.clone();
            lengths = first.column_bit_lengths.clone();
        }
        Ok((as_vec, bs_vec, offsets, lengths))
    }
}

#[tonic::async_trait]
impl TableLookup for TableLookupService {
    /// Existing FindTable RPC — unchanged behavior
    async fn find_table(
        &self,
        req: Request<FindTableRequest>,
    ) -> Result<Response<FindTableResponse>, Status> {
        let r = req.into_inner();
        let table_name = r.table_name;
        let column_name = r.column_name;

        // Use helper
        match self.find_local_table_dir_and_schema(&table_name, &column_name) {
            Ok((table_dir, schema_json)) => {
                let row_count = serde_json::from_str::<serde_json::Value>(&schema_json)
                    .ok()
                    .and_then(|v| v.get("row_count").and_then(|rc| rc.as_u64()))
                    .unwrap_or(0);
                let resp = FindTableResponse {
                    found: true,
                    table_dir: table_dir.to_string_lossy().into_owned(),
                    row_count,
                    schema_json,
                };
                info!("Table '{}' with column '{}' found at '{}'", table_name, column_name, resp.table_dir);
                Ok(Response::new(resp))
            }
            Err(_) => {
                Ok(Response::new(FindTableResponse {
                    found: false,
                    table_dir: "".into(),
                    row_count: 0,
                    schema_json: "".into(),
                }))
            }
        }
    }

    /// Non-aggregator: extract its own column bitstrings and forward them to aggregator via ReceiveShares RPC.
    async fn extract_and_forward(
        &self,
        request: Request<ExtractAndForwardRequest>,
    ) -> Result<Response<ExtractAndForwardResponse>, Status> {
        let r = request.into_inner();
        let table_name = r.table_name;
        let column_name = r.column_name;
        let aggregator_url = r.aggregator_url;
        let party_id = r.party_id as u32;

        // locate local table_dir and schema
        let (table_dir, schema_json) = self.find_local_table_dir_and_schema(&table_name, &column_name)?;

        // compute column index
        let column_index = Self::column_index_from_schema(&schema_json, &column_name)?;

        // read own party bytes
        let party_bytes = self.read_party_file_bytes(&table_dir, party_id)?;

        // extract per-row bitstrings and offsets/lengths
        let (as_vec, bs_vec, offsets, lengths) =
            self.extract_column_bytes_from_party_bytes(&party_bytes, column_index)?;

        // Build a client to aggregator and send ReceiveSharesRequest
        let mut client = crate::find_table::table_lookup_client::TableLookupClient::connect(aggregator_url.clone())
            .await
            .map_err(|e| Status::internal(format!("connect to aggregator failed: {}", e)))?;

        let req_msg = crate::find_table::ReceiveSharesRequest {
            table_name: table_name.clone(),
            column_name: column_name.clone(),
            sender_party_id: party_id,
            // prost uses bytes as Vec<u8>, so we can move Vec<u8> directly
            bitstring_a: as_vec.into_iter().map(|v| v.into()).collect(),
            bitstring_b: bs_vec.into_iter().map(|v| v.into()).collect(),
            column_offsets: offsets.into_iter().map(|x| x as u32).collect(),
            column_lengths: lengths.into_iter().map(|x| x as u32).collect(),
        };

        client.receive_shares(tonic::Request::new(req_msg)).await
            .map_err(|e| Status::internal(format!("forward to aggregator failed: {}", e)))?;

        Ok(Response::new(ExtractAndForwardResponse { ok: true, message: "forwarded".into() }))
    }

    /// Aggregator: extract own shares, insert into the in-memory store, wait for 3 parties, reconstruct and compute SUM (stub).
    async fn extract_and_compute(
        &self,
        request: Request<ExtractAndComputeRequest>,
    ) -> Result<Response<ExtractAndComputeResponse>, Status> {
        let r = request.into_inner();
        let table_name = r.table_name;
        let column_name = r.column_name;
        let party_id = r.party_id as u32; // aggregator's own party id
        let row_count = r.row_count as usize;

        // locate local table_dir and schema
        let (table_dir, schema_json) = self.find_local_table_dir_and_schema(&table_name, &column_name)?;

        // compute column index
        let column_index = Self::column_index_from_schema(&schema_json, &column_name)?;

        // read own party bytes and extract as_vec/bs_vec and offsets/lengths
        let party_bytes = self.read_party_file_bytes(&table_dir, party_id)?;
        let (local_as_vec, local_bs_vec, offsets, lengths) =
            self.extract_column_bytes_from_party_bytes(&party_bytes, column_index)?;

        // Build a ReceiveSharesRequest representing our own shares and insert into store
        let key = format!("{}::{}", table_name, column_name);
        let my_req = crate::find_table::ReceiveSharesRequest {
            table_name: table_name.clone(),
            column_name: column_name.clone(),
            sender_party_id: party_id,
            bitstring_a: local_as_vec.into_iter().map(|v| v.into()).collect(),
            bitstring_b: local_bs_vec.into_iter().map(|v| v.into()).collect(),
            column_offsets: offsets.iter().map(|&x| x as u32).collect(),
            column_lengths: lengths.iter().map(|&x| x as u32).collect(),
        };

        {
            let mut store = self.aggregator_store.lock().unwrap();
            let entry = store.entry(key.clone()).or_insert_with(Vec::new);
            entry.push(my_req);
        }

        // wait until we have at least three distinct party ids for this key (simple polling; timeout could be added)
        loop {
            {
                let store = self.aggregator_store.lock().unwrap();
                if let Some(vecs) = store.get(&key) {
                    let mut ids: Vec<u32> = vecs.iter().map(|r| r.sender_party_id).collect();
                    ids.sort_unstable();
                    ids.dedup();
                    if ids.len() >= 3 {
                        break;
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }

        // fetch entries (clone)
        let entries = {
            let store = self.aggregator_store.lock().unwrap();
            store.get(&key).cloned().unwrap_or_default()
        };

        // Build lookup by party id
        let mut by_id: HashMap<u32, crate::find_table::ReceiveSharesRequest> = HashMap::new();
        for e in entries.into_iter() {
            by_id.entry(e.sender_party_id).or_insert(e);
        }

        // Require parties 0,1,2 present; adapt if your party ids differ
        let req0 = by_id.get(&0).ok_or_else(|| Status::internal("missing party 0 shares"))?;
        let req1 = by_id.get(&1).ok_or_else(|| Status::internal("missing party 1 shares"))?;
        let req2 = by_id.get(&2).ok_or_else(|| Status::internal("missing party 2 shares"))?;

        // offsets and lengths should be identical across entries; use req0
        let off_vec = &req0.column_offsets;
        let len_vec = &req0.column_lengths;
        let offset_bits = off_vec.get(column_index).copied().unwrap_or(0) as usize;
        let length_bits = len_vec.get(column_index).copied().unwrap_or(0) as usize;

        // sum over rows
        let mut sum: u128 = 0;
        for row_idx in 0..row_count {
            // Extract shares according to mapping P0:(a,b), P1:(b,c), P2:(a,c)
            let a = extract_bits_as_u64(&req0.bitstring_a[row_idx], offset_bits, length_bits)
                .map_err(|e| Status::internal(format!("extract a failed: {}", e)))?;
            let b = extract_bits_as_u64(&req0.bitstring_b[row_idx], offset_bits, length_bits)
                .map_err(|e| Status::internal(format!("extract b failed: {}", e)))?;
            let c = extract_bits_as_u64(&req1.bitstring_b[row_idx], offset_bits, length_bits)
                .map_err(|e| Status::internal(format!("extract c failed: {}", e)))?;

            let original = a ^ b ^ c;
            sum = sum.wrapping_add(original as u128);
        }

        // cleanup
        {
            let mut store = self.aggregator_store.lock().unwrap();
            store.remove(&key);
        }

        Ok(Response::new(ExtractAndComputeResponse {
            success: true,
            message: "computed".into(),
            sum_result: sum as u64, // cast — ensure sum fits u64 or change proto/type
        }))
    }

    /// RPC called by non-aggregators to deliver their extracted shares to aggregator.
    async fn receive_shares(
        &self,
        request: Request<ReceiveSharesRequest>,
    ) -> Result<Response<ReceiveSharesResponse>, Status> {
        let r = request.into_inner();
        let key = format!("{}::{}", r.table_name, r.column_name);
        {
            let mut store = self.aggregator_store.lock().unwrap();
            let entry = store.entry(key).or_insert_with(Vec::new);
            entry.push(r);
        }
        info!("Received shares for {}::{} from party {}", request.get_ref().table_name, request.get_ref().column_name, request.get_ref().sender_party_id);
        Ok(Response::new(ReceiveSharesResponse { ok: true, msg: "received".into() }))
    }
}
