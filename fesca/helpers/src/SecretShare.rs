use rand::Rng;


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecretShareType {
    Boolean,
    SQL,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretShare {
    pub id: u64,
    pub share: Vec<u8>,
    pub share_type: SecretShareType,
}


pub fn check(share1: &SecretShare, share2: &SecretShare) -> bool{
    if share1.share_type != share2.share_type {
        return false;
    }
    if share1.share.len() != share2.share.len() {
        return false;    
    }
    if share1.id != share2.id {
       return false;    
    }
    return true;
}

pub fn generate_shares_vec(secret_data: Vec<u8>, share_id: u64) -> (SecretShare, SecretShare, SecretShare) {
    let mut rng = rand::rng();
    
    // Generate random shares for each byte in the vector
    let mut share1_data = Vec::with_capacity(secret_data.len());
    let mut share2_data = Vec::with_capacity(secret_data.len());
    let mut share3_data = Vec::with_capacity(secret_data.len());
    
    for &_byte in &secret_data {
        // Generate two random bytes
        let x1 = rng.random::<u8>();
        let x2 = rng.random::<u8>();
        // Third share is XOR of first two to maintain the secret
        let x3 = x1 ^ x2;
        
        share1_data.push(x1);
        share2_data.push(x2);
        share3_data.push(x3);
    }
    
    // Create additional random shares for the 'a' component (similar to single bit example)
    let mut a1_data = Vec::with_capacity(secret_data.len());
    let mut a2_data = Vec::with_capacity(secret_data.len());
    let mut a3_data = Vec::with_capacity(secret_data.len());
    
    for &byte in &secret_data {
        let x1 = share1_data[a1_data.len()];
        let x2 = share2_data[a2_data.len()];
        let x3 = share3_data[a3_data.len()];
        
        a1_data.push(x3 ^ byte);
        a2_data.push(x1 ^ byte);
        a3_data.push(x2 ^ byte);
    }
    
    // Combine x and a shares for each party
    let mut p1_share = share1_data;
    p1_share.extend(a1_data);
    
    let mut p2_share = share2_data;
    p2_share.extend(a2_data);
    
    let mut p3_share = share3_data;
    p3_share.extend(a3_data);
    
    let p1 = SecretShare {
        id: share_id,
        share: p1_share,
        share_type: SecretShareType::Boolean,
    };
    
    let p2 = SecretShare {
        id: share_id,
        share: p2_share,
        share_type: SecretShareType::Boolean,
    };
    
    let p3 = SecretShare {
        id: share_id,
        share: p3_share,
        share_type: SecretShareType::Boolean,
    };
    
    println!("share1: {:?}", p1);
    println!("share2: {:?}", p2);
    println!("share3: {:?}", p3);
    
    (p1, p2, p3)
}

pub fn reconstruct_boolean_shares(share1: &SecretShare, share2: &SecretShare, share3: &SecretShare) -> Result<Vec<u8>, String> {
    if !check(share1, share2) || !check(share2, share3) {
        return Err("Shares are incompatible".to_string());
    }
    
    if share1.share_type != SecretShareType::Boolean {
        return Err("Expected boolean shares".to_string());
    }
    
    let data_len = share1.share.len() / 2;
    let mut reconstructed = Vec::with_capacity(data_len);
    
    for i in 0..data_len {
        let x1 = share1.share[i];
        let x2 = share2.share[i];
        let x3 = share3.share[i];
        
        let secret_byte = x1 ^ x2 ^ x3;
        reconstructed.push(secret_byte);
    }
    
    Ok(reconstructed)
}
