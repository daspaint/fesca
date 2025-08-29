// computing_node/src/calculate/utils.rs
//! Small helper utilities used by the find_table service and compute routines.
//! - read_binary_party_data_bytes: read file bytes
//! - deserialize_binary_party_data: try bincode then JSON into provided T
//! - extract_bits_as_u64: read up to 64 bits LSB-first from a byte slice

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Read whole file into Vec<u8>
pub fn read_binary_party_data_bytes(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path)?;
    Ok(bytes)
}

/// Try to deserialize bytes as bincode<T>, falling back to serde_json if bincode fails.
/// T must implement serde::DeserializeOwned.
pub fn deserialize_binary_party_data<T>(bytes: &[u8]) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    // try bincode first
    if let Ok(v) = bincode::deserialize::<T>(bytes) {
        return Ok(v);
    }
    // fallback to JSON
    let v = serde_json::from_slice::<T>(bytes)?;
    Ok(v)
}

/// Extract up to 64 bits from `bytes`, treating bit indexing LSB-first (bit 0 = LSB of byte 0).
/// offset_bits: starting bit index (0 = LSB of byte 0)
/// len_bits: number of bits to read (<= 64)
pub fn extract_bits_as_u64(bytes: &[u8], offset_bits: usize, len_bits: usize) -> Result<u64> {
    if len_bits == 0 {
        return Ok(0);
    }
    if len_bits > 64 {
        anyhow::bail!("extract_bits_as_u64 supports up to 64 bits; requested {}", len_bits);
    }

    let mut acc: u64 = 0;
    for i in 0..len_bits {
        let bit_index = offset_bits + i;
        let byte_idx = bit_index / 8;
        let bit_in_byte = bit_index % 8;
        let b = *bytes.get(byte_idx).unwrap_or(&0);
        let bit = ((b >> bit_in_byte) & 1) as u64;
        acc |= bit << i; // i is LSB position in the resulting integer
    }
    Ok(acc)
}
