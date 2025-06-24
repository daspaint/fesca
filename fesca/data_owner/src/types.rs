/// Represents a single bit share in our 3-party replicated secret sharing scheme.
/// Each bit is split into three shares that are distributed among three computing nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct BitShare {
    pub share_a: bool,
    pub share_b: bool,
}

/// Represents a string of bits where each bit is secret shared across three computing nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedBitString {
    pub bits: Vec<BitShare>,
}

/// Represents a row of data where each entry is a shared bit string.
/// In our 3-party system, this structure holds the shares of all entries in a single row,
#[derive(Debug, Clone, PartialEq)]
pub struct SharedRow {
    pub entries: Vec<SharedBitString>,
}

/// Represents the data held by a single computing node in our 3-party system.
/// Each computing node holds one of the three shares for the entire table, identified by
/// party_id and table_id. The rows field contains all the shared rows that this
/// computing node is responsible for.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedPartyData {
    pub party_id: u32,
    pub table_id: u32,
    pub rows: Vec<SharedRow>,
}

/// Defines the possible data types for columns in the shared table.
/// This is used to ensure proper interpretation of the shared values
/// when they are reconstructed by the three computing nodes.
#[derive(Clone, Debug, PartialEq)]
pub enum Charset {
    Ascii,       // 7 bits per char
    Utf8,        // Variable, but use fixed-length encoding per char
    Custom { bits_per_char: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ColumnType {
    Boolean,
    UnsignedInt { bit_width: usize },       // u8, u16, u32, u64, etc.
    Float { bit_width: usize },             // f32 = 32 bits, f64 = 64 bits
    String { max_chars: usize, charset: Charset }, // Encoded per-char
}

/// Describes the metadata for a single column in the shared table.
/// The bit_width specifies how many bits are used to represent each value
/// in the column.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDescriptor {
    pub name: String,
    pub type_hint: ColumnType,
}

/// Defines the structure and metadata of the entire shared table.
#[derive(Debug, Clone, PartialEq)]
pub struct TableSchema {
    pub table_name: String,
    pub table_id: u32,
    pub columns: Vec<ColumnDescriptor>,
    pub row_count: usize,
}

/// Represents the complete shared table data distributed across three computing nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SharedTableOutput {
    pub party0_data: SharedPartyData,
    pub party1_data: SharedPartyData,
    pub party2_data: SharedPartyData,
    pub schema: TableSchema,
} 