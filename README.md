# FESCA — Framework for Efficient Secure Collaborative Analytics

FESCA is a modular prototype of a relational MPC (Multi-Party Computation) system written in **Rust**. It lets multiple data owners run SQL-style analytics over secret-shared data across **three computing nodes** without revealing raw inputs. The system focuses on a clean, extensible architecture, practical transport (gRPC/HTTP-2), and measurable performance on a LAN.

> Paper context: FESCA was inspired by **SECRECY (NSDI’23)** but built from scratch in Rust with a modular design, gRPC transport, and a built-in benchmarking workflow.

---

## Table of contents

* [Architecture](#architecture)
* [Crates & binaries](#crates--binaries)
* [Getting started](#getting-started)
* [Configuration](#configuration)
* [Running the system](#running-the-system)
* [Benchmarking gRPC/TCP](#benchmarking-grpctcp)
* [Correlated randomness & key exchange](#correlated-randomness--key-exchange)
* [What we measured (quick guidance)](#what-we-measured-quick-guidance)
* [Repo layout](#repo-layout)
* [Roadmap](#roadmap)

---

## Architecture

FESCA follows a **3-party MPC** layout with **replicated/XOR secret sharing**:

* **Data Owner (DO):** reads a local table, encodes it to a binary format, **secret-shares** each bit into three shares `a,b,c` (with `a ⊕ b ⊕ c = value`), and ships the shares to the computing nodes over gRPC.
* **Computing Nodes (CN ×3):** store their assigned shares; validate table/column; translate the requested operation into a Boolean circuit; execute secure computation; send back results.
* **Data Analyst (DA):** accepts a (public) SQL-like query, builds a minimal logical plan (table/column/aggregation), verifies availability with DO/CN via gRPC, triggers computation, and **reconstructs** the result from the three replies.

Transport is **gRPC/HTTP-2** on port **50051**. A lightweight benchmarking tool and optional TCP echo server are included.

---

## Crates & binaries

* `data_owner/` — Data Owner library & runner
* `data_analyst/` — Data Analyst library & runner
* `computing_node/` — Computing Node library & gRPC server implementations
* `helpers/` — shared utilities
* `fesca/` — workspace binary:

  * `cargo run -- data_owner` (DO)
  * `cargo run -- computing_node` (CN)
  * `cargo run -- data_analyst` (DA)
  * `cargo run --bin benchmark` (standalone benchmark tool, for this checkout to branch 50-evaluate)

---

## Getting started

### Prerequisites

* Rust toolchain (stable)
* Access to three hosts (or three terminals) for CNs + one host for DO/DA
* LAN connectivity between hosts

### Build

```bash
# From repository root
cargo build
```

---

## Configuration

### Data Owner config

`data_owner/config_data_owner.json`

```json
{
  "computing_nodes": {
    "node0_url": "http://<CN0_HOST>:50051",
    "node1_url": "http://<CN1_HOST>:50051",
    "node2_url": "http://<CN2_HOST>:50051"
  }
}
```

Set env var (optional):

```bash
export DATA_OWNER_CONFIG=data_owner/config_data_owner.json
```

### Computing Node config

`computing_node/config_computing_node.json`
Contains node identity, storage path, and (optionally) endpoints used by key-exchange/correlated-randomness.

Default gRPC port: **50051**
Default storage path: `~/fesca_shares`

---

## Running the system

> Order: **Data Owner → 3× Computing Node → Data Analyst**

1. **Start Data Owner (DO)**

```bash
cargo run -- data_owner
```

* Loads a local table (`*.tbl`) + its JSON metadata.
* Encodes to binary, splits into `a,b,c` shares.
* Sends shares to the three CNs via gRPC.

2. **Start Computing Nodes (CNs) on three hosts**

```bash
# On each of three hosts
cargo run -- computing_node
```

* Starts gRPC server on `0.0.0.0:50051`
* Services: receive shares, find table, compute/extract, (optional) key-exchange & CR

3. **Start Data Analyst (DA)**

```bash
cargo run -- data_analyst
```

* Accepts a public SQL-like query (e.g., `SELECT SUM(supply_cost) FROM partsupp`)
* Extracts `(table, column, aggregation)` and validates via gRPC
* Triggers the compute protocol and reconstructs the result

---

## Benchmarking gRPC/TCP

A separate binary **`benchmark`** exercises the transport layer independently of MPC. For this checkout the branch **`50-evaluate`**.

### gRPC latency/goodput vs payload (CN must be running)

```bash
cargo run --bin benchmark -- \
  grpc \
  --target http://<CN_HOST>:50051 \
  --size 10000 \
  --iters 500 \
  --conc 16 \
  --warmup 50
```

### Concurrency sweep (fixed size)

```bash
for c in 1 4 16 32 64 128; do
  cargo run --bin benchmark -- grpc \
    --target http://<CN_HOST>:50051 \
    --size 10000 --iters 1000 --conc $c --warmup 50
done
```

### TCP baseline (optional)

Start TCP echo on CN host:

```bash
TCP_ECHO_ADDR=0.0.0.0:6000 cargo run -- computing_node
```

Then from benchmark host:

```bash
cargo run --bin benchmark -- \
  tcp --target <CN_HOST>:6000 \
  --size 10000 --iters 500 --conc 16 --warmup 50
```

### Serialization-only (Protobuf)

```bash
cargo run --bin benchmark -- serde --size 10000 --iters 10000
# Example result:
# Protobuf 10000B -> enc ~0.290 µs/op, dec ~1.027 µs/op
```

---

## Correlated randomness & key exchange

FESCA includes a simple **key exchange** service that lets each node exchange small seeds over gRPC and then locally expand them into **correlated randomness** (CR) via a PRG. With CR available:

* **XOR** gates are local (no rounds).
* **AND** gates use one masked open per layer (one synchronization round), leveraging **AND triples** provided by CR.

This aligns with the transport evaluation: we batch many gates per round (≈100–256 KB payloads) and keep **16–32** RPCs in flight to hit a good latency/throughput trade-off on a LAN.

---

## What we measured (quick guidance)

* **Sweet spot (LAN, gRPC):** ~**16–32** concurrent RPCs; **100–256 KB** payloads.
* **Per-round budget:** ~**2–3 ms** for small batches; **~12–20+ ms** around 100 KB.
* **TCP vs gRPC:** TCP is a bit lower latency at the same size, but gRPC’s ecosystem (TLS, Protobuf, tooling) and **fat-batch friendliness** made it the better fit for MPC rounds.
* **Protobuf cost:** sub-microsecond per op; negligible vs millisecond network rounds.

---

## Repo layout

```
fesca/                    # workspace
├─ data_owner/            # DO crate
├─ data_analyst/          # DA crate
├─ computing_node/        # CN crate (gRPC services, storage, optional TCP echo)
└─ src/                 
   └─ main.rs
```

---

## Roadmap

* Secure XOR/AND protocol path integrated end-to-end (using CR AND-triples)
* More SQL operators (joins, filters, group-by) via circuit compiler
* Cost-based optimizer using measured per-round costs
* Config auto-generation for table metadata (DO)
* Optional “volcano” batching executor

---