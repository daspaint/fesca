use rand::Rng;
use sha2::{Sha256, Digest};
use crate::types::{
    PartyState, CorrelatedRandomnessResult, CorrelatedRandomnessBoolean, 
    CorrelatedRandomnessArithmetic, ComputationalCorrelatedRandomness
};

// ============================================================================
// INFORMATION-THEORETIC CORRELATED RANDOMNESS (Paper Section 2.2)
// ============================================================================

/// Generate correlated randomness for a single bit
/// Returns (alpha, beta, gamma) where alpha ⊕ beta ⊕ gamma = 0
pub fn generate_correlated_single_bit() -> (bool, bool, bool) {
    let mut rng = rand::thread_rng();

    let alpha = rng.random::<bool>();
    let beta = rng.random::<bool>();
    let gamma = alpha ^ beta;

    (alpha, beta, gamma)
}

/// Information-theoretic correlated randomness generation
/// Based on Paper: Each party Pi chooses random pi and sends to Pi+1
/// α = p3 ⊕ p1, β = p1 ⊕ p2, γ = p2 ⊕ p3
pub fn generate_information_theoretic_correlated_randomness() -> CorrelatedRandomnessBoolean {
    let mut rng = rand::thread_rng();
    
    // Each party chooses a random bit
    let p1 = rng.random::<bool>();
    let p2 = rng.random::<bool>();
    let p3 = rng.random::<bool>();
    
    // Compute correlated values according to paper
    let alpha = p3 ^ p1;  // P1 computes α = p3 ⊕ p1
    let beta = p1 ^ p2;   // P2 computes β = p1 ⊕ p2
    let gamma = p2 ^ p3;  // P3 computes γ = p2 ⊕ p3
    
    println!("=== Information-Theoretic Correlated Randomness ===");
    println!("P1 chooses p1 = {}", p1);
    println!("P2 chooses p2 = {}", p2);
    println!("P3 chooses p3 = {}", p3);
    println!("P1 computes α = p3 ⊕ p1 = {} ⊕ {} = {}", p3, p1, alpha);
    println!("P2 computes β = p1 ⊕ p2 = {} ⊕ {} = {}", p1, p2, beta);
    println!("P3 computes γ = p2 ⊕ p3 = {} ⊕ {} = {}", p2, p3, gamma);
    println!("Verification: α ⊕ β ⊕ γ = {} ⊕ {} ⊕ {} = {}", alpha, beta, gamma, alpha ^ beta ^ gamma);
    
    CorrelatedRandomnessBoolean { alpha, beta, gamma }
}

// ============================================================================
// COMPUTATIONAL CORRELATED RANDOMNESS (Paper Section 2.2)
// ============================================================================

/// Simple PRF implementation using SHA-256
/// Based on Paper: F: {0,1}* x {0,1}* → {0,1}
pub fn prf(key: &[u8], id: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(key);
    hasher.update(id.as_bytes());
    let result = hasher.finalize();
    
    // Take the first bit of the hash
    (result[0] & 1) == 1
}

/// Initialize computational correlated randomness
/// Based on Paper: Each Pi chooses random key ki and exchanges keys
pub fn init_computational_correlated_randomness() -> ComputationalCorrelatedRandomness {
    let mut rng = rand::thread_rng();
    
    // Each party chooses a random 256-bit key
    let k1: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    let k2: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    let k3: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
    
    println!("=== Computational Correlated Randomness Init ===");
    println!("P1 chooses key k1 (256 bits)");
    println!("P2 chooses key k2 (256 bits)");
    println!("P3 chooses key k3 (256 bits)");
    println!("Keys exchanged: P1→P3, P2→P1, P3→P2");
    
    ComputationalCorrelatedRandomness { k1, k2, k3 }
}

