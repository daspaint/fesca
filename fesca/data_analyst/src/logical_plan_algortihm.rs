/*
Builds logical plan from SQL AST for SELECT avg(X) FROM Y WHERE Z; (right now only this query)
 */
use sqlparser::ast::{Expr, Statement}; //Expr is mathematical expression, Statement is SQL AST node (like SELECT, INSERT, etc.)

/*
List of "available" logical operations in the plan.
 */
pub enum LogicalOp {
    Scan { table: String },
    Filter { predicate: Expr, input: Box<LogicalOp> },
    Aggregate {
        aggs: Vec<Expr>, // AVG(salary)
        input: Box<LogicalOp>,
    },
}

/*
Function to dynamically build a LogicalOp-tree from a SQL AST statement (Statement::Query).
Currently supports only simple SELECT statements with aggregation and filtering.
 */
pub fn build_logical_plan(stmt: &Statement) -> LogicalOp {
    // Check if the statement is a query at all
    if let Statement::Query(q) = stmt {
        // Check if the query body (the q variable extracted from stmt) is a SELECT statement
        if let sqlparser::ast::SetExpr::Select(sel) = &*q.body {
            // If it is a SELECT, we can start building the logical plan
            let mut plan = LogicalOp::Scan {
                // sel.from is a Vec of TableWithJoins, we take the first table for simplicity
                table: sel.from[0].relation.to_string(),
            };
            // If there is a WHERE clause, we add a filter operation
            if let Some(pred) = &sel.selection {
                // sel.selection is an Option<Expr>, we unwrap it to get the predicate
                plan = LogicalOp::Filter {
                    // pred is an Expr, we clone it to use in the Filter operation
                    predicate: pred.clone(),
                    // The input for the Filter operation is the previous plan (the Scan operation)
                    input: Box::new(plan),
                };
            }
            // If there are aggregations in the SELECT statement, we add an Aggregate operation
            plan = LogicalOp::Aggregate {
                // sel.projection is a Vec of SelectItem, we filter it to get only UnnamedExpr (which are expressions like AVG(salary))
                aggs: sel.projection
                    .iter()
                    .filter_map(|p| {
                        // We check if the SelectItem is an UnnamedExpr, which is an expression without an alias
                        if let sqlparser::ast::SelectItem::UnnamedExpr(e) = p {
                            // If it is, we return Some(e) to include it in the aggregation list
                            // If it is not, we return None to exclude it
                            Some(e.clone())
                        } else { None }
                    })
                    // We collect the filtered expressions into a Vec<Expr>
                    .collect(),
            // The input for the Aggregate operation is the previous plan (the Filter operation, or the Scan if there was no filter)
                input: Box::new(plan),
            };
            // Finally, we return the complete logical plan
            return plan;
        }
    }
    panic!("Unsupported AST for logical planner, not a SELECT");
}
