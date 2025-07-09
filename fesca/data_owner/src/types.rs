// Type Definitions for Secure Multi-Party Computation
// ===================================================
// This module defines all the data structures used in the secure multi-party computation
// system for the data owner component. The types support a 3-party replicated secret
// sharing scheme where data is split into shares distributed among three computing nodes.
//
// Key concepts:
// - BitShare: Individual bit shares for 3-party replication
// - SharedBitString: Collections of bit shares representing encoded values
// - SharedRow/SharedPartyData: Organizational structures for tabular data
// - Schema types: Metadata describing data types and structure

use serde::{Deserialize, Serialize};

/// Represents a single bit share in our 3-party replicated secret sharing scheme.
/// 
/// In replicated secret sharing, each bit is split into three shares where each party
/// holds two shares. The original bit can be reconstructed by XORing all three shares:
/// `original_bit = share_a ⊕ share_b ⊕ share_c`
/// 
/// Each BitShare contains two of the three shares, and the three parties hold
/// overlapping pairs.
/// 
/// # Fields
/// * `share_a` - First share of the bit
/// * `share_b` - Second share of the bit
/// 
/// # Example
/// ```
/// // To share a bit `true`:
/// let a = random_bit();
/// let b = random_bit();
/// let c = true ^ a ^ b;  // Ensure XOR reconstruction works
/// 
/// // Party 0 gets: BitShare { share_a: a, share_b: b }
/// // Party 1 gets: BitShare { share_a: b, share_b: c }
/// // Party 2 gets: BitShare { share_a: c, share_b: a }
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitShare {
    pub share_a: bool,
    pub share_b: bool,
}

/// Represents a string of bits where each bit is secret shared across three computing nodes.
/// 
/// This structure contains a vector of BitShares, where each BitShare represents
/// one bit of the original data. This is used to represent encoded values like
/// integers, floats, or strings after they've been converted to binary and shared.
/// 
/// # Fields
/// * `bits` - Vector of bit shares, ordered from least significant to most significant
/// 
/// # Example
/// ```
/// // For encoding the number 5 (binary: 101):
/// // bits[0] = BitShare for bit 1 (LSB)
/// // bits[1] = BitShare for bit 0
/// // bits[2] = BitShare for bit 1 (MSB)
/// // bits[3..31] = BitShares for remaining bits (all 0)
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SharedBitString {
    pub bits: Vec<BitShare>,
}

/// Represents a row of data where each entry is a shared bit string.
/// 
/// In our tabular data structure, each row contains multiple fields (columns),
/// and each field is represented as a SharedBitString. This structure holds
/// the shared representation of all fields in a single row.
/// 
/// # Fields
/// * `entries` - Vector of SharedBitString, one for each column in the row
/// 
/// # Example
/// ```
/// // For a row with columns [name: "Alice", age: 25, active: true]:
/// // entries[0] = SharedBitString for "Alice" (string encoding)
/// // entries[1] = SharedBitString for 25 (u32 encoding)
/// // entries[2] = SharedBitString for true (boolean encoding)
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SharedRow {
    pub entries: Vec<SharedBitString>,
}

/// Represents the data held by a single computing node in our 3-party system.
/// 
/// Each computing node in the system holds one share of the entire table.
/// This structure contains all the rows that a particular party is responsible for,
/// along with identifying information about the party and table.
/// 
/// # Fields
/// * `party_id` - Unique identifier for this computing party (0, 1, or 2)
/// * `table_id` - Unique identifier for the table this data belongs to
/// * `rows` - All rows of shared data for this party
/// 
/// # Security Note
/// Each party holds shares of the data, not the original values. The original
/// data can only be reconstructed when 2 parties collaborate.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SharedPartyData {
    pub party_id: u32,
    pub table_id: u32,
    pub rows: Vec<SharedRow>,
}

/// Defines character encoding schemes for string data.
/// 
/// Different character sets require different numbers of bits per character.
/// This enum allows the system to handle various encoding requirements while
/// maintaining consistent bit-level operations.
/// 
/// # Variants
/// * `Ascii` - Standard ASCII encoding (7 bits per character, 0-127)
/// * `Utf8` - Simplified UTF-8 encoding (8 bits per character, 0-255)
/// * `Custom` - User-defined encoding with specified bits per character
/// 
/// # Example
/// ```
/// let ascii_charset = Charset::Ascii;        // 7 bits per char
/// let utf8_charset = Charset::Utf8;          // 8 bits per char  
/// let custom_charset = Charset::Custom { bits_per_char: 6 }; // 6 bits per char
/// ```
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Charset {
    Ascii,       // 7 bits per char
    Utf8,        // 8 bits per char (simplified, truncates Unicode)
    Custom { bits_per_char: usize },
}

