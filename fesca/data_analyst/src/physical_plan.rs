use crate::circuit_builder::{CircuitBuilder, Circuit};
use crate::logical_plan::{LogicalPlan, Expr as LPExpr, BinaryOperator};

/// Compile a logical plan to a pure Boolean circuit using your custom builder.
pub fn compile_to_circuit(
    plan: &LogicalPlan,
    num_rows: usize,
    num_columns: usize,
) -> Circuit {
    let mut b = CircuitBuilder::new();

    // Allocate input wires: table[row][col]
    let mut table: Vec<Vec<usize>> = vec![vec![0; num_columns]; num_rows];
    for r in 0..num_rows {
        for c in 0..num_columns {
            table[r][c] = b.input();
        }
    }

    // Recursively lower plan
    fn lower(
        b: &mut CircuitBuilder,
        plan: &LogicalPlan,
        table: &Vec<Vec<usize>>,
    ) -> Vec<usize> {
        match plan {
            LogicalPlan::Scan { .. } => table.iter().flatten().cloned().collect(),

            LogicalPlan::Filter { input, predicate } => {
                let child = lower(b, input, table);
                let mut out = Vec::new();
                for (r, row) in table.iter().enumerate() {
                    let mask = compile_expr(b, &row, predicate);
                    // mask each column
                    for c in 0..row.len() {
                        out.push(b.and(mask, child[r * row.len() + c]));
                    }
                }
                out
            }

            LogicalPlan::Project { input, exprs } => {
                // evaluate each expr per row
                let mut out = Vec::new();
                // ensure child wires match table structure if needed
                for row in table.iter() {
                    for (expr, _) in exprs {
                        out.push(compile_expr(b, row, expr));
                    }
                }
                out
            }

            LogicalPlan::Aggregate { input, aggr_exprs, .. } => {
                // For boolean only, compute parity of first aggregate over all rows
                // assume single expr
                let mut bits = Vec::new();
                for row in table.iter() {
                    let w = compile_expr(b, row, &aggr_exprs[0].1);
                    bits.push(w);
                }
                // fold XOR
                let mut acc = bits[0];
                for &w in &bits[1..] {
                    acc = b.xor(acc, w);
                }
                vec![acc]
            }
        }
    }

    fn compile_expr(
        b: &mut CircuitBuilder,
        row: &[usize],
        expr: &LPExpr,
    ) -> usize {
        match expr {
            LPExpr::Column(i) => row[*i],
            LPExpr::LiteralInt(v) => if *v == 0 { b.zero() } else { b.one() },
            LPExpr::BinaryOp { op, left, right } => {
                let l = compile_expr(b, row, left);
                let r = compile_expr(b, row, right);
                match op {
                    BinaryOperator::And => b.and(l, r),
                    BinaryOperator::Plus => b.xor(l, r),
                    BinaryOperator::Eq => {
                        // NOT(xor)
                        let x = b.xor(l, r);
                        let one = b.one();
                        b.xor(x, one)
                    }
                }
            }
            _ => b.zero(),
        }
    }

    let outputs = lower(&mut b, plan, &table);
    b.finish_with_outputs(outputs)
}