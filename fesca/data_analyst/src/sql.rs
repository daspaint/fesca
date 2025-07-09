use log::{info, warn};
use sqlparser::ast::Statement;
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::{Parser, ParserError};

// Parse SQL into AST, also handles invalid SQL
pub fn parse_sql(sql: &str) -> Result<Vec<Statement>, ParserError> {
    let dialect = GenericDialect {};
    Parser::parse_sql(&dialect, sql)
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parses_simple_select() {
//         let ast = parse_sql("SELECT 1;").unwrap();
//         assert_eq!(ast.len(), 1);
//                 if let Statement::Query(_) = &ast[0] {
//         } else {
//             panic!("Expected Query");
//         }
//     }
// }
