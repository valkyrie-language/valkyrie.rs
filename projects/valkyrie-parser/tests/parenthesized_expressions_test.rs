//! 括号表达式解析单元测试
//! 测试括号表达式的解析功能和优先级控制

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
mod simple_parentheses {
    use super::*;

    #[test]
    fn test_parenthesized_integer() {
        let expr = parse_expression("(42)").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Literal(Literal::Integer(42)) => {}
                _ => panic!("Expected integer literal 42, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parenthesized_string() {
        let expr = parse_expression(r#"("hello")""#).unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
                _ => panic!("Expected string literal \"hello\", got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parenthesized_boolean() {
        let expr = parse_expression("(true)").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Literal(Literal::Boolean(true)) => {}
                _ => panic!("Expected boolean literal true, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parenthesized_variable() {
        let expr = parse_expression("(x)").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Variable(var) => assert_eq!(var.name, "x"),
                _ => panic!("Expected variable x, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod nested_parentheses {
    use super::*;

    #[test]
    fn test_double_parentheses() {
        let expr = parse_expression("((42))").unwrap();
        match expr {
            Expression::Parenthesized(outer) => match &*outer.expression {
                Expression::Parenthesized(inner) => match &*inner.expression {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", inner.expression),
                },
                _ => panic!("Expected inner parenthesized expression, got {:?}", outer.expression),
            },
            _ => panic!("Expected outer parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_triple_parentheses() {
        let expr = parse_expression("(((x)))").unwrap();
        match expr {
            Expression::Parenthesized(first) => match &*first.expression {
                Expression::Parenthesized(second) => match &*second.expression {
                    Expression::Parenthesized(third) => match &*third.expression {
                        Expression::Variable(var) => assert_eq!(var.name, "x"),
                        _ => panic!("Expected variable x, got {:?}", third.expression),
                    },
                    _ => panic!("Expected third parenthesized expression, got {:?}", second.expression),
                },
                _ => panic!("Expected second parenthesized expression, got {:?}", first.expression),
            },
            _ => panic!("Expected first parenthesized expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod parentheses_with_operators {
    use super::*;

    #[test]
    fn test_parenthesized_binary_expression() {
        let expr = parse_expression("(1 + 2)").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Binary(binary) => {
                    assert_eq!(binary.operator, BinaryOperator::Add);
                    match (&*binary.left, &*binary.right) {
                        (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                        _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                    }
                }
                _ => panic!("Expected binary expression, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parenthesized_unary_expression() {
        let expr = parse_expression("(-42)").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Unary(unary) => {
                    assert_eq!(unary.operator, UnaryOperator::Minus);
                    match &*unary.operand {
                        Expression::Literal(Literal::Integer(42)) => {}
                        _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                    }
                }
                _ => panic!("Expected unary expression, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_unary_of_parenthesized() {
        let expr = parse_expression("-(1 + 2)").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Parenthesized(paren) => match &*paren.expression {
                        Expression::Binary(binary) => {
                            assert_eq!(binary.operator, BinaryOperator::Add);
                            match (&*binary.left, &*binary.right) {
                                (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                                _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                            }
                        }
                        _ => panic!("Expected binary expression, got {:?}", paren.expression),
                    },
                    _ => panic!("Expected parenthesized expression, got {:?}", unary.operand),
                }
            }
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod precedence_override {
    use super::*;

    #[test]
    fn test_addition_before_multiplication() {
        let expr = parse_expression("(2 + 3) * 4").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Multiply);
                match (&*binary.left, &*binary.right) {
                    (Expression::Parenthesized(paren), Expression::Literal(Literal::Integer(4))) => match &*paren.expression {
                        Expression::Binary(inner_binary) => {
                            assert_eq!(inner_binary.operator, BinaryOperator::Add);
                            match (&*inner_binary.left, &*inner_binary.right) {
                                (Expression::Literal(Literal::Integer(2)), Expression::Literal(Literal::Integer(3))) => {}
                                _ => panic!("Expected 2 + 3, got {:?} + {:?}", inner_binary.left, inner_binary.right),
                            }
                        }
                        _ => panic!("Expected binary expression in parentheses, got {:?}", paren.expression),
                    },
                    _ => panic!("Expected (2 + 3) * 4, got {:?} * {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_subtraction_before_division() {
        let expr = parse_expression("10 / (8 - 2)").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Divide);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(10)), Expression::Parenthesized(paren)) => match &*paren.expression {
                        Expression::Binary(inner_binary) => {
                            assert_eq!(inner_binary.operator, BinaryOperator::Subtract);
                            match (&*inner_binary.left, &*inner_binary.right) {
                                (Expression::Literal(Literal::Integer(8)), Expression::Literal(Literal::Integer(2))) => {}
                                _ => panic!("Expected 8 - 2, got {:?} - {:?}", inner_binary.left, inner_binary.right),
                            }
                        }
                        _ => panic!("Expected binary expression in parentheses, got {:?}", paren.expression),
                    },
                    _ => panic!("Expected 10 / (8 - 2), got {:?} / {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_logical_or_before_and() {
        let expr = parse_expression("(a || b) && c").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::And);
                match (&*binary.left, &*binary.right) {
                    (Expression::Parenthesized(paren), Expression::Variable(var)) => {
                        assert_eq!(var.name, "c");
                        match &*paren.expression {
                            Expression::Binary(inner_binary) => {
                                assert_eq!(inner_binary.operator, BinaryOperator::Or);
                                match (&*inner_binary.left, &*inner_binary.right) {
                                    (Expression::Variable(left), Expression::Variable(right)) => {
                                        assert_eq!(left.name, "a");
                                        assert_eq!(right.name, "b");
                                    }
                                    _ => panic!("Expected a || b, got {:?} || {:?}", inner_binary.left, inner_binary.right),
                                }
                            }
                            _ => panic!("Expected binary expression in parentheses, got {:?}", paren.expression),
                        }
                    }
                    _ => panic!("Expected (a || b) && c, got {:?} && {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod complex_parentheses {
    use super::*;

    #[test]
    fn test_multiple_parenthesized_groups() {
        let expr = parse_expression("(1 + 2) * (3 + 4)").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Multiply);
                match (&*binary.left, &*binary.right) {
                    (Expression::Parenthesized(left_paren), Expression::Parenthesized(right_paren)) => {
                        // Check left parentheses: (1 + 2)
                        match &*left_paren.expression {
                            Expression::Binary(left_binary) => {
                                assert_eq!(left_binary.operator, BinaryOperator::Add);
                                match (&*left_binary.left, &*left_binary.right) {
                                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                                    _ => panic!("Expected 1 + 2, got {:?} + {:?}", left_binary.left, left_binary.right),
                                }
                            }
                            _ => panic!("Expected binary expression in left parentheses, got {:?}", left_paren.expression),
                        }
                        // Check right parentheses: (3 + 4)
                        match &*right_paren.expression {
                            Expression::Binary(right_binary) => {
                                assert_eq!(right_binary.operator, BinaryOperator::Add);
                                match (&*right_binary.left, &*right_binary.right) {
                                    (Expression::Literal(Literal::Integer(3)), Expression::Literal(Literal::Integer(4))) => {}
                                    _ => panic!("Expected 3 + 4, got {:?} + {:?}", right_binary.left, right_binary.right),
                                }
                            }
                            _ => panic!("Expected binary expression in right parentheses, got {:?}", right_paren.expression),
                        }
                    }
                    _ => panic!("Expected (1 + 2) * (3 + 4), got {:?} * {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_nested_with_different_operators() {
        let expr = parse_expression("((a + b) * c) - d").unwrap();
        match expr {
            Expression::Binary(outer_binary) => {
                assert_eq!(outer_binary.operator, BinaryOperator::Subtract);
                match (&*outer_binary.left, &*outer_binary.right) {
                    (Expression::Parenthesized(outer_paren), Expression::Variable(var)) => {
                        assert_eq!(var.name, "d");
                        match &*outer_paren.expression {
                            Expression::Binary(middle_binary) => {
                                assert_eq!(middle_binary.operator, BinaryOperator::Multiply);
                                match (&*middle_binary.left, &*middle_binary.right) {
                                    (Expression::Parenthesized(inner_paren), Expression::Variable(c_var)) => {
                                        assert_eq!(c_var.name, "c");
                                        match &*inner_paren.expression {
                                            Expression::Binary(inner_binary) => {
                                                assert_eq!(inner_binary.operator, BinaryOperator::Add);
                                                match (&*inner_binary.left, &*inner_binary.right) {
                                                    (Expression::Variable(a_var), Expression::Variable(b_var)) => {
                                                        assert_eq!(a_var.name, "a");
                                                        assert_eq!(b_var.name, "b");
                                                    }
                                                    _ => panic!(
                                                        "Expected a + b, got {:?} + {:?}",
                                                        inner_binary.left, inner_binary.right
                                                    ),
                                                }
                                            }
                                            _ => panic!(
                                                "Expected binary expression in inner parentheses, got {:?}",
                                                inner_paren.expression
                                            ),
                                        }
                                    }
                                    _ => {
                                        panic!("Expected (a + b) * c, got {:?} * {:?}", middle_binary.left, middle_binary.right)
                                    }
                                }
                            }
                            _ => panic!("Expected binary expression in outer parentheses, got {:?}", outer_paren.expression),
                        }
                    }
                    _ => panic!("Expected ((a + b) * c) - d, got {:?} - {:?}", outer_binary.left, outer_binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod whitespace_handling {
    use super::*;

    #[test]
    fn test_parentheses_with_spaces() {
        let expr = parse_expression("  (  42  )  ").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Literal(Literal::Integer(42)) => {}
                _ => panic!("Expected integer literal 42, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parentheses_with_newlines() {
        let expr = parse_expression("\n(\n1 + 2\n)\n").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Binary(binary) => {
                    assert_eq!(binary.operator, BinaryOperator::Add);
                    match (&*binary.left, &*binary.right) {
                        (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                        _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                    }
                }
                _ => panic!("Expected binary expression, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_parentheses_with_tabs() {
        let expr = parse_expression("\t(\tx\t)\t").unwrap();
        match expr {
            Expression::Parenthesized(paren) => match &*paren.expression {
                Expression::Variable(var) => assert_eq!(var.name, "x"),
                _ => panic!("Expected variable x, got {:?}", paren.expression),
            },
            _ => panic!("Expected parenthesized expression, got {:?}", expr),
        }
    }
}
