/*
This is a recursive function that builds a PhysicalOp tree from a LogicalOp tree.
(logical -> physical)
 */
use crate::logical_plan::LogicalOp;
use crate::physical_plan::PhysicalOp;

pub fn build_physical_plan(lop: &LogicalOp) -> PhysicalOp {
    // we use match to see what kind of logical operation we have
    match lop {
        LogicalOp::Scan { table } => PhysicalOp::TableScan { table: table.clone() },

        LogicalOp::Filter { predicate, input } => {
            let child = build_physical_plan(input);
            PhysicalOp::Filter {
                predicate_expr: predicate.clone(),
                input: Box::new(child),
            }
        }

        LogicalOp::Aggregate { aggs, input } => {
            let child = build_physical_plan(input);
            PhysicalOp::Aggregate {
                aggs: aggs.clone(),
                input: Box::new(child),
            }
        }
    }
}
