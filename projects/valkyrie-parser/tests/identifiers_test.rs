//! 标识符解析单元测试
//! 测试变量名、函数名等标识符的解析功能

use nyar_ast::*;
use nyar_core::*;
use valkyrie_parser::*;

/// 辅助函数：解析单个表达式
fn parse_expression(input: &str) -> Result<Expression, String> {
    let result = parse_string(input);
    match result {
        Ok(program) => {
            if program.statements.len() != 1 {
                return Err(format!("Expected 1 statement, got {}", program.statements.len()));
            }
            match &program.statements[0] {
                Statement::Expression(expr_stmt) => Ok(expr_stmt.expression.clone()),
                _ => Err("Expected expression statement".to_string()),
            }
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[cfg(test)]
mod simple_identifiers {
    use super::*;

    #[test]
    fn test_single_letter_identifier() {
        let expr = parse_expression("x").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "x"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_simple_word_identifier() {
        let expr = parse_expression("hello").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "hello"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_with_numbers() {
        let expr = parse_expression("var123").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "var123"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_with_underscore() {
        let expr = parse_expression("my_var").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "my_var"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_starting_with_underscore() {
        let expr = parse_expression("_private").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "_private"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod complex_identifiers {
    use super::*;

    #[test]
    fn test_camel_case_identifier() {
        let expr = parse_expression("myVariable").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "myVariable"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_pascal_case_identifier() {
        let expr = parse_expression("MyClass").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "MyClass"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_snake_case_identifier() {
        let expr = parse_expression("my_long_variable_name").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "my_long_variable_name"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_mixed_case_with_numbers() {
        let expr = parse_expression("myVar123_test").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "myVar123_test"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_with_multiple_underscores() {
        let expr = parse_expression("__special__var__").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "__special__var__"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod identifier_edge_cases {
    use super::*;

    #[test]
    fn test_identifier_with_whitespace() {
        let expr = parse_expression("  myVar  ").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "myVar"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_with_newlines() {
        let expr = parse_expression("\nmyVar\n").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "myVar"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_identifier_with_tabs() {
        let expr = parse_expression("\tmyVar\t").unwrap();
        match expr {
            Expression::Variable(var) => assert_eq!(var.name, "myVar"),
            _ => panic!("Expected variable expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod reserved_words {
    use super::*;

    #[test]
    fn test_true_is_not_identifier() {
        let expr = parse_expression("true").unwrap();
        match expr {
            Expression::Literal(Literal::Boolean(true)) => {}
            _ => panic!("Expected boolean literal true, got {:?}", expr),
        }
    }

    #[test]
    fn test_false_is_not_identifier() {
        let expr = parse_expression("false").unwrap();
        match expr {
            Expression::Literal(Literal::Boolean(false)) => {}
            _ => panic!("Expected boolean literal false, got {:?}", expr),
        }
    }

    #[test]
    fn test_null_is_not_identifier() {
        let expr = parse_expression("null").unwrap();
        match expr {
            Expression::Literal(Literal::Null) => {}
            _ => panic!("Expected null literal, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod common_identifier_patterns {
    use super::*;

    #[test]
    fn test_common_variable_names() {
        let common_names = vec![
            "i", "j", "k", "x", "y", "z", "count", "index", "value", "result", "data", "item", "element", "node", "temp",
            "tmp", "buffer", "cache",
        ];

        for name in common_names {
            let expr = parse_expression(name).unwrap();
            match expr {
                Expression::Variable(var) => assert_eq!(var.name, name),
                _ => panic!("Expected variable '{}', got {:?}", name, expr),
            }
        }
    }

    #[test]
    fn test_function_like_names() {
        let function_names = vec![
            "main", "init", "setup", "cleanup", "process", "handle", "execute", "run", "create", "destroy", "update", "render",
        ];

        for name in function_names {
            let expr = parse_expression(name).unwrap();
            match expr {
                Expression::Variable(var) => assert_eq!(var.name, name),
                _ => panic!("Expected variable '{}', got {:?}", name, expr),
            }
        }
    }

    #[test]
    fn test_type_like_names() {
        let type_names = vec![
            "String", "Integer", "Boolean", "Array", "List", "Map", "Set", "Option", "Result", "Error", "Success", "Failure",
        ];

        for name in type_names {
            let expr = parse_expression(name).unwrap();
            match expr {
                Expression::Variable(var) => assert_eq!(var.name, name),
                _ => panic!("Expected variable '{}', got {:?}", name, expr),
            }
        }
    }
}
