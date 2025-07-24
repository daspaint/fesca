use crate::types::{
    SecretShareSingleBit, GateType, CircuitNode, BooleanCircuit, 
    CorrelatedRandomnessBoolean, CompleteShares, MPCProtocolState
};
use rand::Rng;

// ============================================================================
// SECRET SHARING IMPLEMENTATION (Paper Section 2.1)
// ============================================================================

/// Generate shares for a boolean value using (3)-sharing
/// Based on Paper: Dealer chooses x1, x2, x3 such that x1 ⊕ x2 ⊕ x3 = 0
/// P1 gets (x1, a1) where a1 = x3 ⊕ v
/// P2 gets (x2, a2) where a2 = x1 ⊕ v  
/// P3 gets (x3, a3) where a3 = x2 ⊕ v
pub fn generate_shares(secret: bool) -> CompleteShares {
    let mut rng = rand::thread_rng();

    // Choose random bits x1, x2, x3 such that x1 ⊕ x2 ⊕ x3 = 0
    let x1 = rng.random::<bool>();
    let x2 = rng.random::<bool>();
    let x3 = x1 ^ x2; // Ensures x1 ⊕ x2 ⊕ x3 = 0

    // Compute shares according to paper
    let p1_share = SecretShareSingleBit {
        x: x1,
        a: x3 ^ secret, // a1 = x3 ⊕ v
    };

    let p2_share = SecretShareSingleBit {
        x: x2,
        a: x1 ^ secret, // a2 = x1 ⊕ v
    };
    
    let p3_share = SecretShareSingleBit {
        x: x3,
        a: x2 ^ secret, // a3 = x2 ⊕ v
    };

    CompleteShares { p1_share, p2_share, p3_share }
}

/// Reconstruct a secret from any two shares
/// Based on Paper: Any two shares suffice to reconstruct v
pub fn reconstruct_shares(share1: &SecretShareSingleBit, share2: &SecretShareSingleBit) -> bool {
    // Reconstruct using the formula: v = (x1 ⊕ a1) ⊕ (x2 ⊕ a2)
    let reconstructed = (share1.x ^ share1.a) ^ (share2.x ^ share2.a);
    reconstructed
}

/// Verify that shares are valid (x1 ⊕ x2 ⊕ x3 = 0)
pub fn verify_shares(shares: &CompleteShares) -> bool {
    let x_sum = shares.p1_share.x ^ shares.p2_share.x ^ shares.p3_share.x;
    x_sum == false // Should be 0 (false in boolean)
}

// ============================================================================
// GATE IMPLEMENTATIONS (Paper Section 2.1)
// ============================================================================

/// XOR gate for single bit shares
/// Based on Paper: Local operation, no communication needed
/// Each party Pi locally computes (zi, ci) where zi = xi ⊕ yi and ci = ai ⊕ bi
pub fn xor_gate_single_bit(share1: SecretShareSingleBit, share2: SecretShareSingleBit) -> SecretShareSingleBit {
    println!("=== XOR Gate (Local Operation) ===");
    println!("Input shares: ({}, {}) and ({}, {})", share1.x, share1.a, share2.x, share2.a);
    
    let result = SecretShareSingleBit {
        x: share1.x ^ share2.x, // zi = xi ⊕ yi
        a: share1.a ^ share2.a, // ci = ai ⊕ bi
    };
    
    println!("Output share: ({}, {})", result.x, result.a);
    println!("No communication required!");
    
    result
}

