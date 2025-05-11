use crate::types::{SecretShare_single_bit};


// mock xor gate, local operation
pub fn xor_gate_single_bit(share1:SecretShare_single_bit,share2:SecretShare_single_bit) -> SecretShare_single_bit {
    let r = SecretShare_single_bit{
        x: share1.x ^ share2.x,
        a: share1.a ^ share2.a,
    };
    r
}


pub fn and_gate_single_bit(share1:SecretShare_single_bit,share2:SecretShare_single_bit , coorelated_r:bool) -> bool {

    // step1 – compute 3/3 shares
    let r = (share1.x & share2.x) ^ (share1.a  & share2.a) ^ coorelated_r;
    r
}