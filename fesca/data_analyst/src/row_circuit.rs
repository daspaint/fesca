// data_analyst/src/row_circuit.rs
// Drop this into your crate and `mod row_circuit;` from lib.rs or call it from a binary.

use anyhow::{Result, bail};
use std::collections::HashMap;

/// Basic gate types for our simple circuit
#[derive(Debug, Clone)]
pub enum Gate {
    Input { output: usize },             // allocates a wire index for an external input
    Const { value: bool, output: usize },// optional constant wire
    Xor { left: usize, right: usize, output: usize },
    And { left: usize, right: usize, output: usize },
    Or  { left: usize, right: usize, output: usize },
    Not { input: usize, output: usize },
    // No sequential state; all combinational.
}

/// Simple circuit structure
#[derive(Debug, Clone)]
pub struct Circuit {
    pub gates: Vec<Gate>,
    pub wire_count: usize,
    pub outputs: Vec<usize>, // indices of output wires in the circuit
}

/// Builder to make circuits conveniently
pub struct CircuitBuilder {
    gates: Vec<Gate>,
    wire_count: usize,
    // cache const wires
    const0: Option<usize>,
    const1: Option<usize>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self { gates: Vec::new(), wire_count: 0, const0: None, const1: None }
    }

    fn next_wire(&mut self) -> usize {
        let w = self.wire_count;
        self.wire_count += 1;
        w
    }

    pub fn input(&mut self) -> usize {
        let w = self.next_wire();
        self.gates.push(Gate::Input { output: w });
        w
    }

    pub fn const_wire(&mut self, val: bool) -> usize {
        if val {
            if let Some(w) = self.const1 { return w; }
        } else {
            if let Some(w) = self.const0 { return w; }
        }
        let w = self.next_wire();
        self.gates.push(Gate::Const { value: val, output: w });
        if val { self.const1 = Some(w); } else { self.const0 = Some(w); }
        w
    }

    pub fn xor(&mut self, a: usize, b: usize) -> usize {
        let out = self.next_wire();
        self.gates.push(Gate::Xor { left: a, right: b, output: out });
        out
    }

    pub fn and(&mut self, a: usize, b: usize) -> usize {
        let out = self.next_wire();
        self.gates.push(Gate::And { left: a, right: b, output: out });
        out
    }

    pub fn or(&mut self, a: usize, b: usize) -> usize {
        let out = self.next_wire();
        self.gates.push(Gate::Or { left: a, right: b, output: out });
        out
    }

    pub fn not(&mut self, a: usize) -> usize {
        let out = self.next_wire();
        self.gates.push(Gate::Not { input: a, output: out });
        out
    }

    /// full adder: returns (sum_wire, carry_out_wire)
    pub fn full_adder(&mut self, a: usize, b: usize, cin: usize) -> (usize, usize) {
        // sum = a XOR b XOR cin
        let axb = self.xor(a, b);
        let sum = self.xor(axb, cin);

        // carry = (a & b) OR (cin & (a XOR b))
        let ab = self.and(a, b);
        let cin_axb = self.and(cin, axb);
        let carry = self.or(ab, cin_axb);

        (sum, carry)
    }

    /// ripple adder: adds a + b with carry_in wire; a and b must have same length.
    /// returns (sum_wires Vec, carry_out_wire)
    pub fn ripple_adder(&mut self, a_bits: &[usize], b_bits: &[usize], carry_in: usize) -> (Vec<usize>, usize) {
        assert_eq!(a_bits.len(), b_bits.len(), "ripple_adder expects same lengths");
        let mut sums = Vec::with_capacity(a_bits.len());
        let mut c = carry_in;
        for (&ai, &bi) in a_bits.iter().zip(b_bits.iter()) {
            let (s, cout) = self.full_adder(ai, bi, c);
            sums.push(s);
            c = cout;
        }
        (sums, c)
    }

    /// Build equality-of-two-bit-vectors: returns wire that is 1 iff vectors equal
    /// eq = NOT( OR_i ( xor(a_i, b_i) ) )
    pub fn equal_bits(&mut self, a_bits: &[usize], b_bits: &[usize]) -> usize {
        assert_eq!(a_bits.len(), b_bits.len(), "equal_bits expects same lengths");
        if a_bits.is_empty() {
            // empty vectors equal trivially
            return self.const_wire(true);
        }
        let mut ors: Option<usize> = None;
        for (&ai, &bi) in a_bits.iter().zip(b_bits.iter()) {
            let x = self.xor(ai, bi); // x==1 if mismatch on this bit
            ors = Some(if let Some(prev) = ors { self.or(prev, x) } else { x });
        }
        // ors is 1 if any mismatch; equality = NOT(ors)
        let or_all = ors.unwrap();
        self.not(or_all)
    }

    pub fn finish(self, outputs: Vec<usize>) -> Circuit {
        Circuit { gates: self.gates, wire_count: self.wire_count, outputs }
    }
}