/// AND gate for single bit shares using correlated randomness
/// Based on Paper: Requires communication and correlated randomness
/// Protocol has two phases:
/// 1. Compute (3)-XOR-sharing of AND of input bits
/// 2. Convert to defined (3)-sharing
pub fn and_gate_single_bit(
    share1: SecretShareSingleBit, 
    share2: SecretShareSingleBit, 
    correlated_randomness: &CorrelatedRandomnessBoolean
) -> SecretShareSingleBit {
    println!("=== AND Gate (With Communication) ===");
    println!("Input shares: ({}, {}) and ({}, {})", share1.x, share1.a, share2.x, share2.a);
    println!("Correlated randomness: α={}, β={}, γ={}", 
             correlated_randomness.alpha, correlated_randomness.beta, correlated_randomness.gamma);
    
    // Step 1: Compute (3)-XOR-sharing of AND of input bits
    // Based on Paper: ri = xiyi ⊕ aibi ⊕ α/β/γ
    let r1 = (share1.x & share2.x) ^ (share1.a & share2.a) ^ correlated_randomness.alpha;
    let r2 = (share1.x & share2.x) ^ (share1.a & share2.a) ^ correlated_randomness.beta;
    let r3 = (share1.x & share2.x) ^ (share1.a & share2.a) ^ correlated_randomness.gamma;
    
    println!("Step 1 - Compute r values:");
    println!("P1 computes r1 = x1y1 ⊕ a1b1 ⊕ α = {} ⊕ {} ⊕ {} = {}", 
             share1.x & share2.x, share1.a & share2.a, correlated_randomness.alpha, r1);
    println!("P2 computes r2 = x2y2 ⊕ a2b2 ⊕ β = {} ⊕ {} ⊕ {} = {}", 
             share1.x & share2.x, share1.a & share2.a, correlated_randomness.beta, r2);
    println!("P3 computes r3 = x3y3 ⊕ a3b3 ⊕ γ = {} ⊕ {} ⊕ {} = {}", 
             share1.x & share2.x, share1.a & share2.a, correlated_randomness.gamma, r3);
    
    // Communication: P1→P2, P2→P3, P3→P1
    println!("Communication: P1→P2, P2→P3, P3→P1");
    
    // Step 2: Convert to defined (3)-sharing
    // Based on Paper: zi = ri ⊕ ri-1, ci = ri
    let z1 = r1 ^ r3; // r1 ⊕ r3
    let z2 = r2 ^ r1; // r2 ⊕ r1  
    let z3 = r3 ^ r2; // r3 ⊕ r2
    
    let c1 = r1;
    let c2 = r2;
    let c3 = r3;
    
    println!("Step 2 - Convert to (3)-sharing:");
    println!("P1 stores (z1, c1) = ({}, {})", z1, c1);
    println!("P2 stores (z2, c2) = ({}, {})", z2, c2);
    println!("P3 stores (z3, c3) = ({}, {})", z3, c3);
    
    // Verify: z1 ⊕ z2 ⊕ z3 = 0
    let verification = z1 ^ z2 ^ z3;
    println!("Verification: z1 ⊕ z2 ⊕ z3 = {} ⊕ {} ⊕ {} = {}", z1, z2, z3, verification);
    
    // Return P1's share (assuming we're P1)
    SecretShareSingleBit { x: z1, a: c1 }
}

/// NOT gate for single bit shares
/// Based on Paper: Local operation, no communication needed
pub fn not_gate_single_bit(share: SecretShareSingleBit) -> SecretShareSingleBit {
    println!("=== NOT Gate (Local Operation) ===");
    println!("Input share: ({}, {})", share.x, share.a);
    
    let result = SecretShareSingleBit {
        x: share.x,
        a: !share.a,  // Only flip the 'a' component
    };
    
    println!("Output share: ({}, {})", result.x, result.a);
    println!("No communication required!");
    
    result
}

/// OR gate for single bit shares
/// Based on Paper: Can be implemented using De Morgan's law: A OR B = NOT(NOT A AND NOT B)
pub fn or_gate_single_bit(
    share1: SecretShareSingleBit, 
    share2: SecretShareSingleBit, 
    correlated_randomness: &CorrelatedRandomnessBoolean
) -> SecretShareSingleBit {
    println!("=== OR Gate (Using De Morgan's Law) ===");
    
    let not_share1 = not_gate_single_bit(share1);
    let not_share2 = not_gate_single_bit(share2);
    let and_result = and_gate_single_bit(not_share1, not_share2, correlated_randomness);
    not_gate_single_bit(and_result)
}

// ============================================================================
// CIRCUIT EVALUATION (Paper Section 2.1)
// ============================================================================

