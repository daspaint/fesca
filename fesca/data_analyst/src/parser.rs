/**
* Fesca Data Analyst - SQL to Logical Plan Conversion
*/
use anyhow::{Result, Context, bail};
use log::info;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;
use sqlparser::ast::{Expr, Function, SetExpr, Statement, TableFactor, SelectItem, FunctionArg, FunctionArgExpr};

/// Parses the SQL and returns (table_name, column_name, aggregation_name)
pub fn extract_execution_plan(sql: &str) -> Result<(String, String, String)> {
    // 1. Parse/validate SQL
    let dialect = GenericDialect {}; // generic SQL dialect
    let ast = Parser::parse_sql(&dialect, sql)
        .map_err(|e| anyhow::anyhow!("SQL syntax error: {}", e))?;

    // 2. Ensure single statement
    if ast.len() != 1 {
        bail!("Only a single SELECT statement is supported");
    }

    // 3. extract table name, column name, aggregation function type
    let mut table_name: Option<String> = None;
    let mut column_name: Option<String> = None;
    let mut agg_name: Option<String> = None;

    match &ast[0] {
        Statement::Query(q) => {
            if let SetExpr::Select(select) = &*q.body {
                // FROM
                if let Some(from) = select.from.get(0) {
                    match &from.relation {
                        TableFactor::Table { name, .. } => {
                            table_name = Some(name.to_string());
                        }
                        other => bail!("Unsupported FROM clause: {:?}", other),
                    }
                } else {
                    bail!("No FROM clause found");
                }

                // SELECT projection - expect a single function
                if let Some(proj) = select.projection.get(0) {
                    match proj {
                        SelectItem::UnnamedExpr(Expr::Function(Function { name, args, .. })) => {
                            agg_name = Some(name.to_string().to_uppercase());
                            // Expect first arg to be an identifier
                            if let Some(arg) = args.get(0) {
                                match arg {
                                    FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::Identifier(ident))) => {
                                        column_name = Some(ident.to_string());
                                    }
                                    FunctionArg::Unnamed(FunctionArgExpr::Expr(Expr::CompoundIdentifier(idents))) => {
                                        if let Some(last) = idents.last() {
                                            column_name = Some(last.to_string());
                                        }
                                    }
                                    other => bail!("Unsupported function argument: {:?}", other),
                                }
                            } else {
                                bail!("Aggregation function has no arguments")
                            }
                        }
                        other => bail!("Unsupported SELECT projection: {:?}. Expected an aggregation function.", other),
                    }
                } else {
                    bail!("Empty SELECT projection")
                }
            } else {
                bail!("Only simple SELECT queries are supported")
            }
        }
        other => bail!("Only SELECT queries are supported, got: {:?}", other),
    }

    let table_name = table_name.context("Couldn't extract table name from query")?;
    let column_name = column_name.context("Couldn't extract column name from query")?;
    let agg_name = agg_name.context("Couldn't extract aggregation function from query")?;

    Ok((table_name, column_name, agg_name))
}
