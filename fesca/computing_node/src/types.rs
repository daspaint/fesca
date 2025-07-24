use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// SECRET SHARING TYPES (from Paper Section 2.1 and 2.3)
// ============================================================================

/// Secret share for a single bit in replicated secret sharing (Boolean case)
/// Based on Paper: (3)-sharing for Boolean circuits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretShareSingleBit {
    pub x: bool,  // First component of the share (random mask)
    pub a: bool,  // Second component of the share (masked value)
}

/// Secret share for arithmetic values (ring modulo 2^n)
/// Based on Paper: (3)-sharing for arithmetic circuits
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecretShareArithmetic {
    pub x: u64,  // First component (random mask)
    pub a: u64,  // Second component (masked value)
    pub modulus: u64, // Ring modulus (2^n)
}

/// Complete set of shares for 3-party protocol
/// Based on Paper: P1 gets (x1, a1), P2 gets (x2, a2), P3 gets (x3, a3)
#[derive(Debug, Clone)]
pub struct CompleteShares {
    pub p1_share: SecretShareSingleBit,
    pub p2_share: SecretShareSingleBit,
    pub p3_share: SecretShareSingleBit,
}

// ============================================================================
// CORRELATED RANDOMNESS TYPES (from Paper Section 2.2)
// ============================================================================

/// Correlated randomness for Boolean circuits
/// Based on Paper: α ⊕ β ⊕ γ = 0
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelatedRandomnessBoolean {
    pub alpha: bool,  // P1's value: α
    pub beta: bool,   // P2's value: β  
    pub gamma: bool,  // P3's value: γ
}

/// Correlated randomness for arithmetic circuits
/// Based on Paper: α + β + γ = 0 mod 2^n
#[derive(Debug, Clone, PartialEq)]
pub struct CorrelatedRandomnessArithmetic {
    pub alpha: u64,  // P1's value: α
    pub beta: u64,   // P2's value: β
    pub gamma: u64,  // P3's value: γ
    pub modulus: u64, // Ring modulus
}

/// Party state for correlated randomness generation
#[derive(Debug, Clone)]
pub struct PartyState {
    pub rho: u8,           // The random bit this party chooses
    pub received: u8,      // The received bit from the previous party
    pub computed_value: u8, // α, β, or γ
    pub party_id: String,
}

/// Result of correlated randomness generation
#[derive(Debug, Clone)]
pub struct CorrelatedRandomnessResult {
    pub alpha: u8,  // P1's computed value: α = ρ₃ ⊕ ρ₁
    pub beta: u8,   // P2's computed value: β = ρ₁ ⊕ ρ₂
    pub gamma: u8,  // P3's computed value: γ = ρ₂ ⊕ ρ₃
}

/// Computational correlated randomness with PRF keys
/// Based on Paper: F_k(id) implementation
#[derive(Debug, Clone)]
pub struct ComputationalCorrelatedRandomness {
    pub k1: Vec<u8>,  // P1's key
    pub k2: Vec<u8>,  // P2's key  
    pub k3: Vec<u8>,  // P3's key
}

// ============================================================================
// CIRCUIT TYPES (from Paper Section 2.1)
// ============================================================================

/// Boolean circuit gate types
#[derive(Debug, Clone, PartialEq)]
pub enum GateType {
    AND,
    OR,
    XOR,
    NOT,
}

/// Circuit node representing a gate
#[derive(Debug, Clone)]
pub struct CircuitNode {
    pub gate_type: GateType,
    pub input1: Option<usize>,  // Index of first input
    pub input2: Option<usize>,  // Index of second input (None for NOT gates)
    pub output: usize,          // Index of output
    pub gate_id: String,        // Unique identifier for correlated randomness
}

/// Boolean circuit structure
#[derive(Debug, Clone)]
pub struct BooleanCircuit {
    pub nodes: Vec<CircuitNode>,
    pub input_count: usize,
    pub output_count: usize,
    pub topological_order: Vec<usize>, // Topological ordering of gates
}

/// Arithmetic circuit gate types
#[derive(Debug, Clone, PartialEq)]
pub enum ArithmeticGateType {
    ADD,
    MUL,
    SUB,
    CONST(u64), // Constant multiplication
}

/// Arithmetic circuit node
#[derive(Debug, Clone)]
pub struct ArithmeticCircuitNode {
    pub gate_type: ArithmeticGateType,
    pub input1: Option<usize>,
    pub input2: Option<usize>,
    pub output: usize,
    pub gate_id: String,
}

/// Arithmetic circuit structure
#[derive(Debug, Clone)]
pub struct ArithmeticCircuit {
    pub nodes: Vec<ArithmeticCircuitNode>,
    pub input_count: usize,
    pub output_count: usize,
    pub modulus: u64,
    pub topological_order: Vec<usize>,
}

// ============================================================================
// PROTOCOL TYPES (from Paper Section 2)
// ============================================================================

/// 3-party MPC protocol state
#[derive(Debug, Clone)]
pub struct MPCProtocolState {
    pub party_id: usize, // 1, 2, or 3
    pub shares: HashMap<usize, SecretShareSingleBit>, // wire_id -> share
    pub correlated_randomness: HashMap<String, CorrelatedRandomnessBoolean>,
    pub communication_rounds: usize,
    pub total_operations: usize,
}

/// Protocol message types for inter-party communication
#[derive(Debug, Clone)]
pub enum ProtocolMessage {
    Share(usize, SecretShareSingleBit), // wire_id, share
    CorrelatedRandomness(String, CorrelatedRandomnessBoolean), // gate_id, randomness
    MultiplicationResult(usize, bool), // wire_id, r_value
    ReconstructionRequest(usize), // wire_id
    ReconstructionResponse(usize, bool), // wire_id, share_value
}

/// Cost model for protocol operations
/// Based on Paper: Co (operation cost) and Cs (synchronization cost)
#[derive(Debug, Clone)]
pub struct CostModel {
    pub operation_cost: usize,      // Co: number of primitive operations
    pub synchronization_cost: usize, // Cs: number of communication rounds
    pub communication_bits: usize,   // Total bits communicated
}

/// Performance metrics for the protocol
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_gates: usize,
    pub xor_gates: usize,
    pub and_gates: usize,
    pub total_rounds: usize,
    pub total_operations: usize,
    pub total_communication: usize,
    pub execution_time_ms: u64,
}

// ============================================================================
// DATABASE OPERATOR TYPES (from Paper Section 4)
// ============================================================================

/// Oblivious relational operators
#[derive(Debug, Clone)]
pub enum ObliviousOperator {
    SELECT { predicate: String },
    JOIN { predicate: String },
    PROJECT { attributes: Vec<String> },
    GROUPBY { group_attributes: Vec<String>, aggregations: Vec<String> },
    ORDERBY { sort_attributes: Vec<String> },
}

/// Database relation with secret-shared data
#[derive(Debug, Clone)]
pub struct ObliviousRelation {
    pub name: String,
    pub attributes: Vec<String>,
    pub tuples: Vec<Vec<SecretShareSingleBit>>, // Each tuple is a vector of shares
    pub cardinality: usize,
}

/// Query result with cost information
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub result_relation: ObliviousRelation,
    pub cost_model: CostModel,
    pub performance_metrics: PerformanceMetrics,
} 