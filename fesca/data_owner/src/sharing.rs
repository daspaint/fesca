use rand::Rng;
use crate::types::{BitShare, SharedBitString, Charset};

/// Trait for values that can be secret shared using replicated secret sharing
pub trait ReplicatedShareable {
    type Share;

    /// Generates three shares of the value using replicated secret sharing
    /// Returns a tuple of (share0, share1, share2) where each party gets two shares
    fn replicate(&self, rng: &mut impl Rng) -> (Self::Share, Self::Share, Self::Share);
}

/// Helper function to share a bit string using XOR-based replicated sharing
pub fn share_bit_string(bits: &[bool], rng: &mut impl Rng) -> (SharedBitString, SharedBitString, SharedBitString) {
    let mut shares = (Vec::new(), Vec::new(), Vec::new());
    
    for &bit in bits {
        let (s1, s2, s3) = bool::replicate(&bit, rng);
        shares.0.push(s1);
        shares.1.push(s2);
        shares.2.push(s3);
    }
    
    (
        SharedBitString { bits: shares.0 },
        SharedBitString { bits: shares.1 },
        SharedBitString { bits: shares.2 }
    )
}

/// Helper function to share a string using the provided charset and max_chars for encoding
pub fn share_string_with_encoding(s: &str, charset: &Charset, max_chars: usize, rng: &mut impl rand::Rng) -> (SharedBitString, SharedBitString, SharedBitString) {
    let mut bits = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    for i in 0..max_chars {
        let c = chars.get(i).copied().unwrap_or('\0');
        let char_bits: Vec<bool> = match charset {
            Charset::Ascii => {
                let b = c as u32 & 0x7F;
                (0..7).map(|i| (b >> i) & 1 == 1).collect()
            }
            Charset::Utf8 => {
                // Use 8 bits per char (truncate if >255)
                let b = c as u32 & 0xFF;
                (0..8).map(|i| (b >> i) & 1 == 1).collect()
            }
            Charset::Custom { bits_per_char } => {
                let b = c as u32;
                (0..*bits_per_char).map(|i| (b >> i) & 1 == 1).collect()
            }
        };
        bits.extend(char_bits);
    }
    share_bit_string(&bits, rng)
}

// Implementation for boolean values
impl ReplicatedShareable for bool {
    type Share = BitShare;

    fn replicate(&self, rng: &mut impl Rng) -> (BitShare, BitShare, BitShare) {
        let a = rng.random();
        let b = rng.random();
        let c = *self ^ a ^ b;
        (
            BitShare { share_a: a, share_b: b },
            BitShare { share_a: b, share_b: c },
            BitShare { share_a: c, share_b: a },
        )
    }
}

// Implementation for unsigned integers
impl ReplicatedShareable for u32 {
    type Share = SharedBitString;

    fn replicate(&self, rng: &mut impl Rng) -> (SharedBitString, SharedBitString, SharedBitString) {
        // Convert u32 to bit string (32 bits)
        let bits: Vec<bool> = (0..32)
            .map(|i| (self >> i) & 1 == 1)
            .collect();
        
        share_bit_string(&bits, rng)
    }
}

// Implementation for floating-point numbers
impl ReplicatedShareable for f64 {
    type Share = SharedBitString;

    fn replicate(&self, rng: &mut impl Rng) -> (SharedBitString, SharedBitString, SharedBitString) {
        // Convert f64 to bit string (64 bits)
        let bits: Vec<bool> = self.to_bits()
            .to_le_bytes()
            .iter()
            .flat_map(|&byte| (0..8).map(move |i| (byte >> i) & 1 == 1))
            .collect();
        
        share_bit_string(&bits, rng)
    }
}

impl ReplicatedShareable for String {
    type Share = SharedBitString;

    fn replicate(&self, _rng: &mut impl rand::Rng) -> (SharedBitString, SharedBitString, SharedBitString) {
        panic!("Use explicit encoding and sharing for String; replicate_with_encoding is not part of the trait");
    }
} 