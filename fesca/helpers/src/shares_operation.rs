use anyhow::{Error, anyhow};
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretShareType {
    Boolean,
    SQL,
}

#[derive(Debug, Clone)]
pub struct SecretShare {
    pub id: u64,
    pub share: Vec<u8>,
    pub share_type: SecretShareType,
    pub party_id: u8, 
}

/* Helper */
pub fn validate_shares_compatible(share1: &SecretShare, share2: &SecretShare)->Result<(), anyhow::Error>{
    if share1.share_type != share2.share_type {
        return Err(anyhow!("Shares must be of the same type"));
    }
    if share1.share.len() != share2.share.len() {
        return Err(anyhow!("Shares must be of the same length"));
    }
    if share1.id != share2.id {
        return Err(anyhow!("Shares must be of the same ID"));
    }
    return Ok(())
}

/* Boolean Helper Operations */
pub fn xor_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    validate_shares_compatible(share1, share2)?;
    
    let xor_share: Vec<u8> = share1
        .share
        .iter()
        .zip(&share2.share)
        .map(|(a, b)| a ^ b)
        .collect();
    
    Ok(SecretShare {
        id: share1.id,
        share: xor_share,
        share_type: share1.share_type.clone(),
        party_id: share1.party_id,
    })
}

pub fn and_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    validate_shares_compatible(share1, share2)?;
    
    Err(anyhow!("AND operation is not supported for Boolean shares"))
}

pub fn or_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    let not_a = not_share(share1)?;
    let not_b = not_share(share2)?;
    let not_a_and_not_b = and_shares(&not_a, &not_b)?;
    not_share(&not_a_and_not_b)
}

pub fn not_share(share: &SecretShare) -> Result<SecretShare, Error> {
    if share.share_type != SecretShareType::Boolean {
        return Err(anyhow!("NOT operation only supported for Boolean shares"));
    }
    
    let result_share = if share.party_id == 1 {
        share.share.iter().map(|byte| !byte).collect()
    } else {
        share.share.clone()
    };
    
    Ok(SecretShare {
        id: share.id,
        share: result_share,
        share_type: SecretShareType::Boolean,
        party_id: share.party_id,
    })
}


pub fn eq_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    validate_shares_compatible(share1, share2)?;
    
    let xor_result = xor_shares(share1, share2)?;
    let not_xor = not_share(&xor_result)?;
    
    and_all_bits(&not_xor)
}

fn and_all_bits(share: &SecretShare) -> Result<SecretShare, Error> {
    if share.share.is_empty() {
        return Err(anyhow!("Cannot AND empty share"));
    }
    
    let mut result = 0xFF_u8;
    
    for byte in &share.share {
        for bit_pos in 0..8 {
            let bit = (byte >> bit_pos) & 1;
            result &= if bit == 1 { 0xFF } else { 0x00 };
        }
    }
    
    Ok(SecretShare {
        id: share.id,
        share: vec![result],
        share_type: SecretShareType::Boolean,
        party_id: share.party_id,
    })
}


// Add this to your existing code
impl SecretShare {
    /// Create a new secret share
    pub fn new(id: u64, share: Vec<u8>, share_type: SecretShareType, party_id: u8) -> Self {
        SecretShare {
            id,
            share,
            share_type,
            party_id,
        }
    }
}

/// Convert a value to its binary representation as bytes
pub fn value_to_bytes(value: u64, byte_length: usize) -> Vec<u8> {
    let mut bytes = vec![0u8; byte_length];
    for i in 0..byte_length {
        bytes[i] = ((value >> (i * 8)) & 0xFF) as u8;
    }
    bytes
}

/// Convert bytes back to a value (for testing/verification)
pub fn bytes_to_value(bytes: &[u8]) -> u64 {
    let mut value = 0u64;
    for (i, &byte) in bytes.iter().enumerate() {
        value |= (byte as u64) << (i * 8);
    }
    value
}

/// Create 3-party boolean secret shares 
pub fn create_boolean_shares(value: u64, id: u64, byte_length: usize) -> Result<(SecretShare, SecretShare, SecretShare), Error> {
    if byte_length == 0 {
        return Err(anyhow!("Byte length must be greater than 0"));
    }
    
    let value_bytes = value_to_bytes(value, byte_length);
    let mut rng = rand::rng();
    
    // Generate two random shares
    let share1_bytes: Vec<u8> = (0..byte_length).map(|_| rng.random()).collect();
    let share2_bytes: Vec<u8> = (0..byte_length).map(|_| rng.random()).collect();
    
    // Third share is computed to satisfy: share1 ⊕ share2 ⊕ share3 = value
    let share3_bytes: Vec<u8> = value_bytes
        .iter()
        .zip(&share1_bytes)
        .zip(&share2_bytes)
        .map(|((&v, &s1), &s2)| v ^ s1 ^ s2)
        .collect();
    
    let share1 = SecretShare::new(id, share1_bytes, SecretShareType::Boolean, 1);
    let share2 = SecretShare::new(id, share2_bytes, SecretShareType::Boolean, 2);
    let share3 = SecretShare::new(id, share3_bytes, SecretShareType::Boolean, 3);
    
    Ok((share1, share2, share3))
}

pub fn reconstruct_boolean_value(share1: &SecretShare, share2: &SecretShare, share3: &SecretShare) -> Result<u64, Error> {
    // Validate all shares are compatible
    validate_shares_compatible(share1, share2)?;
    validate_shares_compatible(share2, share3)?;
    
    if share1.share_type != SecretShareType::Boolean {
        return Err(anyhow!("All shares must be Boolean type"));
    }
    
    // XOR all three shares to get original value
    let reconstructed_bytes: Vec<u8> = share1.share
        .iter()
        .zip(&share2.share)
        .zip(&share3.share)
        .map(|((&s1, &s2), &s3)| s1 ^ s2 ^ s3)
        .collect();
    
    Ok(bytes_to_value(&reconstructed_bytes))
}
