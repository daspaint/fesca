/*
Builds logical plan from SQL AST for SELECT avg(X) FROM Y WHERE Z; (right now only this query)
 */
use sqlparser::ast::{Expr, Statement};

/*
List of "available" logical operations in the plan.
 */
pub enum LogicalOp {
    Scan { table: String },

    Filter { predicate: Expr, input: Box<LogicalOp> },

    Aggregate {
        aggs: Vec<Expr>, // e.g. AVG(salary)
        input: Box<LogicalOp>,
    },
}

/*
Function to dynamically build a logical plan from a SQL AST statement.
Currently supports only simple SELECT statements with aggregation and filtering.
 */
pub fn build_logical_plan(stmt: &Statement) -> LogicalOp {
    if let Statement::Query(q) = stmt {
        if let sqlparser::ast::SetExpr::Select(sel) = &*q.body {
            let mut plan = LogicalOp::Scan {
                table: sel.from[0].relation.to_string(),
            };
            if let Some(pred) = &sel.selection {
                plan = LogicalOp::Filter {
                    predicate: pred.clone(),
                    input: Box::new(plan),
                };
            }
            plan = LogicalOp::Aggregate {
                aggs: sel.projection
                    .iter()
                    .filter_map(|p| {
                        if let sqlparser::ast::SelectItem::UnnamedExpr(e) = p {
                            Some(e.clone())
                        } else { None }
                    })
                    .collect(),
                input: Box::new(plan),
            };
            return plan;
        }
    }
    panic!("Unsupported AST for logical planner");
}
