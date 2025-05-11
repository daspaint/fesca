
use crate::types::{SecretShare_single_bit};

pub fn recover_shares_single_bit(share1:&SecretShare_single_bit,share2:&SecretShare_single_bit) -> bool {
    (share2.a ^share1.x)
}