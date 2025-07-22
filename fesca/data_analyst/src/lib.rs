// data_analyst/src/lib.rs

mod logical_plan;

use anyhow::{Result, bail};
use log::info;
use logical_plan::{Expr as LPExpr, BinaryOperator, LogicalPlan, AggregateFunc};

use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::ast::{
    Statement, Query, SetExpr, SelectItem, TableWithJoins, TableFactor,
    Expr as AstExpr, Value as AstValue, BinaryOperator as AstOp,
    Function as AstFunction, FunctionArg, FunctionArgExpr
};

/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    // 1) Hard‑coded SQL
    let sql = "SELECT AVG(salary) FROM employees WHERE dept = 'R&D'";
    info!("Running SQL statement: {}", sql);
    // 2) Parse & lower to a LogicalPlan
    let logical = sql_to_logical_plan(sql)?;
    info!("Logical plan = {:#?}", logical);

    // …next: pass `logical` into your physical‑planner → circuit builder…

    Ok(())
}

/// Parse SQL text into a single LogicalPlan
pub fn sql_to_logical_plan(sql: &str) -> Result<LogicalPlan> {
    let dialect = GenericDialect {};
    let mut stmts = Parser::parse_sql(&dialect, sql)?;
    info!("Parsed SQL AST: {:?}", stmts);
    if stmts.len() != 1 {
        bail!("Only single‑statement queries are supported");
    }
    let stmt = stmts.pop().unwrap();

    match stmt {
        Statement::Query(boxed_q) => from_query(*boxed_q),
        other => bail!("Unsupported statement: {:?}", other),
    }
}

fn from_query(query: Query) -> Result<LogicalPlan> {
    // 1) Extract the SELECT node
    let select = match *query.body {
        SetExpr::Select(box_select) => box_select,
        _ => bail!("Only simple SELECT is supported"),
    };

    // 2) FROM → Scan
    let twj: &TableWithJoins = select
        .from
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Missing FROM clause"))?;
    let (table_name, alias) = match &twj.relation {
        TableFactor::Table { name, alias: tbl_alias, .. } => {
            (name.to_string(), tbl_alias.as_ref().map(|a| a.name.value.clone()))
        }
        _ => bail!("Unsupported table factor: {:?}", twj.relation),
    };
    let mut plan = LogicalPlan::Scan { table_name, alias };

    // 3) WHERE → Filter
    if let Some(selection) = select.selection {
        let predicate = ast_expr_to_expr(selection)?;
        plan = LogicalPlan::Filter {
            input: Box::new(plan),
            predicate,
        };
    }

    // 4) Detect aggregation vs. simple projection
    let has_agg = select.projection.iter().any(|item| matches!(item,
        SelectItem::UnnamedExpr(AstExpr::Function(_)) |
        SelectItem::ExprWithAlias { expr: AstExpr::Function(_), .. }
    ));

    if has_agg {
        // a) GROUP BY expressions
        let group_exprs = select
            .group_by
            .iter()
            .map(|e| ast_expr_to_expr(e.clone()))
            .collect::<Result<Vec<_>>>()?;

        // b) Aggregates in SELECT
        let mut aggr_exprs = Vec::new();
        for item in &select.projection {
            match item {
                SelectItem::UnnamedExpr(AstExpr::Function(f)) => {
                    let (func, arg_expr) = unpack_agg(f)?;
                    let expr = ast_expr_to_expr(arg_expr)?;
                    aggr_exprs.push((func, expr, None));
                }
                SelectItem::ExprWithAlias { expr: AstExpr::Function(f), alias } => {
                    let (func, arg_expr) = unpack_agg(f)?;
                    let expr = ast_expr_to_expr(arg_expr)?;
                    aggr_exprs.push((func, expr, Some(alias.value.clone())));
                }
                _ => bail!("Mixed projection + aggregation isn't supported"),
            }
        }

        plan = LogicalPlan::Aggregate {
            input: Box::new(plan),
            group_exprs,
            aggr_exprs,
        };
    } else {
        // Pure projection
        let exprs = select
            .projection
            .iter()
            .map(|item| match item {
                SelectItem::UnnamedExpr(e) => Ok((ast_expr_to_expr(e.clone())?, None)),
                SelectItem::ExprWithAlias { expr, alias } => Ok((ast_expr_to_expr(expr.clone())?, Some(alias.value.clone()))),
                _ => bail!("Unsupported select item: {:?}", item),
            })
            .collect::<Result<_, _>>()?;

        plan = LogicalPlan::Project {
            input: Box::new(plan),
            exprs,
        };
    }

    Ok(plan)
}

/// Translate a sqlparser‑rs `Expr` into our `logical_plan::Expr`
fn ast_expr_to_expr(ast: AstExpr) -> Result<LPExpr> {
    match ast {
        AstExpr::Identifier(ident) => {
            let idx = match ident.value.as_str() {
                "dept"   => 0,
                "salary" => 1,
                _ => bail!("Unknown column {}", ident.value),
            };
            Ok(LPExpr::Column(idx))
        }

        AstExpr::Value(AstValue::Number(s, _)) => {
            let v = s.parse()?;
            Ok(LPExpr::LiteralInt(v))
        }

        AstExpr::Value(AstValue::SingleQuotedString(s)) => {
            Ok(LPExpr::LiteralString(s))
        }

        AstExpr::BinaryOp { left, op, right } => {
            let op = match op {
                AstOp::Eq   => BinaryOperator::Eq,
                AstOp::And  => BinaryOperator::And,
                AstOp::Plus => BinaryOperator::Plus,
                other => bail!("Unsupported operator {:?}", other),
            };
            let l = ast_expr_to_expr(*left)?;
            let r = ast_expr_to_expr(*right)?;
            Ok(LPExpr::BinaryOp {
                op,
                left: Box::new(l),
                right: Box::new(r),
            })
        }

        other => bail!("Unsupported AST expression {:?}", other),
    }
}

/// Pull out (AggregateFunc, inner‑expression) from an `AstFunction`
fn unpack_agg(f: &AstFunction) -> Result<(AggregateFunc, AstExpr)> {
    let name = f.name.to_string().to_uppercase();
    let func = match name.as_str() {
        "AVG"   => AggregateFunc::Avg,
        "SUM"   => AggregateFunc::Sum,
        "COUNT" => AggregateFunc::Count,
        other   => bail!("Unknown aggregate {}", other),
    };

    // Extract the first argument
    let arg_expr = match f.args.get(0) {
        Some(FunctionArg::Unnamed(FunctionArgExpr::Expr(expr))) => expr.clone(),
        Some(FunctionArg::Named { arg: FunctionArgExpr::Expr(expr), .. }) => expr.clone(),
        _ => bail!("Aggregate function must have one expression argument"),
    };

    Ok((func, arg_expr))
}
