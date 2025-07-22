/*
Holds the enums for the logical plan.
 */

/// A boolean or arithmetic expression over columns/constants.
#[derive(Debug, Clone)]
pub enum Expr {
    Column(usize),          // col index in the scan
    LiteralInt(u64),        // e.g. 42
    LiteralString(String),  // e.g. 'R&D'
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Eq, Neq, Lt, Gt, And, Or, Plus, Minus, Mul, Div,
}

// The four logical operators
#[derive(Debug)]
pub enum LogicalPlan {
    // Read from a named table
    Scan { table_name: String, alias: Option<String> },

    // Filter rows by a predicate
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },

    // Compute / drop columns
    Project {
        input: Box<LogicalPlan>,
        exprs: Vec<(Expr, Option<String>)>, // expression + optional output alias
    },

    // Group‐by + aggregate functions
    Aggregate {
        input: Box<LogicalPlan>,
        group_exprs: Vec<Expr>,
        aggr_exprs: Vec<(AggregateFunc, Expr, Option<String>)>,
    },
}

#[derive(Debug, Clone)]
pub enum AggregateFunc {
    Sum,
    Count,
    Avg,
    Min,
    Max,
}
