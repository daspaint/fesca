# Receive Testing Server

This directory contains a gRPC server implementation for testing the reception and storage of table shares from data owners.

## Purpose

The receive testing server simulates computing nodes that receive secret shares from data owners. It provides:

1. **gRPC Service**: Implements the `ShareService` to receive table shares
2. **Persistent Storage**: Stores shares in `~/fesca_shares/` organized by data owner and table name
3. **Metadata Tracking**: Records data owner information, table schema, and storage metadata

## Directory Structure

```
~/fesca_shares/
├── {data_owner_id}/
│   └── {table_name}/
│       ├── party_0/
│       │   ├── schema_YYYYMMDD_HHMMSS.json      # Complete table schema
│       │   ├── party_data_YYYYMMDD_HHMMSS.json  # Complete SharedPartyData with all bit shares
│       │   └── metadata_YYYYMMDD_HHMMSS.json    # Metadata and file references
│       ├── party_1/
│       │   ├── schema_YYYYMMDD_HHMMSS.json
│       │   ├── party_data_YYYYMMDD_HHMMSS.json
│       │   └── metadata_YYYYMMDD_HHMMSS.json
│       └── party_2/
│           ├── schema_YYYYMMDD_HHMMSS.json
│           ├── party_data_YYYYMMDD_HHMMSS.json
│           └── metadata_YYYYMMDD_HHMMSS.json
```

## Usage

### Starting the Server

```bash
cd "receive testing"
cargo run
```

The server will start on `0.0.0.0:50051` by default.

### Testing with Multiple Nodes

To simulate the three computing nodes, start three instances on different ports:

```bash
# Terminal 1 - Node 0
GRPC_PORT=50051 cargo run

# Terminal 2 - Node 1  
GRPC_PORT=50052 cargo run

# Terminal 3 - Node 2
GRPC_PORT=50053 cargo run
```

### Storage Format

Data is stored in three separate files per party:

**Schema file** (`schema_YYYYMMDD_HHMMSS.json`):
```json
{
  "table_name": "partsupp",
  "table_id": 4,
  "row_count": 4,
  "columns": [
    {
      "name": "part_key",
      "type_hint": "UnsignedInt"
    },
    {
      "name": "supplier_key", 
      "type_hint": "UnsignedInt"
    }
  ]
}
```

**Party Data file** (`party_data_YYYYMMDD_HHMMSS.json`):
```json
{
  "party_id": 0,
  "table_id": 4,
  "rows": [
    {
      "entries": [
        {
          "bits": [
            {"share_a": true, "share_b": false},
            {"share_a": false, "share_b": true}
          ]
        }
      ]
    }
  ]
}
```

**Metadata file** (`metadata_YYYYMMDD_HHMMSS.json`):
```json
{
  "data_owner": {
    "owner_id": "owner_001",
    "owner_name": "First Data Owner",
    "timestamp": 1234567890
  },
  "storage_metadata": {
    "stored_at": "2024-01-01T12:00:00Z",
    "schema_file": "~/fesca_shares/owner_001/partsupp/party_0/schema_20240101_120000.json",
    "party_data_file": "~/fesca_shares/owner_001/partsupp/party_0/party_data_20240101_120000.json",
    "storage_path": "~/fesca_shares/owner_001/partsupp/party_0"
  }
}
```

## Configuration

The server uses the same protobuf schema as the data owner client. Make sure both are using the same `share_service.proto` file.

## Notes

- This implementation stores the complete schema and all bit shares for testing
- In production, you would add encryption and more secure storage
- The storage is now in the user's home directory for easier testing
- The server is configured to accept connections from any IP address for testing purposes 