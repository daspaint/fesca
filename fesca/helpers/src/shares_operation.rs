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
}

/* Boolean Helper Operations */
pub fn xor_shares(share1: &SecretShare, share2: &SecretShare) -> Result<SecretShare, Error> {
    if share1.share_type != share2.share_type {
        return Err(anyhow!("Shares must be of the same type"));
    }
    if share1.share.len() != share2.share.len() {
        return Err(anyhow!("Shares must be of the same length"));
    }
    if share1.id != share2.id {
        return Err(anyhow!("Shares must be of the same ID"));
    }
    let xor_share: Vec<u8> = share1
        .share
        .iter()
        .zip(&share2.share)
        .map(|(a, b)| a ^ b)
        .collect();
    Ok(SecretShare {
        id: share1.id.clone(),
        share: xor_share,
        share_type: share1.share_type.clone(),
    })
}