/// Get next correlated random bit using computational method
/// Based on Paper: GetNextBit with unique identifier id
pub fn get_next_correlated_bit(
    keys: &ComputationalCorrelatedRandomness, 
    id: &str
) -> CorrelatedRandomnessBoolean {
    // Each party computes their value using PRF
    let alpha = prf(&keys.k1, id) ^ prf(&keys.k2, id);  // P1: F_k1(id) ⊕ F_k2(id)
    let beta = prf(&keys.k2, id) ^ prf(&keys.k3, id);   // P2: F_k2(id) ⊕ F_k3(id)
    let gamma = prf(&keys.k3, id) ^ prf(&keys.k1, id);  // P3: F_k3(id) ⊕ F_k1(id)
    
    println!("=== Computational Correlated Randomness (id: {}) ===", id);
    println!("P1 computes α = F_k1({}) ⊕ F_k2({}) = {} ⊕ {} = {}", 
             id, id, prf(&keys.k1, id), prf(&keys.k2, id), alpha);
    println!("P2 computes β = F_k2({}) ⊕ F_k3({}) = {} ⊕ {} = {}", 
             id, id, prf(&keys.k2, id), prf(&keys.k3, id), beta);
    println!("P3 computes γ = F_k3({}) ⊕ F_k1({}) = {} ⊕ {} = {}", 
             id, id, prf(&keys.k3, id), prf(&keys.k1, id), gamma);
    println!("Verification: α ⊕ β ⊕ γ = {} ⊕ {} ⊕ {} = {}", alpha, beta, gamma, alpha ^ beta ^ gamma);
    
    CorrelatedRandomnessBoolean { alpha, beta, gamma }
}

/// Generate arithmetic correlated randomness for ring modulo 2^n
/// Based on Paper: α + β + γ = 0 mod 2^n
pub fn generate_arithmetic_correlated_randomness(modulus: u64) -> CorrelatedRandomnessArithmetic {
    let mut rng = rand::thread_rng();
    
    // Generate random values in the ring
    let alpha = rng.random_range(0..modulus);
    let beta = rng.random_range(0..modulus);
    let gamma = (modulus - ((alpha + beta) % modulus)) % modulus; // Ensures α + β + γ = 0 mod 2^n
    
    println!("=== Arithmetic Correlated Randomness (mod {}) ===", modulus);
    println!("α = {}", alpha);
    println!("β = {}", beta);
    println!("γ = {}", gamma);
    println!("Verification: α + β + γ = {} + {} + {} = {} ≡ 0 (mod {})", 
             alpha, beta, gamma, alpha + beta + gamma, modulus);
    
    CorrelatedRandomnessArithmetic { alpha, beta, gamma, modulus }
}

// ============================================================================
// LEGACY FUNCTIONS (keeping for backward compatibility)
// ============================================================================

/// Party 1 generates its part of correlated randomness
pub fn party_1_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_1 = rng.random_range(0..=1);
    
    // P1 sendet rho_1 an P2 (simuliert)
    println!("P1: Wähle ρ₁ = {}", rho_1);
    println!("P1: Sende ρ₁ = {} an P2", rho_1);
    
    // P1 empfängt ρ₃ von P3 (wird später simuliert)
    let rho_3 = rng.random_range(0..=1); // Simuliert ρ₃ von P3
    
    // P1 berechnet α = ρ₃ ⊕ ρ₁
    let alpha = rho_3 ^ rho_1;
    
    println!("P1: Empfange ρ₃ = {} von P3", rho_3);
    println!("P1: Berechne α = ρ₃ ⊕ ρ₁ = {} ⊕ {} = {}", rho_3, rho_1, alpha);
    
    PartyState {
        rho: rho_1,
        received: rho_3,
        computed_value: alpha,
        party_id: "P1".to_string(),
    }
}

