//! 一元表达式解析单元测试
//! 测试一元操作符（+、-、!）的解析功能

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
mod unary_minus {
    use super::*;

    #[test]
    fn test_negative_integer() {
        let expr = parse_expression("-42").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_negative_zero() {
        let expr = parse_expression("-0").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(0)) => {}
                    _ => panic!("Expected integer literal 0, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_negative_float() {
        let expr = parse_expression("-3.14").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Float(f)) => {
                        assert!((f - 3.14).abs() < f64::EPSILON);
                    }
                    _ => panic!("Expected float literal 3.14, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_negative_variable() {
        let expr = parse_expression("-x").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Variable(var) => assert_eq!(var.name, "x"),
                    _ => panic!("Expected variable x, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod unary_plus {
    use super::*;

    #[test]
    fn test_positive_integer() {
        let expr = parse_expression("+42").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Plus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_positive_zero() {
        let expr = parse_expression("+0").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Plus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(0)) => {}
                    _ => panic!("Expected integer literal 0, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_positive_float() {
        let expr = parse_expression("+2.71").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Plus);
                match &*unary.operand {
                    Expression::Literal(Literal::Float(f)) => {
                        assert!((f - 2.71).abs() < f64::EPSILON);
                    }
                    _ => panic!("Expected float literal 2.71, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_positive_variable() {
        let expr = parse_expression("+y").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Plus);
                match &*unary.operand {
                    Expression::Variable(var) => assert_eq!(var.name, "y"),
                    _ => panic!("Expected variable y, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod logical_not {
    use super::*;

    #[test]
    fn test_not_true() {
        let expr = parse_expression("!true").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Not);
                match &*unary.operand {
                    Expression::Literal(Literal::Boolean(true)) => {}
                    _ => panic!("Expected boolean literal true, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_not_false() {
        let expr = parse_expression("!false").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Not);
                match &*unary.operand {
                    Expression::Literal(Literal::Boolean(false)) => {}
                    _ => panic!("Expected boolean literal false, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_not_variable() {
        let expr = parse_expression("!condition").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Not);
                match &*unary.operand {
                    Expression::Variable(var) => assert_eq!(var.name, "condition"),
                    _ => panic!("Expected variable condition, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod nested_unary {
    use super::*;

    #[test]
    fn test_double_negative() {
        let expr = parse_expression("--42").unwrap();
        match expr {
            Expression::Unary(outer) => {
                assert_eq!(outer.operator, UnaryOperator::Minus);
                match &*outer.operand {
                    Expression::Unary(inner) => {
                        assert_eq!(inner.operator, UnaryOperator::Minus);
                        match &*inner.operand {
                            Expression::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected integer literal 42, got {:?}", inner.operand),
                        }
                    }
                    _ => panic!("Expected inner unary expression, got {:?}", outer.operand),
                }
            }
            _ => panic!("Expected outer unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_double_not() {
        let expr = parse_expression("!!true").unwrap();
        match expr {
            Expression::Unary(outer) => {
                assert_eq!(outer.operator, UnaryOperator::Not);
                match &*outer.operand {
                    Expression::Unary(inner) => {
                        assert_eq!(inner.operator, UnaryOperator::Not);
                        match &*inner.operand {
                            Expression::Literal(Literal::Boolean(true)) => {}
                            _ => panic!("Expected boolean literal true, got {:?}", inner.operand),
                        }
                    }
                    _ => panic!("Expected inner unary expression, got {:?}", outer.operand),
                }
            }
            _ => panic!("Expected outer unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_mixed_unary_operators() {
        let expr = parse_expression("-!true").unwrap();
        match expr {
            Expression::Unary(outer) => {
                assert_eq!(outer.operator, UnaryOperator::Minus);
                match &*outer.operand {
                    Expression::Unary(inner) => {
                        assert_eq!(inner.operator, UnaryOperator::Not);
                        match &*inner.operand {
                            Expression::Literal(Literal::Boolean(true)) => {}
                            _ => panic!("Expected boolean literal true, got {:?}", inner.operand),
                        }
                    }
                    _ => panic!("Expected inner unary expression, got {:?}", outer.operand),
                }
            }
            _ => panic!("Expected outer unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_triple_unary() {
        let expr = parse_expression("---x").unwrap();
        match expr {
            Expression::Unary(first) => {
                assert_eq!(first.operator, UnaryOperator::Minus);
                match &*first.operand {
                    Expression::Unary(second) => {
                        assert_eq!(second.operator, UnaryOperator::Minus);
                        match &*second.operand {
                            Expression::Unary(third) => {
                                assert_eq!(third.operator, UnaryOperator::Minus);
                                match &*third.operand {
                                    Expression::Variable(var) => assert_eq!(var.name, "x"),
                                    _ => panic!("Expected variable x, got {:?}", third.operand),
                                }
                            }
                            _ => panic!("Expected third unary expression, got {:?}", second.operand),
                        }
                    }
                    _ => panic!("Expected second unary expression, got {:?}", first.operand),
                }
            }
            _ => panic!("Expected first unary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod unary_with_whitespace {
    use super::*;

    #[test]
    fn test_unary_with_spaces() {
        let expr = parse_expression("  -  42  ").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_unary_with_newlines() {
        let expr = parse_expression("\n-\n42\n").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_unary_with_tabs() {
        let expr = parse_expression("\t!\ttrue\t").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Not);
                match &*unary.operand {
                    Expression::Literal(Literal::Boolean(true)) => {}
                    _ => panic!("Expected boolean literal true, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}
