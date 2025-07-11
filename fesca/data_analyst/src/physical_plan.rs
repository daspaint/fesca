use crate::logical_plan::LogicalOp;

pub enum PhysicalOp {
    TableScan { table: String },
    // the box is used to allow recursive types (e.g. filter ... (filter...))
    Filter { predicate_expr: sqlparser::ast::Expr, input: Box<PhysicalOp> },
    Aggregate { aggs: Vec<sqlparser::ast::Expr>, input: Box<PhysicalOp> },
}
