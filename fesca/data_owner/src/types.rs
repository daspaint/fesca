// 3-party replicated secret sharing data structures.
// Uses bitvec for efficient bit storage (2 bits per original bit, ~87.5% memory reduction).

use serde::{Deserialize, Serialize};
use bitvec::prelude::*;

/// Character encoding schemes for string data.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Charset {
    Ascii,       // 7 bits per char
    Utf8,        // 8 bits per char (simplified, truncates Unicode)
}

/// Data types for table columns.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ColumnType {
    Boolean,
    UnsignedInt,       // Only u32 is supported (32 bits)
    Float,             // f64 (64 bits)
    String { max_chars: usize, charset: Charset }, // Fixed-length string encoding
}

/// Column metadata for table schema.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ColumnDescriptor {
    pub name: String,
    pub type_hint: ColumnType,
}

/// Table schema with column definitions and metadata.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TableSchema {
    pub table_name: String,
    pub table_id: u32,
    pub columns: Vec<ColumnDescriptor>,
    pub row_count: usize,
}

/// Efficient bit vector using u8 storage with LSB-first ordering.
pub type BitVector = BitVec<u8, bitvec::order::Lsb0>;

/// Bit vector pair for 3-party replicated secret sharing.
/// Each party gets 2 of 3 shares. Reconstruction: original = share_a ⊕ share_b ⊕ share_c
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SharedBitVector {
    pub share_a: BitVector,
    pub share_b: BitVector,
}



/// Binary row data with concatenated column bitstrings and metadata.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BinaryRow {
    pub bitstring_a: Vec<u8>,  // First bitstring as bytes
    pub bitstring_b: Vec<u8>,  // Second bitstring as bytes
    pub column_bit_offsets: Vec<u32>,  // Starting bit position for each column
    pub column_bit_lengths: Vec<u32>,  // Bit length for each column
}

/// Binary representation of party data for efficient transmission.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BinaryPartyData {
    pub party_id: u32,
    pub table_id: u32,
    pub rows: Vec<BinaryRow>,
}