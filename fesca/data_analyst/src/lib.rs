/*
Handles the SQL parsing and conversion to a logical plan.
 */
mod logical_plan;

use anyhow::{Result, bail};
use sqlparser::ast::*;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use logical_plan::{Expr, BinaryOperator, LogicalPlan, AggregateFunc};

mod logical_plan;
mod sql;           // or wherever you put your SQL→AST helpers

use anyhow::Result;
use log::info;
use logical_plan::LogicalPlan;
use sql::sql_to_logical_plan;  // the function you pasted earlier

/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    // 1) grab your SQL
    let sql = "\
        SELECT AVG(salary) \
        FROM employees \
        WHERE dept = 'R&D'\
    ";

    // 2) parse + lower to logical plan
    let logical: LogicalPlan = sql_to_logical_plan(sql)?;
    info!("Logical plan = {:#?}", logical);

    // (you’d do more here: pass `logical` to your physical‑planner → circuit)

    Ok(())
}


// Top‐level: parse SQL and turn it into a single LogicalPlan object
pub fn sql_to_logical_plan(sql: &str) -> Result<LogicalPlan> {
    let dialect = GenericDialect {};
    let mut stmts = Parser::parse_sql(&dialect, sql)?;
    if stmts.len() != 1 {
        bail!("Only single‐statement queries are supported");
    }

    match stmts.pop().unwrap() {
        Statement::Query(box query) => from_query(*query),
        other => bail!("Unsupported statement: {:?}", other),
    }
}

fn from_query(query: Query) -> Result<LogicalPlan> {
    // Only support simple SELECT without subqueries or joins
    let body = match query.body {
        SetExpr::Select(box select) => select,
        _ => bail!("Only simple SELECT is supported"),
    };

    // 1) FROM → Scan
    let scan = {
        let table = &body.from.get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing FROM clause"))?;
        let name = &table.relation;
        let table_name = match name {
            TableFactor::Table { name, .. } => name.to_string(),
            _ => bail!("Unsupported table factor: {:?}", name),
        };
        let alias = table.alias.map(|a| a.name.to_string());
        LogicalPlan::Scan { table_name, alias }
    };

    // 2) WHERE → Filter
    let filtered = if let Some(selection) = body.selection {
        let predicate = ast_expr_to_expr(selection)?;
        LogicalPlan::Filter {
            input: Box::new(scan),
            predicate,
        }
    } else {
        scan
    };

    // 3) PROJECTION and/or AGGREGATE
    // Detect if any SELECT item is an Aggregate function
    let has_agg = body.projection.iter().any(|item| {
        matches!(item, SelectItem::ExprWithAlias { expr: Expr::Function(_), .. })
            || matches!(item, SelectItem::UnnamedExpr(Expr::Function(_)))
    });

    let plan = if has_agg {
        // Split group_by vs. aggr_exprs
        let group_exprs = body.group_by.iter()
            .map(|e| ast_expr_to_expr(e.clone()))
            .collect::<Result<Vec<_>>>()?;

        let mut aggr_exprs = Vec::new();
        for item in &body.projection {
            match item {
                SelectItem::ExprWithAlias { expr, alias } if is_agg(expr) => {
                    let (func, inner) = unpack_agg(expr)?;
                    aggr_exprs.push((func, ast_expr_to_expr(*inner)?, Some(alias.name.to_string())));
                }
                SelectItem::UnnamedExpr(expr) if is_agg(expr) => {
                    let (func, inner) = unpack_agg(expr)?;
                    aggr_exprs.push((func, ast_expr_to_expr(*inner)?, None));
                }
                _ => bail!("Mixed projection + aggregation isn't supported"),
            }
        }

        LogicalPlan::Aggregate {
            input: Box::new(filtered),
            group_exprs,
            aggr_exprs,
        }
    } else {
        // Pure projection: map each SELECT item
        let exprs = body.projection.iter().map(|item| {
            match item {
                SelectItem::ExprWithAlias { expr, alias } => {
                    Ok((ast_expr_to_expr(expr.clone())?, Some(alias.name.to_string())))
                }
                SelectItem::UnnamedExpr(expr) => {
                    Ok((ast_expr_to_expr(expr.clone())?, None))
                }
                SelectItem::Wildcard => bail!("Wildcard '*' is not supported"),
                _ => bail!("Unsupported select item: {:?}", item),
            }
        }).collect::<Result<Vec<_>>>()?;

        LogicalPlan::Project {
            input: Box::new(filtered),
            exprs,
        }
    };

    Ok(plan)
}

/// Recursively translate sqlparser‑rs `Expr` → our `Expr` IR
fn ast_expr_to_expr(ast: sqlparser::ast::Expr) -> Result<Expr> {
    use sqlparser::ast::BinaryOperator as AstOp;
    match ast {
        sqlparser::ast::Expr::Identifier(ident) => {
            // column by name — map name → index
            let idx = match ident.value.as_str() {
                "dept"      => 0,
                "salary"    => 1,
                "is_admin"  => 2,
                _ => bail!("Unknown column {}", ident.value),
            };
            Ok(Expr::Column(idx))
        }
        sqlparser::ast::Expr::Value(Value::Number(s, _)) => {
            let v = s.parse()?;
            Ok(Expr::LiteralInt(v))
        }
        sqlparser::ast::Expr::Value(Value::SingleQuotedString(s)) => {
            Ok(Expr::LiteralString(s))
        }
        sqlparser::ast::Expr::BinaryOp { left, op, right } => {
            let op = match op {
                AstOp::Eq  => BinaryOperator::Eq,
                AstOp::And => BinaryOperator::And,
                AstOp::Plus => BinaryOperator::Plus,
                // ... handle the ones you need
                _ => bail!("Unsupported operator {:?}", op),
            };
            Ok(Expr::BinaryOp {
                op,
                left: Box::new(ast_expr_to_expr(*left)?),
                right: Box::new(ast_expr_to_expr(*right)?),
            })
        }
        _ => bail!("Unsupported expression {:?}", ast),
    }
}

fn is_agg(expr: &sqlparser::ast::Expr) -> bool {
    matches!(expr, sqlparser::ast::Expr::Function(f) if ["AVG","SUM","COUNT"].contains(&f.name.to_string().as_str()))
}

fn unpack_agg(expr: &sqlparser::ast::Expr) -> Result<(AggregateFunc, Box<sqlparser::ast::Expr>)> {
    if let sqlparser::ast::Expr::Function(func) = expr {
        let name = func.name.to_string().to_uppercase();
        let func = match name.as_str() {
            "AVG"   => AggregateFunc::Avg,
            "SUM"   => AggregateFunc::Sum,
            "COUNT" => AggregateFunc::Count,
            other   => bail!("Unknown aggregate {}", other),
        };
        let inner = func.args.get(0)
            .ok_or_else(|| anyhow::anyhow!("Empty AVG/SUM"))?
            .clone();
        Ok((func, Box::new(inner)))
    } else {
        unreachable!()
    }
}
