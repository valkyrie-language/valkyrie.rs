//! 二元表达式解析单元测试
//! 测试二元操作符（算术、比较、逻辑）的解析功能

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
mod arithmetic_operators {
    use super::*;

    #[test]
    fn test_addition() {
        let expr = parse_expression("1 + 2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                    _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_subtraction() {
        let expr = parse_expression("5 - 3").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Subtract);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(5)), Expression::Literal(Literal::Integer(3))) => {}
                    _ => panic!("Expected 5 - 3, got {:?} - {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_multiplication() {
        let expr = parse_expression("4 * 6").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Multiply);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(4)), Expression::Literal(Literal::Integer(6))) => {}
                    _ => panic!("Expected 4 * 6, got {:?} * {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_division() {
        let expr = parse_expression("8 / 2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Divide);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(8)), Expression::Literal(Literal::Integer(2))) => {}
                    _ => panic!("Expected 8 / 2, got {:?} / {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_modulo() {
        let expr = parse_expression("10 % 3").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Modulo);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(10)), Expression::Literal(Literal::Integer(3))) => {}
                    _ => panic!("Expected 10 % 3, got {:?} % {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod comparison_operators {
    use super::*;

    #[test]
    fn test_equal() {
        let expr = parse_expression("x == y").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Equal);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "x");
                        assert_eq!(right.name, "y");
                    }
                    _ => panic!("Expected x == y, got {:?} == {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_not_equal() {
        let expr = parse_expression("a != b").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::NotEqual);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "a");
                        assert_eq!(right.name, "b");
                    }
                    _ => panic!("Expected a != b, got {:?} != {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_less_than() {
        let expr = parse_expression("1 < 2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Less);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                    _ => panic!("Expected 1 < 2, got {:?} < {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_less_equal() {
        let expr = parse_expression("3 <= 3").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::LessEqual);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(3)), Expression::Literal(Literal::Integer(3))) => {}
                    _ => panic!("Expected 3 <= 3, got {:?} <= {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_greater_than() {
        let expr = parse_expression("5 > 4").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Greater);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(5)), Expression::Literal(Literal::Integer(4))) => {}
                    _ => panic!("Expected 5 > 4, got {:?} > {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_greater_equal() {
        let expr = parse_expression("6 >= 6").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::GreaterEqual);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(6)), Expression::Literal(Literal::Integer(6))) => {}
                    _ => panic!("Expected 6 >= 6, got {:?} >= {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod logical_operators {
    use super::*;

    #[test]
    fn test_logical_and() {
        let expr = parse_expression("true && false").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::And);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Boolean(true)), Expression::Literal(Literal::Boolean(false))) => {}
                    _ => panic!("Expected true && false, got {:?} && {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_logical_or() {
        let expr = parse_expression("false || true").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Or);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Boolean(false)), Expression::Literal(Literal::Boolean(true))) => {}
                    _ => panic!("Expected false || true, got {:?} || {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_logical_and_with_variables() {
        let expr = parse_expression("condition1 && condition2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::And);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "condition1");
                        assert_eq!(right.name, "condition2");
                    }
                    _ => panic!("Expected condition1 && condition2, got {:?} && {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_logical_or_with_variables() {
        let expr = parse_expression("flag1 || flag2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Or);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "flag1");
                        assert_eq!(right.name, "flag2");
                    }
                    _ => panic!("Expected flag1 || flag2, got {:?} || {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod operator_precedence {
    use super::*;

    #[test]
    fn test_multiplication_before_addition() {
        let expr = parse_expression("2 + 3 * 4").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(2)), Expression::Binary(right_binary)) => {
                        assert_eq!(right_binary.operator, BinaryOperator::Multiply);
                        match (&*right_binary.left, &*right_binary.right) {
                            (Expression::Literal(Literal::Integer(3)), Expression::Literal(Literal::Integer(4))) => {}
                            _ => panic!("Expected 3 * 4, got {:?} * {:?}", right_binary.left, right_binary.right),
                        }
                    }
                    _ => panic!("Expected 2 + (3 * 4), got {:?} + {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_division_before_subtraction() {
        let expr = parse_expression("10 - 8 / 2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Subtract);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(10)), Expression::Binary(right_binary)) => {
                        assert_eq!(right_binary.operator, BinaryOperator::Divide);
                        match (&*right_binary.left, &*right_binary.right) {
                            (Expression::Literal(Literal::Integer(8)), Expression::Literal(Literal::Integer(2))) => {}
                            _ => panic!("Expected 8 / 2, got {:?} / {:?}", right_binary.left, right_binary.right),
                        }
                    }
                    _ => panic!("Expected 10 - (8 / 2), got {:?} - {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_comparison_before_logical() {
        let expr = parse_expression("x < 5 && y > 3").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::And);
                match (&*binary.left, &*binary.right) {
                    (Expression::Binary(left_binary), Expression::Binary(right_binary)) => {
                        assert_eq!(left_binary.operator, BinaryOperator::Less);
                        assert_eq!(right_binary.operator, BinaryOperator::Greater);
                    }
                    _ => panic!("Expected (x < 5) && (y > 3), got {:?} && {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod associativity {
    use super::*;

    #[test]
    fn test_left_associative_addition() {
        let expr = parse_expression("1 + 2 + 3").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                match (&*binary.left, &*binary.right) {
                    (Expression::Binary(left_binary), Expression::Literal(Literal::Integer(3))) => {
                        assert_eq!(left_binary.operator, BinaryOperator::Add);
                        match (&*left_binary.left, &*left_binary.right) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                            _ => panic!("Expected 1 + 2, got {:?} + {:?}", left_binary.left, left_binary.right),
                        }
                    }
                    _ => panic!("Expected (1 + 2) + 3, got {:?} + {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_left_associative_subtraction() {
        let expr = parse_expression("10 - 3 - 2").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Subtract);
                match (&*binary.left, &*binary.right) {
                    (Expression::Binary(left_binary), Expression::Literal(Literal::Integer(2))) => {
                        assert_eq!(left_binary.operator, BinaryOperator::Subtract);
                        match (&*left_binary.left, &*left_binary.right) {
                            (Expression::Literal(Literal::Integer(10)), Expression::Literal(Literal::Integer(3))) => {}
                            _ => panic!("Expected 10 - 3, got {:?} - {:?}", left_binary.left, left_binary.right),
                        }
                    }
                    _ => panic!("Expected (10 - 3) - 2, got {:?} - {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod mixed_operands {
    use super::*;

    #[test]
    fn test_integer_and_float() {
        let expr = parse_expression("42 + 3.14").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(42)), Expression::Literal(Literal::Float(f))) => {
                        assert!((f - 3.14).abs() < f64::EPSILON);
                    }
                    _ => panic!("Expected 42 + 3.14, got {:?} + {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_variable_and_literal() {
        let expr = parse_expression("x * 5").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Multiply);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(var), Expression::Literal(Literal::Integer(5))) => {
                        assert_eq!(var.name, "x");
                    }
                    _ => panic!("Expected x * 5, got {:?} * {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_two_variables() {
        let expr = parse_expression("width / height").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Divide);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "width");
                        assert_eq!(right.name, "height");
                    }
                    _ => panic!("Expected width / height, got {:?} / {:?}", binary.left, binary.right),
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
    fn test_binary_with_spaces() {
        let expr = parse_expression("  1   +   2  ").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                    _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_binary_with_newlines() {
        let expr = parse_expression("\n3\n*\n4\n").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Multiply);
                match (&*binary.left, &*binary.right) {
                    (Expression::Literal(Literal::Integer(3)), Expression::Literal(Literal::Integer(4))) => {}
                    _ => panic!("Expected 3 * 4, got {:?} * {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_binary_with_tabs() {
        let expr = parse_expression("\tx\t==\ty\t").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Equal);
                match (&*binary.left, &*binary.right) {
                    (Expression::Variable(left), Expression::Variable(right)) => {
                        assert_eq!(left.name, "x");
                        assert_eq!(right.name, "y");
                    }
                    _ => panic!("Expected x == y, got {:?} == {:?}", binary.left, binary.right),
                }
            }
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}
