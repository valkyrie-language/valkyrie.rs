//! 字面量解析单元测试
//! 测试各种字面量类型的解析功能

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
mod integer_literals {
    use super::*;

    #[test]
    fn test_positive_integer() {
        let expr = parse_expression("42").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_zero() {
        let expr = parse_expression("0").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 0),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_large_integer() {
        let expr = parse_expression("123456789").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 123456789),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_single_digit() {
        for digit in 0..=9 {
            let input = digit.to_string();
            let expr = parse_expression(&input).unwrap();
            match expr {
                Expression::Literal(Literal::Integer(n)) => assert_eq!(n, digit),
                _ => panic!("Expected integer literal for {}, got {:?}", digit, expr),
            }
        }
    }
}

#[cfg(test)]
mod float_literals {
    use super::*;

    #[test]
    fn test_simple_float() {
        let expr = parse_expression("3.14").unwrap();
        match expr {
            Expression::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < f64::EPSILON),
            _ => panic!("Expected float literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_zero_float() {
        let expr = parse_expression("0.0").unwrap();
        match expr {
            Expression::Literal(Literal::Float(f)) => assert_eq!(f, 0.0),
            _ => panic!("Expected float literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_float_without_leading_zero() {
        let expr = parse_expression(".5").unwrap();
        match expr {
            Expression::Literal(Literal::Float(f)) => assert_eq!(f, 0.5),
            _ => panic!("Expected float literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_float_without_trailing_digits() {
        let expr = parse_expression("42.").unwrap();
        match expr {
            Expression::Literal(Literal::Float(f)) => assert_eq!(f, 42.0),
            _ => panic!("Expected float literal, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod string_literals {
    use super::*;

    #[test]
    fn test_simple_string() {
        let expr = parse_expression(r#""hello""#).unwrap();
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_empty_string() {
        let expr = parse_expression(r#""""#).unwrap();
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, ""),
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_string_with_spaces() {
        let expr = parse_expression(r#""hello world"""#).unwrap();
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_string_with_numbers() {
        let expr = parse_expression(r#""test123"""#).unwrap();
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "test123"),
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_string_with_special_chars() {
        let expr = parse_expression(r#""hello!@#$%^&*()"""#).unwrap();
        match expr {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello!@#$%^&*()"),
            _ => panic!("Expected string literal, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod boolean_literals {
    use super::*;

    #[test]
    fn test_true_literal() {
        let expr = parse_expression("true").unwrap();
        match expr {
            Expression::Literal(Literal::Boolean(b)) => assert_eq!(b, true),
            _ => panic!("Expected boolean literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_false_literal() {
        let expr = parse_expression("false").unwrap();
        match expr {
            Expression::Literal(Literal::Boolean(b)) => assert_eq!(b, false),
            _ => panic!("Expected boolean literal, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod null_literal {
    use super::*;

    #[test]
    fn test_null_literal() {
        let expr = parse_expression("null").unwrap();
        match expr {
            Expression::Literal(Literal::Null) => {}
            _ => panic!("Expected null literal, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod literal_edge_cases {
    use super::*;

    #[test]
    fn test_literal_with_whitespace() {
        let expr = parse_expression("  42  ").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_literal_with_newlines() {
        let expr = parse_expression("\n42\n").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }

    #[test]
    fn test_literal_with_tabs() {
        let expr = parse_expression("\t42\t").unwrap();
        match expr {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(n, 42),
            _ => panic!("Expected integer literal, got {:?}", expr),
        }
    }
}
