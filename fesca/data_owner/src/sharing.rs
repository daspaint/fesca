use rand::Rng;
use crate::types::BitVector;

/// Share a BitVector using 3-party replicated secret sharing and convert to bytes.
/// Returns three tuples, each containing (share_a_bytes, share_b_bytes) for each party.
pub fn share_bit_vector(bits: &BitVector, rng: &mut impl Rng) -> ((Vec<u8>, Vec<u8>), (Vec<u8>, Vec<u8>), (Vec<u8>, Vec<u8>)) {
    let mut a_bits = BitVector::new();
    let mut b_bits = BitVector::new();
    let mut c_bits = BitVector::new();
    
    // Generate random shares for each bit
    for bit in bits.iter() {
        let a = rng.gen_bool(0.5);
        let b = rng.gen_bool(0.5);
        let c = *bit ^ a ^ b;  // Ensure XOR reconstruction works
        
        a_bits.push(a);
        b_bits.push(b);
        c_bits.push(c);
    }
    
    // Convert bit vectors to bytes directly
    let mut a_bytes = Vec::new();
    for chunk in a_bits.chunks(8) {
        let mut byte = 0u8;
        for (i, bit) in chunk.iter().enumerate() {
            if *bit {
                byte |= 1 << i;
            }
        }
        a_bytes.push(byte);
    }
    
    let mut b_bytes = Vec::new();
    for chunk in b_bits.chunks(8) {
        let mut byte = 0u8;
        for (i, bit) in chunk.iter().enumerate() {
            if *bit {
                byte |= 1 << i;
            }
        }
        b_bytes.push(byte);
    }
    
    let mut c_bytes = Vec::new();
    for chunk in c_bits.chunks(8) {
        let mut byte = 0u8;
        for (i, bit) in chunk.iter().enumerate() {
            if *bit {
                byte |= 1 << i;
            }
        }
        c_bytes.push(byte);
    }
    
    // Return bytes for each party: (share_a, share_b)
    (
        (a_bytes.clone(), b_bytes.clone()),    // Party 0: shares a and b
        (b_bytes.clone(), c_bytes.clone()),    // Party 1: shares b and c  
        (a_bytes, c_bytes),                    // Party 2: shares a and c
    )
}

