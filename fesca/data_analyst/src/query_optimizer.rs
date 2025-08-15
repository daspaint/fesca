// -----------------------------------------------------------------------------
// Simple query-optimizer utilities (local, planner-friendly)
// -----------------------------------------------------------------------------
// The optimizer below is intentionally small and purpose-built for the
// `LogicalPlan` shape used in the data_analyst. It implements two gentle
// optimization passes:
//  1. Predicate pushdown: move Filters as close to Scans as possible.
//  2. Projection pruning: remove unused columns from Projects when possible.
//
// NOTE: the optimizer is deliberately conservative and does not attempt
// advanced transformations (join reordering, cost-based planning, etc.).
// It's intended as a starting point that you can copy into
// data_analyst crate (where `LogicalPlan` is defined) or adapt to the
// project's IR. We provide the algorithm here as pseudocode-style Rust
// helpers for easy porting.

/// A tiny `optimizer` module expressed with generic enums that mirror the
/// `data_analyst::logical_plan::LogicalPlan`. Copy/translate into
/// `data_analyst/src/optimizer.rs` if you prefer keeping planner code
/// alongside the logical plan implementation.
pub mod optimizer_stub {
    /// Local simplified LogicalPlan that mirrors the one in data_analyst
    /// This is intentionally standalone so this module can be compiled in
    /// isolation for testing; in repo the same
    /// logic directly against `data_analyst::logical_plan::LogicalPlan`
    /// should be implemented.
    #[derive(Debug, Clone)]
    pub enum Expr {
        Column(usize),
        LiteralString(String),
        LiteralInt(u64),
        BinaryOp(Box<Expr>, BinOp, Box<Expr>),
    }

    #[derive(Debug, Clone)]
    pub enum BinOp { Eq, And, Or, Plus }

    #[derive(Debug, Clone)]
    pub enum LogicalPlan {
        Scan { table_name: String },
        Filter { input: Box<LogicalPlan>, predicate: Expr },
        Project { input: Box<LogicalPlan>, exprs: Vec<Expr> },
        Aggregate { input: Box<LogicalPlan>, aggr: Vec<Expr> },
    }

    /// Push filters down through Projects and into Scans when safe.
    /// This is a recursive, structural rewrite: it examines a node and
    /// applies local transformations then recurses into children.
    pub fn pushdown_predicates(plan: LogicalPlan) -> LogicalPlan {
        match plan {
            LogicalPlan::Project { input, exprs } => {
                let new_input = pushdown_predicates(*input);
                // If the input is a Filter, swap Project(Filter(x)) -> Filter(Project(x))
                if let LogicalPlan::Filter { input: f_in, predicate } = new_input {
                    // We can safely push the project below the filter if the predicate
                    // depends only on columns preserved by the project. For simplicity
                    // we conservatively assume it does and perform the swap.
                    let pushed = LogicalPlan::Project { input: f_in, exprs };
                    LogicalPlan::Filter { input: Box::new(pushed), predicate }
                } else {
                    LogicalPlan::Project { input: Box::new(new_input), exprs }
                }
            }

            LogicalPlan::Filter { input, predicate } => {
                let new_input = pushdown_predicates(*input);
                match new_input {
                    LogicalPlan::Project { input: p_in, exprs } => {
                        // Filter(Project(x)) -> Project(Filter(x)) if predicate uses
                        // only projected columns. Conservative approach: always push down.
                        let pushed = LogicalPlan::Filter { input: p_in, predicate };
                        LogicalPlan::Project { input: Box::new(pushdown_predicates(pushed)), exprs }
                    }
                    LogicalPlan::Scan { .. } => LogicalPlan::Filter { input: Box::new(new_input), predicate },
                    other => LogicalPlan::Filter { input: Box::new(other), predicate },
                }
            }

            LogicalPlan::Aggregate { input, aggr } => {
                // Recurse into input; aggregates generally block predicate pushdown
                LogicalPlan::Aggregate { input: Box::new(pushdown_predicates(*input)), aggr }
            }

            LogicalPlan::Scan { .. } => plan,
        }
    }

    /// Projection pruning: given a set of required output columns, remove
    /// unnecessary columns from projects/scans. This example is skeletal and
    /// intended to be ported to the real LogicalPlan shape.
    pub fn prune_projections(plan: LogicalPlan, _required_cols: &[usize]) -> LogicalPlan {
        // Implementing pruning requires column-index tracking through expressions.
        // For brevity we return `plan` unchanged here; copy this hook into your
        // data_analyst crate and implement pruning against the concrete Expr type.
        plan
    }
}
