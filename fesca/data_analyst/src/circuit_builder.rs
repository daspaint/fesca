/*
Boolean circuit builder for AND/XOR/CONST/INPUT gates.
*/

/// Each gate in the circuit
#[derive(Debug, Clone)]
pub enum Gate {
    /// An input wire (secret-shared input)
    Input { output: usize },
    /// A constant wire: always 0 or 1
    Const { value: bool, output: usize },
    /// AND gate: output = left AND right
    And { id: usize, left: usize, right: usize, output: usize },
    /// XOR gate: output = left XOR right
    Xor { left: usize, right: usize, output: usize },
}

/// A complete Boolean circuit
#[derive(Debug)]
pub struct Circuit {
    /// Total number of wires
    pub wire_count: usize,
    /// List of gates in topological order
    pub gates: Vec<Gate>,
    /// Wires designated as public outputs
    pub outputs: Vec<usize>,
    /// The table name this circuit operates on
    pub table_name: String,
}

/// Builder for incrementally constructing a Boolean circuit
#[derive(Debug)]
pub struct CircuitBuilder {
    next_wire: usize,
    next_gate_id: usize,
    gates: Vec<Gate>,
}

impl CircuitBuilder {
    /// Create a new circuit builder
    pub fn new() -> Self {
        CircuitBuilder {
            next_wire: 0,
            next_gate_id: 0,
            gates: Vec::new(),
        }
    }

    /// Allocate a fresh wire ID for a secret-shared input
    pub fn input(&mut self) -> usize {
        let w = self.next_wire;
        self.next_wire += 1;
        self.gates.push(Gate::Input { output: w });
        w
    }

    /// Allocate a wire tied to constant zero
    pub fn zero(&mut self) -> usize {
        let w = self.next_wire;
        self.next_wire += 1;
        self.gates.push(Gate::Const { value: false, output: w });
        w
    }

    /// Allocate a wire tied to constant one
    pub fn one(&mut self) -> usize {
        let w = self.next_wire;
        self.next_wire += 1;
        self.gates.push(Gate::Const { value: true, output: w });
        w
    }

    /// AND gate
    pub fn and(&mut self, a: usize, b: usize) -> usize {
        let out = self.next_wire;
        let gid = self.next_gate_id;
        self.next_wire += 1;
        self.next_gate_id += 1;
        self.gates.push(Gate::And { id: gid, left: a, right: b, output: out });
        out
    }

    /// XOR gate
    pub fn xor(&mut self, a: usize, b: usize) -> usize {
        let out = self.next_wire;
        self.next_wire += 1;
        self.gates.push(Gate::Xor { left: a, right: b, output: out });
        out
    }

    /// Return the next gate id that will be assigned (peek)
    pub fn peek_next_gate_id(&self) -> usize {
        self.next_gate_id
    }

    /// Finalize the circuit and specify which wires are outputs
    pub fn finish_with_outputs(mut self, outputs: Vec<usize>, table_name: String) -> Circuit {
        Circuit {
            wire_count: self.next_wire,
            gates: self.gates,
            outputs,
            table_name,
        }
    }
}