/// Defines the possible data types for columns in the shared table.
/// 
/// This enum specifies how data should be interpreted and encoded into bits.
/// Each type has specific encoding rules that determine the bit representation
/// used for secure sharing.
/// 
/// # Variants
/// * `Boolean` - Single bit values (true/false)
/// * `UnsignedInt` - 32-bit unsigned integers (0 to 4,294,967,295)
/// * `Float` - IEEE 754 floating point numbers with specified bit width
/// * `String` - Text data with maximum character count and encoding scheme
/// 
/// # Example
/// ```
/// let bool_type = ColumnType::Boolean;
/// let int_type = ColumnType::UnsignedInt;
/// let float_type = ColumnType::Float { bit_width: 64 };
/// let string_type = ColumnType::String { 
///     max_chars: 50, 
///     charset: Charset::Utf8 
/// };
/// ```
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ColumnType {
    Boolean,
    UnsignedInt,       // Only u32 is supported (32 bits)
    Float { bit_width: usize },             // f32 (32 bits) or f64 (64 bits)
    String { max_chars: usize, charset: Charset }, // Fixed-length string encoding
}


/// 
/// This structure contains all the information needed to properly encode
/// and interpret data in a specific column, including the column name
/// and its data type specification.
/// 
/// # Fields
/// * `name` - Human-readable name of the column
/// * `type_hint` - Data type specification for encoding/decoding
/// 
/// # Example
/// ```
/// let age_column = ColumnDescriptor {
///     name: "age".to_string(),
///     type_hint: ColumnType::UnsignedInt,
/// };
/// 
/// let name_column = ColumnDescriptor {
///     name: "full_name".to_string(),
///     type_hint: ColumnType::String { 
///         max_chars: 100, 
///         charset: Charset::Utf8 
///     },
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct ColumnDescriptor {
    pub name: String,
    pub type_hint: ColumnType,
}


/// 
/// This structure serves as the schema definition for a table, containing
/// all the metadata needed to interpret the shared data correctly. It includes
/// table identification, column definitions, and row count information.
/// 
/// # Fields
/// * `table_name` - Human-readable name of the table
/// * `table_id` - Unique numerical identifier for the table
/// * `columns` - Vector of column descriptors defining the table structure
/// * `row_count` - Total number of rows in the table
/// 
/// # Example
/// ```
/// let schema = TableSchema {
///     table_name: "users".to_string(),
///     table_id: 1,
///     columns: vec![
///         ColumnDescriptor {
///             name: "name".to_string(),
///             type_hint: ColumnType::String { max_chars: 50, charset: Charset::Utf8 },
///         },
///         ColumnDescriptor {
///             name: "age".to_string(),
///             type_hint: ColumnType::UnsignedInt,
///         },
///     ],
///     row_count: 1000,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TableSchema {
    pub table_name: String,
    pub table_id: u32,
    pub columns: Vec<ColumnDescriptor>,
    pub row_count: usize,
}

/// 
/// This is the final output structure that contains all the shared data organized
/// by party, along with the schema information needed to interpret the data.
/// Each party receives their portion of the shares, and the original data can
/// only be reconstructed when all three parties collaborate.
/// 
/// # Fields
/// * `party0_data` - Shared data for computing party 0
/// * `party1_data` - Shared data for computing party 1  
/// * `party2_data` - Shared data for computing party 2
/// * `schema` - Table schema describing the data structure
/// 
/// # Usage
/// This structure would typically be serialized and distributed to the three
/// computing nodes in a real deployment. Each node would receive only their
/// corresponding party data plus the schema.
/// 
/// # Example
/// ```
/// let shared_table = SharedTableOutput {
///     party0_data: SharedPartyData { party_id: 0, table_id: 1, rows: vec![...] },
///     party1_data: SharedPartyData { party_id: 1, table_id: 1, rows: vec![...] },
///     party2_data: SharedPartyData { party_id: 2, table_id: 1, rows: vec![...] },
///     schema: table_schema,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct SharedTableOutput {
    pub party0_data: SharedPartyData,
    pub party1_data: SharedPartyData,
    pub party2_data: SharedPartyData,
    pub schema: TableSchema,
} 