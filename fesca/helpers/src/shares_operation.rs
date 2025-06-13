use anyhow::{Error, anyhow};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretShareType {
    Boolean,
    SQL,
}

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
    
    let not_share: Vec<u8> = share
        .share
        .iter()
        .map(|byte| !byte)
        .collect();
    
    Ok(SecretShare {
        id: share.id,
        share: not_share,
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
