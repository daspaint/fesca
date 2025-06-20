use crate::types::{ColumnDescriptor, ColumnType, Charset};

/// Encodes a value string into a bit vector based on the column descriptor.
pub fn encode_value(value: &str, column: &ColumnDescriptor) -> Vec<bool> {
    match &column.type_hint {
        ColumnType::Boolean => encode_bool(value),
        ColumnType::UnsignedInt => encode_unsigned(value),
        ColumnType::Float { .. } => encode_float(value),
        ColumnType::String { max_chars, charset } => encode_string(value, *max_chars, charset),
    }
}

fn encode_bool(value: &str) -> Vec<bool> {
    let b = match value.trim().to_lowercase().as_str() {
        "true" | "1" => true,
        "false" | "0" => false,
        _ => panic!("Invalid boolean value: {}", value),
    };
    vec![b]
}

fn encode_unsigned(value: &str) -> Vec<bool> {
    let n: u32 = value.parse().expect("Invalid u32 value");
    (0..32).map(|i| (n >> i) & 1 == 1).collect()
}

fn encode_float(value: &str) -> Vec<bool> {
    let f: f64 = value.parse().expect("Invalid f64 value");
    let bits = f.to_bits();
    (0..64).map(|i| (bits >> i) & 1 == 1).collect()
}

fn encode_string(value: &str, max_chars: usize, charset: &Charset) -> Vec<bool> {
    let mut bits = Vec::new();
    let chars: Vec<char> = value.chars().collect();
    for i in 0..max_chars {
        let c = chars.get(i).copied().unwrap_or('\0');
        let char_bits: Vec<bool> = match charset {
            Charset::Ascii => {
                let b = c as u32 & 0x7F;
                (0..7).map(|i| (b >> i) & 1 == 1).collect()
            }
            Charset::Utf8 => {
                let b = c as u32 & 0xFF;
                (0..8).map(|i| (b >> i) & 1 == 1).collect()
            }
            Charset::Custom { bits_per_char } => {
                let b = c as u32;
                (0..*bits_per_char).map(|i| (b >> i) & 1 == 1).collect()
            }
        };
        bits.extend(char_bits);
    }
    bits
} 