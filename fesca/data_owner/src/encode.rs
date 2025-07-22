// Data Encoding Module
// ====================
// This module handles encoding of various data types into bit vectors for secure sharing.
// The encoding process converts human-readable data into binary representations that
// can be processed by the secure multi-party computation system.
//
// Supported data types:
// - Boolean: Single bit representation
// - Unsigned integers (u32): 32-bit little-endian encoding
// - Floating point (f64): IEEE 754 double precision encoding
// - Strings: Character-by-character encoding with configurable charsets
//
// Memory Optimization:
// - Uses BitVector for efficient bit storage (1 bit per bit instead of 8 bits per bool)

use crate::types::{ColumnDescriptor, ColumnType, Charset, BitVector};

/// Main encoding function that dispatches to type-specific encoders.
/// 
/// This function examines the column type and delegates to the appropriate
/// encoding function. Each encoder converts the string representation of
/// the data into a bit vector for efficient storage.
/// 
/// # Arguments
/// * `value` - String representation of the value to encode
/// * `column` - Column descriptor containing type information
/// 
/// # Returns
/// * `BitVector` - Binary representation of the value
pub fn encode_value(value: &str, column: &ColumnDescriptor) -> BitVector {
    match &column.type_hint {
        ColumnType::Boolean => {
            let mut bv = BitVector::new();
            bv.push(encode_bool(value));
            bv
        },
        ColumnType::UnsignedInt => encode_unsigned(value),
        ColumnType::Float => encode_float(value),
        ColumnType::String { max_chars, charset } => encode_string(value, *max_chars, charset),
    }
}

/// Encodes a boolean value into a single bit.
/// 
/// Accepts various string representations of boolean values:
/// - "true", "1" -> true
/// - "false", "0" -> false
/// 
/// The encoding is case-insensitive and trims whitespace.
/// 
/// # Arguments
/// * `value` - String representation of the boolean
/// 
/// # Returns
/// * `bool` - The boolean value (not wrapped in a vector)
/// 
/// # Panics
/// * If the value cannot be parsed as a boolean
fn encode_bool(value: &str) -> bool {
    match value.trim().to_lowercase().as_str() {
        "true" | "1" => true,
        "false" | "0" => false,
        _ => panic!("Invalid boolean value: {}", value),
    }
}

/// Encodes an unsigned 32-bit integer into a 32-bit little-endian representation.
/// 
/// The encoding uses little-endian bit order, where the least significant bit
/// comes first in the vector. This is consistent with how most modern processors
/// handle integer representations.
/// 
/// # Arguments
/// * `value` - String representation of the unsigned integer
/// 
/// # Returns
/// * `BitVector` - 32-bit vector with bits in little-endian order
/// 
/// # Example
/// ```
/// let bits = encode_unsigned("5");
/// // bits[0] = true  (bit 0: 1)
/// // bits[1] = false (bit 1: 0)
/// // bits[2] = true  (bit 2: 1)
/// // bits[3..31] = false (remaining bits: 0)
/// // This represents: 00000000000000000000000000000101 (binary for 5)
/// ```
/// 
/// # Panics
/// * If the value cannot be parsed as a u32
fn encode_unsigned(value: &str) -> BitVector {
    let n: u32 = value.parse().expect("Invalid u32 value");
    let mut bv = BitVector::new();
    // Extract bits in little-endian order (LSB first)
    for i in 0..32 {
        bv.push((n >> i) & 1 == 1);
    }
    bv
}

/// Encodes a 64-bit floating point number using IEEE 754 double precision format.
/// 
/// The encoding uses the standard IEEE 754 representation, which consists of:
/// - 1 sign bit
/// - 11 exponent bits
/// - 52 mantissa bits
/// 
/// The bits are extracted in little-endian byte order for consistency with
/// the integer encoding.
/// 
/// # Arguments
/// * `value` - String representation of the floating point number
/// 
/// # Returns
/// * `BitVector` - 64-bit vector representing the IEEE 754 encoding
/// 
/// # Example
/// ```
/// let bits = encode_float("3.14");
/// // bits will contain 64 bits representing 3.14 in IEEE 754 format
/// ```
/// 
/// # Panics
/// * If the value cannot be parsed as an f64
fn encode_float(value: &str) -> BitVector {
    let f: f64 = value.parse().expect("Invalid f64 value");
    let bits_u64 = f.to_bits();
    let mut bv = BitVector::new();
    // Convert to little-endian byte order, then extract bits
    for i in 0..64 {
        bv.push((bits_u64 >> i) & 1 == 1);
    }
    bv
}

/// Encodes a string using the specified character set and maximum length.
/// 
/// This function converts each character in the string to its binary representation
/// according to the specified charset. The encoding is fixed-length, padding with
/// null characters if the string is shorter than max_chars, or truncating if longer.
/// 
/// # Arguments
/// * `value` - The string to encode
/// * `max_chars` - Maximum number of characters to encode
/// * `charset` - Character encoding scheme to use
/// 
/// # Returns
/// * `BitVector` - Binary representation of the string
/// 
/// # Character Set Support
/// - **ASCII**: 7 bits per character, supports standard ASCII characters (0-127)
/// - **UTF-8**: 8 bits per character, truncates Unicode to first 256 values
/// 
/// # Example
/// ```
/// let bits = encode_string("hello", 10, &Charset::Ascii);
/// // Encodes "hello" + 5 null characters using 7 bits per character
/// // Total: 10 characters × 7 bits = 70 bits
/// ```
fn encode_string(value: &str, max_chars: usize, charset: &Charset) -> BitVector {
    let mut bv = BitVector::new();
    let chars: Vec<char> = value.chars().collect();
    
    // Process each character position up to max_chars
    for i in 0..max_chars {
        // Get character at position i, or null character if past end of string
        let c = chars.get(i).copied().unwrap_or('\0');
        
        // Encode the character according to the charset
        match charset {
            Charset::Ascii => {
                // ASCII: 7 bits per character, mask to ensure valid ASCII range
                let b = c as u32 & 0x7F;
                for j in 0..7 {
                    bv.push((b >> j) & 1 == 1);
                }
            }
            Charset::Utf8 => {
                // UTF-8 simplified: 8 bits per character, truncate to first 256 values
                let b = c as u32 & 0xFF;
                for j in 0..8 {
                    bv.push((b >> j) & 1 == 1);
                }
            }
        };
    }
    
    bv
} 