/// Party 2 generates its part of correlated randomness
pub fn party_2_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_2 = rng.random_range(0..=1);
    
    // P2 sendet rho_2 an P3 (simuliert)
    println!("P2: Wähle ρ₂ = {}", rho_2);
    println!("P2: Sende ρ₂ = {} an P3", rho_2);
    
    // P2 empfängt ρ₁ von P1 (wird später simuliert)
    let rho_1 = rng.random_range(0..=1); // Simuliert ρ₁ von P1
    
    // P2 berechnet β = ρ₁ ⊕ ρ₂
    let beta = rho_1 ^ rho_2;
    
    println!("P2: Empfange ρ₁ = {} von P1", rho_1);
    println!("P2: Berechne β = ρ₁ ⊕ ρ₂ = {} ⊕ {} = {}", rho_1, rho_2, beta);
    
    PartyState {
        rho: rho_2,
        received: rho_1,
        computed_value: beta,
        party_id: "P2".to_string(),
    }
}

/// Party 3 generates its part of correlated randomness
pub fn party_3_generate() -> PartyState {
    let mut rng = rand::thread_rng();
    let rho_3 = rng.random_range(0..=1);
    
    // P3 sendet rho_3 an P3 (simuliert)
    println!("P3: Wähle ρ₃ = {}", rho_3);
    println!("P3: Sende ρ₃ = {} an P1", rho_3);
    
    // P3 empfängt ρ₂ von P2 (wird später simuliert)
    let rho_2 = rng.random_range(0..=1); // Simuliert ρ₂ von P2
    
    // P3 berechnet γ = ρ₂ ⊕ ρ₃
    let gamma = rho_2 ^ rho_3;
    
    println!("P3: Empfange ρ₂ = {} von P2", rho_2);
    println!("P3: Berechne γ = ρ₂ ⊕ ρ₃ = {} ⊕ {} = {}", rho_2, rho_3, gamma);
    
    PartyState {
        rho: rho_3,
        received: rho_2,
        computed_value: gamma,
        party_id: "P3".to_string(),
    }
}

/// Simulate the complete paper protocol for correlated randomness
pub fn simulate_paper_protocol() -> CorrelatedRandomnessResult {
    println!("\n=== Paper Protokoll Simulation ===");
    
    let p1_state = party_1_generate();
    let p2_state = party_2_generate();
    let p3_state = party_3_generate();
    
    let result = CorrelatedRandomnessResult {
        alpha: p1_state.computed_value,
        beta: p2_state.computed_value,
        gamma: p3_state.computed_value,
    };
    
    // Verify the correlation: α ⊕ β ⊕ γ = 0
    let verification = result.alpha ^ result.beta ^ result.gamma;
    
    println!("\n=== Verifikation ===");
    println!("α ⊕ β ⊕ γ = {} ⊕ {} ⊕ {} = {}", 
             result.alpha, result.beta, result.gamma, verification);
    
    if verification == 0 {
        println!("✅ Protokoll erfolgreich: α ⊕ β ⊕ γ = 0");
    } else {
        println!("❌ Protokoll fehlgeschlagen: α ⊕ β ⊕ γ ≠ 0");
    }
    
    result
}

/// Generate multiple correlated random bits
pub fn generate_correlated_bits(count: usize) -> Vec<(bool, bool, bool)> {
    (0..count)
        .map(|_| generate_correlated_single_bit())
        .collect()
}

/// Generate correlated randomness for a specific party
pub fn generate_for_party(party_id: &str) -> PartyState {
    match party_id {
        "P1" => party_1_generate(),
        "P2" => party_2_generate(),
        "P3" => party_3_generate(),
        _ => panic!("Unknown party ID: {}", party_id),
    }
}

// ============================================================================
// BATCH GENERATION FOR EFFICIENCY
// ============================================================================

/// Generate multiple correlated random bits efficiently using computational method
pub fn generate_batch_correlated_randomness(
    keys: &ComputationalCorrelatedRandomness,
    count: usize
) -> Vec<CorrelatedRandomnessBoolean> {
    (0..count)
        .map(|i| get_next_correlated_bit(keys, &format!("gate_{}", i)))
        .collect()
}

/// Generate multiple arithmetic correlated random values
pub fn generate_batch_arithmetic_correlated_randomness(
    modulus: u64,
    count: usize
) -> Vec<CorrelatedRandomnessArithmetic> {
    (0..count)
        .map(|_| generate_arithmetic_correlated_randomness(modulus))
        .collect()
} 