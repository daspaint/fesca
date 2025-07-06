use log::{info, warn};
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::{Parser, ParserError};

/// Parse SQL into AST
pub fn parse_sql(sql: &str) -> Result<Vec<Statement>, ParserError> {
    let dialect = GenericDialect {};
    Parser::parse_sql(&dialect, sql)
}

/// Log any SELECT queries found in the AST
pub fn extract_select(ast: &[Statement]) {
    for stmt in ast {
        match stmt {
            Statement::Query(q)  => info!("Got SELECT: {:#?}", q),
            other                => warn!("Not SELECT: {:#?}", other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_select() {
        let ast = parse_sql("SELECT 1;").unwrap();
        assert_eq!(ast.len(), 1);
                if let Statement::Query(_) = &ast[0] {
        } else {
            panic!("Expected Query");
        }
    }
}
