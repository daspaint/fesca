use super::secret_share::{SecretShare, SecretShareSend};

// Boolean operations for SecretShare

//XOR operation
pub fn xor_operation(a: &SecretShare, b: &SecretShare) -> SecretShare {
    SecretShare {
        id: a.id ^ b.id,
        share: a.share ^ b.share,
        mask: a.mask ^ b.mask,
    }
}

//AND operation after sharing
pub fn and_operation(
    a1: &SecretShare,
    b1: &SecretShare,
    a2: &SecretShareSend, //unmasked
    b2: &SecretShareSend, //unmasked
    mask: u64,
) -> SecretShare {
    let id = a1.id ^ b1.id;

    let share = (a1.share & b1.share) ^ (a1.share & b2.share) ^ (a2.share & b1.share);

    SecretShare {
        id,
        share: share,
        mask,
    }
}