/// Build a per-row update circuit
///
/// Parameters:
/// - sum_width: width in bits of accumulator sum (LSB-first)
/// - salary_width: width in bits of salary value
/// - count_width: width in bits of count accumulator
/// - dept_bits: number of bits used for dept comparison (e.g., ASCII bytes * 8)
///
/// Returns:
/// - Circuit (gate list)
/// - mapping with input indices and output indices so caller knows where to write inputs and read outputs
#[derive(Debug, Clone)]
pub struct RowCircuitSpec {
    pub circuit: Circuit,
    pub input_map: HashMap<String, Vec<usize>>, // named inputs -> wire indices (LSB-first)
    pub single_inputs: HashMap<String, usize>,  // single-bit inputs like mask constant (optional)
    pub outputs: HashMap<String, Vec<usize>>,   // outputs map
}

pub fn build_row_update_circuit(sum_width: usize, salary_width: usize, count_width: usize, dept_bits: usize) -> RowCircuitSpec {
    let mut b = CircuitBuilder::new();

    // Inputs: sum bits, count bits, salary bits, dept bits, const_dept bits
    let sum_inputs: Vec<usize> = (0..sum_width).map(|_| b.input()).collect();
    let count_inputs: Vec<usize> = (0..count_width).map(|_| b.input()).collect();
    let salary_inputs: Vec<usize> = (0..salary_width).map(|_| b.input()).collect();
    let dept_inputs: Vec<usize> = (0..dept_bits).map(|_| b.input()).collect();
    let const_dept_inputs: Vec<usize> = (0..dept_bits).map(|_| b.input()).collect();

    // equality in-circuit (dept == const_dept) -> mask bit
    let mask = b.equal_bits(&dept_inputs, &const_dept_inputs);

    // Mask salary bits: masked_salary_i = salary_i & mask
    let mut masked_salary: Vec<usize> = Vec::with_capacity(salary_width);
    for &s in &salary_inputs {
        let m = b.and(s, mask);
        masked_salary.push(m);
    }

    // Pad masked_salary to sum_width (if salary_width < sum_width)
    let mut masked_salary_padded: Vec<usize> = Vec::with_capacity(sum_width);
    for i in 0..sum_width {
        if i < masked_salary.len() { masked_salary_padded.push(masked_salary[i]); }
        else { masked_salary_padded.push(b.const_wire(false)); } // zero padding
    }

    // carry in zero for sum
    let carry_in_zero = b.const_wire(false);
    let (sum_out, _carry_out_sum) = b.ripple_adder(&sum_inputs, &masked_salary_padded, carry_in_zero);

    // count addition: add mask (1-bit) into count_inputs (LSB)
    // prepare mask vector of length count_width
    let mut mask_vec: Vec<usize> = Vec::with_capacity(count_width);
    for i in 0..count_width {
        if i == 0 { mask_vec.push(mask); } else { mask_vec.push(b.const_wire(false)); }
    }
    let (count_out, _carry_out_count) = b.ripple_adder(&count_inputs, &mask_vec, b.const_wire(false));

    // Outputs are sum_out and count_out
    let mut outputs = Vec::new();
    outputs.extend_from_slice(&sum_out);
    outputs.extend_from_slice(&count_out);

    // Build maps for user
    let mut input_map = HashMap::new();
    input_map.insert("sum".to_string(), sum_inputs.clone());
    input_map.insert("count".to_string(), count_inputs.clone());
    input_map.insert("salary".to_string(), salary_inputs.clone());
    input_map.insert("dept".to_string(), dept_inputs.clone());
    input_map.insert("const_dept".to_string(), const_dept_inputs.clone());

    let mut single_inputs = HashMap::new(); // none here

    let mut outputs_map = HashMap::new();
    outputs_map.insert("sum_out".to_string(), sum_out.clone());
    outputs_map.insert("count_out".to_string(), count_out.clone());

    RowCircuitSpec {
        circuit: b.finish(outputs),
        input_map,
        single_inputs,
        outputs: outputs_map,
    }
}

