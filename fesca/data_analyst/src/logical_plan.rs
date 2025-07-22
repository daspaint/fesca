/*
Holds the enums for the logical plan.
 */
// A simple expression in our logical plan
#[derive(Debug, Clone)]
pub enum Expr {
    Column(usize),             // column index
    LiteralInt(u64),           // e.g. 42
    LiteralString(String),     // e.g. 'R&D'
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// Supported binary operators
#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Eq,
    And,
    Plus,
    // add more as needed
}

/// The core logical operators
#[derive(Debug)]
pub enum LogicalPlan {
    Scan {
        table_name: String,
        alias: Option<String>,
    },
    Filter {
        input: Box<LogicalPlan>,
        predicate: Expr,
    },
    Project {
        input: Box<LogicalPlan>,
        exprs: Vec<(Expr, Option<String>)>,  // expr + optional alias
    },
    Aggregate {
        input: Box<LogicalPlan>,
        group_exprs: Vec<Expr>,
        aggr_exprs: Vec<(AggregateFunc, Expr, Option<String>)>,
    },
}

/// Which aggregation functions we support
#[derive(Debug, Clone)]
pub enum AggregateFunc {
    Sum,
    Count,
    Avg,
    Min,
    Max,
}
