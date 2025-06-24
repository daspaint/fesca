use anyhow::{Error, anyhow};
use super::SecretShare::{SecretShare,SecretShareType,check};

/* Boolean Helper Operations */
pub fn xor_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    if check(share1, share2) {
        return Err(anyhow!("Error shares are not consistent"));
    } 
    if share1.share_type!= SecretShareType::Boolean {
        return Err(anyhow!("Shares are not Boolean"));
    }
}


/* Arithmetic Helper Operations */
pub fn add_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    if !check(share1, share2) {
        return Err(anyhow!("Error: shares are not consistent"));
    }
    
    match share1.share_type {
        SecretShareType::Arithmetic => {
            let mut result_data = Vec::with_capacity(share1.share.len());
            
            for (a, b) in share1.share.iter().zip(share2.share.iter()) {
                result_data.push(a.wrapping_add(*b));
            }
            
            Ok(SecretShare {
                id: share1.id,
                share: result_data,
                share_type: SecretShareType::Arithmetic,
            })
        },
        SecretShareType::Boolean => {
            // For boolean shares, addition is XOR
            xor_shares(share1, share2)
        },
        SecretShareType::SQL => {
            // Treat SQL type as arithmetic
            // let mut result_data = Vec::with_capacity(share1.share.len());
            
            // for (a, b) in share1.share.iter().zip(share2.share.iter()) {
            //     result_data.push(a.wrapping_add(*b));
            // }
            
            // Ok(SecretShare {
            //     id: share1.id,
            //     share: result_data,
            //     share_type: SecretShareType::SQL,
            // })
        }
    }
}

pub fn subtract_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    if !check(share1, share2) {
        return Err(anyhow!("Error: shares are not consistent"));
    }
    
    match share1.share_type {
        SecretShareType::Arithmetic | SecretShareType::SQL => {
            let mut result_data = Vec::with_capacity(share1.share.len());
            
            for (a, b) in share1.share.iter().zip(share2.share.iter()) {
                result_data.push(a.wrapping_sub(*b));
            }
            
            Ok(SecretShare {
                id: share1.id,
                share: result_data,
                share_type: share1.share_type.clone(),
            })
        },
        SecretShareType::Boolean => {
            // For boolean shares, subtraction is also XOR
            xor_shares(share1, share2)
        }
    }
}

