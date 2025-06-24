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