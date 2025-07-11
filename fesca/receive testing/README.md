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
│       │   └── shares_YYYYMMDD_HHMMSS.json
│       ├── party_1/
│       │   └── shares_YYYYMMDD_HHMMSS.json
│       └── party_2/
│           └── shares_YYYYMMDD_HHMMSS.json
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

Each share file contains:

```json
{
  "data_owner": {
    "owner_id": "owner_001",
    "owner_name": "Default Data Owner",
    "contact_info": "owner@example.com",
    "timestamp": 1234567890
  },
  "schema": {
    "table_name": "users",
    "table_id": 1,
    "row_count": 1000,
    "columns": [...]
  },
  "party_data": {
    "party_id": 0,
    "table_id": 1,
    "row_count": 1000,
    "shares_stored": true
  },
  "storage_metadata": {
    "stored_at": "2024-01-01T12:00:00Z",
    "storage_path": "~/fesca_shares/owner_001/users/party_0/shares_20240101_120000.json"
  }
}
```

## Configuration

The server uses the same protobuf schema as the data owner client. Make sure both are using the same `share_service.proto` file.

## Notes

- This is a testing implementation and stores metadata only
- In production, you would store the actual bit shares securely
- The storage is now in the user's home directory for easier testing
- The server is configured to accept connections from any IP address for testing purposes 