/// Evaluate a circuit given input vector (boolean per input wire index).
///
/// `inputs` must be a Vec<bool> of length >= circuit.wire_count for all Input/Const wires.
/// The evaluator will compute gates in order and produce a Vec<bool> of length circuit.wire_count
/// containing values for all wires; caller can read the wires by index.
pub fn evaluate_circuit(c: &Circuit, inputs: &[bool]) -> Result<Vec<bool>> {
    // inputs length must be >= number of Input/Const wires indices
    // We'll simulate gates in order. We maintain a vector of wire values sized to wire_count.
    let mut wires = vec![false; c.wire_count];

    // initialize const and Input gates by scanning gates earlier (we'll fill as we go).
    // But Input and Const gates are also present in gate list; handle them in loop.

    for g in &c.gates {
        match g {
            Gate::Input { output } => {
                // input wires are assigned from `inputs` slice at the same index if provided
                // We'll accept both ways: if inputs.len() >= c.wire_count, use inputs[*output],
                // otherwise use inputs in order: fallback if inputs length < wire_count.
                if inputs.len() > *output {
                    wires[*output] = inputs[*output];
                } else {
                    // fallback: if caller provided a smaller inputs vector, we map up to its length
                    // (better to error)
                    bail!("evaluate_circuit: inputs slice too small for Input wire {}", output);
                }
            }
            Gate::Const { value, output } => {
                wires[*output] = *value;
            }
            Gate::Xor { left, right, output } => {
                wires[*output] = wires[*left] ^ wires[*right];
            }
            Gate::And { left, right, output } => {
                wires[*output] = wires[*left] & wires[*right];
            }
            Gate::Or { left, right, output } => {
                wires[*output] = wires[*left] | wires[*right];
            }
            Gate::Not { input, output } => {
                wires[*output] = !wires[*input];
            }
        }
    }

    Ok(wires)
}

/// Run the row-circuit template over a whole table of rows (decoded as bits).
/// - `rows`: Vec of rows, each row is a map of named bit-vectors ("salary", "dept", ...) LSB-first.
/// - returns final sum bits and count bits (LSB-first)
pub fn run_row_circuit_over_rows(
    spec: &RowCircuitSpec,
    rows: &Vec<HashMap<String, Vec<bool>>>,
    sum_width: usize,
    count_width: usize,
) -> Result<(Vec<bool>, Vec<bool>)> {
    // initial accumulators zero
    let mut sum_bits = vec![false; sum_width];
    let mut count_bits = vec![false; count_width];

    // wires_in_len is spec.circuit.wire_count, so we'll build inputs vector of that length and set inputs by wire index
    let wc = spec.circuit.wire_count;
    for row in rows.iter() {
        let mut inputs = vec![false; wc];

        // fill sum inputs
        let sum_in_wires = spec.input_map.get("sum").unwrap();
        for (i, &wire_idx) in sum_in_wires.iter().enumerate() {
            inputs[wire_idx] = sum_bits[i];
        }
        // fill count inputs
        let count_in_wires = spec.input_map.get("count").unwrap();
        for (i, &wire_idx) in count_in_wires.iter().enumerate() {
            inputs[wire_idx] = count_bits[i];
        }
        // fill salary inputs (if present in row)
        if let Some(salary_bits) = row.get("salary") {
            let salary_in_wires = spec.input_map.get("salary").unwrap();
            for (i, &wire_idx) in salary_in_wires.iter().enumerate() {
                inputs[wire_idx] = if i < salary_bits.len() { salary_bits[i] } else { false };
            }
        } else {
            // if salary missing, leave zeros
        }
        // fill dept inputs
        if let Some(dept_bits) = row.get("dept") {
            let dept_in_wires = spec.input_map.get("dept").unwrap();
            for (i, &wire_idx) in dept_in_wires.iter().enumerate() {
                inputs[wire_idx] = if i < dept_bits.len() { dept_bits[i] } else { false };
            }
        }
        // fill const_dept inputs from row map (or caller can set a global const)
        if let Some(const_dept_bits) = row.get("const_dept") {
            let const_wires = spec.input_map.get("const_dept").unwrap();
            for (i, &wire_idx) in const_wires.iter().enumerate() {
                inputs[wire_idx] = if i < const_dept_bits.len() { const_dept_bits[i] } else { false };
            }
        } else {
            // if const missing, leave zeros (meaning compare to empty string)
        }

        // evaluate circuit
        let wires = evaluate_circuit(&spec.circuit, &inputs)?;
        // read outputs
        let sum_out_wires = spec.outputs.get("sum_out").unwrap();
        let mut new_sum = vec![false; sum_width];
        for (i, &wire_idx) in sum_out_wires.iter().enumerate() {
            new_sum[i] = wires[wire_idx];
        }
        let count_out_wires = spec.outputs.get("count_out").unwrap();
        let mut new_count = vec![false; count_width];
        for (i, &wire_idx) in count_out_wires.iter().enumerate() {
            new_count[i] = wires[wire_idx];
        }

        sum_bits = new_sum;
        count_bits = new_count;
    }

    Ok((sum_bits, count_bits))
}
