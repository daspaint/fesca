use super::hashing::hash_value;
use rand::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct SecretShare {
    pub id: u64,
    pub share: u64,
    pub mask: u64,
}
impl Default for SecretShare {
    fn default() -> Self {
        SecretShare {
            id: 0,
            share: 0,
            mask: 0,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SecretShareSend {
    pub id: u64,
    pub share: u64, // can be masked or not
}

impl Default for SecretShareSend {
    fn default() -> Self {
        SecretShareSend { id: 0, share: 0 }
    }
}
pub fn generate_mask() -> Vec<u64> {
    let mut rng = rand::rng();
    let value1: u64 = rng.random::<u64>();
    let value2: u64 = rng.random::<u64>();
    let result: u64 = value1 ^ value2;
    vec![value1, value2, result]
}

pub fn generate_secret_share(value: u64) -> Vec<SecretShare> {
    let mut rng = rand::rng();

    let id = hash_value(value);
    let mask = generate_mask();

    let share1: u64 = rng.random::<u64>();
    let share2: u64 = rng.random::<u64>();
    let share3: u64 = share1 ^ share2 ^ value;

    vec![
        SecretShare {
            id,
            share: share1,
            mask: mask[0],
        },
        SecretShare {
            id,
            share: share2,
            mask: mask[1],
        },
        SecretShare {
            id,
            share: share3,
            mask: mask[2],
        },
    ]
}

pub fn reconstruct_secret(shares: &[SecretShareSend]) -> Option<u64> {
    if shares.len() != 3 {
        return None;
    }

    let mut result: u64 = 0;
    for share in shares {
        result ^= share.share;
    }

    Some(result)
}
