mod sql;

use log::{error, info};
use anyhow::Result;
use std::process;

/// Entry point for Data Analyst
pub fn run() -> Result<()> {
    info!("Data Analyst: starting query processing");

    // Example SQL; replace with CLI arg later
    let sql_text = "SELECT AVG(salary) FROM employees WHERE dept = 'R&D';";

    match sql::parse_sql(sql_text) {
        Ok(ast) => {
            info!("Parsed AST:\n{:#?}", ast); // to do: the output is repeating itself. paste only either here or in the sql file
            sql::extract_select(&ast);
        }
        Err(e) => {
            error!("SQL parse error: {}", e);
            process::exit(1);
        }
    }

    Ok(())
}