/// Evaluate a single gate with proper correlated randomness
pub fn evaluate_gate(
    gate: &CircuitNode,
    inputs: &[SecretShareSingleBit],
    correlated_randomness: &CorrelatedRandomnessBoolean
) -> SecretShareSingleBit {
    match gate.gate_type {
        GateType::XOR => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            xor_gate_single_bit(input1.clone(), input2.clone())
        },
        GateType::AND => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            and_gate_single_bit(input1.clone(), input2.clone(), correlated_randomness)
        },
        GateType::OR => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            or_gate_single_bit(input1.clone(), input2.clone(), correlated_randomness)
        },
        GateType::NOT => {
            let input1 = &inputs[gate.input1.unwrap()];
            not_gate_single_bit(input1.clone())
        },
    }
}

/// Evaluate a complete boolean circuit
/// Based on Paper: Parties compute each XOR and AND gate in predetermined topological ordering
pub fn evaluate_circuit(
    circuit: &BooleanCircuit,
    input_shares: &[SecretShareSingleBit],
    correlated_randomness: &[CorrelatedRandomnessBoolean]
) -> Vec<SecretShareSingleBit> {
    println!("=== Circuit Evaluation ===");
    println!("Input shares: {}", input_shares.len());
    println!("Gates: {}", circuit.nodes.len());
    println!("Correlated randomness: {}", correlated_randomness.len());
    
    let mut all_values = input_shares.to_vec();
    
    // Evaluate each gate in topological order
    for (i, gate) in circuit.nodes.iter().enumerate() {
        println!("\n--- Gate {}: {:?} ---", i, gate.gate_type);
        
        // Get correlated randomness for this gate
        let cr = &correlated_randomness[i % correlated_randomness.len()];
        
        let result = evaluate_gate(gate, &all_values, cr);
        
        // Ensure we have enough space for the output
        while all_values.len() <= gate.output {
            all_values.push(SecretShareSingleBit { x: false, a: false });
        }
        
        all_values[gate.output] = result;
    }
    
    // Return only the output values
    let start_idx = input_shares.len();
    let outputs = all_values[start_idx..].to_vec();
    
    println!("=== Circuit Evaluation Complete ===");
    println!("Output shares: {}", outputs.len());
    
    outputs
}

// ============================================================================
// EXAMPLE CIRCUITS (Paper Section 2.1)
// ============================================================================

/// Create a simple example circuit: (A XOR B) AND C
pub fn create_example_circuit() -> BooleanCircuit {
    BooleanCircuit {
        input_count: 3,
        output_count: 1,
        nodes: vec![
            // XOR gate: A XOR B -> temp1
            CircuitNode {
                gate_type: GateType::XOR,
                input1: Some(0),  // A
                input2: Some(1),  // B
                output: 3,        // temp1
                gate_id: "xor_1".to_string(),
            },
            // AND gate: temp1 AND C -> output
            CircuitNode {
                gate_type: GateType::AND,
                input1: Some(3),  // temp1
                input2: Some(2),  // C
                output: 4,        // output
                gate_id: "and_1".to_string(),
            },
        ],
        topological_order: vec![0, 1], // XOR first, then AND
    }
}

/// Create a more complex circuit: (A AND B) XOR (C AND D)
pub fn create_complex_circuit() -> BooleanCircuit {
    BooleanCircuit {
        input_count: 4,
        output_count: 1,
        nodes: vec![
            // AND gate: A AND B -> temp1
            CircuitNode {
                gate_type: GateType::AND,
                input1: Some(0),  // A
                input2: Some(1),  // B
                output: 4,        // temp1
                gate_id: "and_1".to_string(),
            },
            // AND gate: C AND D -> temp2
            CircuitNode {
                gate_type: GateType::AND,
                input1: Some(2),  // C
                input2: Some(3),  // D
                output: 5,        // temp2
                gate_id: "and_2".to_string(),
            },
            // XOR gate: temp1 XOR temp2 -> output
            CircuitNode {
                gate_type: GateType::XOR,
                input1: Some(4),  // temp1
                input2: Some(5),  // temp2
                output: 6,        // output
                gate_id: "xor_1".to_string(),
            },
        ],
        topological_order: vec![0, 1, 2], // ANDs first, then XOR
    }
}

// ============================================================================
// PROTOCOL STATE MANAGEMENT
// ============================================================================

/// Initialize MPC protocol state for a party
pub fn init_protocol_state(party_id: usize) -> MPCProtocolState {
    MPCProtocolState {
        party_id,
        shares: std::collections::HashMap::new(),
        correlated_randomness: std::collections::HashMap::new(),
        communication_rounds: 0,
        total_operations: 0,
    }
}

/// Add shares to protocol state
pub fn add_shares_to_state(
    state: &mut MPCProtocolState,
    wire_id: usize,
    share: SecretShareSingleBit
) {
    state.shares.insert(wire_id, share);
}

/// Get share from protocol state
pub fn get_share_from_state(
    state: &MPCProtocolState,
    wire_id: usize
) -> Option<&SecretShareSingleBit> {
    state.shares.get(&wire_id)
}

// ============================================================================
// LEGACY FUNCTIONS (keeping for backward compatibility)
// ============================================================================

/// Legacy AND gate implementation (simplified)
pub fn and_gate_single_bit_legacy(
    share1: SecretShareSingleBit, 
    share2: SecretShareSingleBit, 
    correlated_randomness: bool
) -> SecretShareSingleBit {
    // This is a simplified implementation for backward compatibility
    let result_bit = (share1.x ^ share1.a) & (share2.x ^ share2.a);
    
    // Generate new shares for the result
    let mut rng = rand::thread_rng();
    let new_x = rng.random::<bool>();
    let new_a = new_x ^ result_bit;
    
    SecretShareSingleBit {
        x: new_x,
        a: new_a,
    }
}

/// Legacy OR gate implementation (simplified)
pub fn or_gate_single_bit_legacy(
    share1: SecretShareSingleBit, 
    share2: SecretShareSingleBit, 
    correlated_randomness: bool
) -> SecretShareSingleBit {
    let not_share1 = not_gate_single_bit(share1);
    let not_share2 = not_gate_single_bit(share2);
    let and_result = and_gate_single_bit_legacy(not_share1, not_share2, correlated_randomness);
    not_gate_single_bit(and_result)
}

/// Legacy gate evaluation with bool array
pub fn evaluate_gate_legacy(
    gate: &CircuitNode,
    inputs: &[SecretShareSingleBit],
    correlated_randomness: &[bool]
) -> SecretShareSingleBit {
    match gate.gate_type {
        GateType::XOR => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            xor_gate_single_bit(input1.clone(), input2.clone())
        },
        GateType::AND => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            let cr = correlated_randomness.get(0).copied().unwrap_or(false);
            and_gate_single_bit_legacy(input1.clone(), input2.clone(), cr)
        },
        GateType::OR => {
            let input1 = &inputs[gate.input1.unwrap()];
            let input2 = &inputs[gate.input2.unwrap()];
            let cr = correlated_randomness.get(0).copied().unwrap_or(false);
            or_gate_single_bit_legacy(input1.clone(), input2.clone(), cr)
        },
        GateType::NOT => {
            let input1 = &inputs[gate.input1.unwrap()];
            not_gate_single_bit(input1.clone())
        },
    }
}

/// Legacy circuit evaluation with bool array
pub fn evaluate_circuit_legacy(
    circuit: &BooleanCircuit,
    input_shares: &[SecretShareSingleBit],
    correlated_randomness: &[bool]
) -> Vec<SecretShareSingleBit> {
    let mut all_values = input_shares.to_vec();
    
    // Evaluate each gate in the circuit
    for gate in &circuit.nodes {
        let result = evaluate_gate_legacy(gate, &all_values, correlated_randomness);
        
        // Ensure we have enough space for the output
        while all_values.len() <= gate.output {
            all_values.push(SecretShareSingleBit { x: false, a: false });
        }
        
        all_values[gate.output] = result;
    }
    
    // Return only the output values
    let start_idx = input_shares.len();
    all_values[start_idx..].to_vec()
} 