//! 表达式语句解析单元测试
//! 测试表达式语句的解析功能，包括各种表达式作为语句

use valkyrie_parser::*;
use nyar_ast::*;
use nyar_core::*;

/// 辅助函数：解析单个语句
fn parse_statement(input: &str) -> Result<Statement, String> {
    let result = parse_string(input);
    match result {
        Ok(program) => {
            if program.statements.len() != 1 {
                return Err(format!("Expected 1 statement, got {}", program.statements.len()));
            }
            Ok(program.statements[0].clone())
        }
        Err(e) => Err(format!("Parse error: {:?}", e)),
    }
}

#[cfg(test)]
mod literal_expression_statements {
    use super::*;

    #[test]
    fn test_integer_expression_statement() {
        let stmt = parse_statement("42;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::Integer(42)) => {},
                    _ => panic!("Expected integer literal 42, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_string_expression_statement() {
        let stmt = parse_statement(r#""hello";"").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string literal \"hello\", got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_boolean_expression_statement() {
        let stmt = parse_statement("true;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::Boolean(true)) => {},
                    _ => panic!("Expected boolean literal true, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_float_expression_statement() {
        let stmt = parse_statement("3.14;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::Float(f)) => assert!((f - 3.14).abs() < f64::EPSILON),
                    _ => panic!("Expected float literal 3.14, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_null_expression_statement() {
        let stmt = parse_statement("null;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::Null) => {},
                    _ => panic!("Expected null literal, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod variable_expression_statements {
    use super::*;

    #[test]
    fn test_simple_variable_expression_statement() {
        let stmt = parse_statement("x;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Variable(var) => assert_eq!(var.name, "x"),
                    _ => panic!("Expected variable x, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_complex_variable_name_expression_statement() {
        let stmt = parse_statement("myVariableName;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Variable(var) => assert_eq!(var.name, "myVariableName"),
                    _ => panic!("Expected variable myVariableName, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_underscore_variable_expression_statement() {
        let stmt = parse_statement("_;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Variable(var) => assert_eq!(var.name, "_"),
                    _ => panic!("Expected variable _, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod binary_expression_statements {
    use super::*;

    #[test]
    fn test_addition_expression_statement() {
        let stmt = parse_statement("1 + 2;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Add);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                            _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_multiplication_expression_statement() {
        let stmt = parse_statement("3 * 4;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Multiply);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Literal(Literal::Integer(3)), Expression::Literal(Literal::Integer(4))) => {},
                            _ => panic!("Expected 3 * 4, got {:?} * {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_comparison_expression_statement() {
        let stmt = parse_statement("x == y;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Equal);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Variable(x), Expression::Variable(y)) => {
                                assert_eq!(x.name, "x");
                                assert_eq!(y.name, "y");
                            },
                            _ => panic!("Expected x == y, got {:?} == {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_logical_expression_statement() {
        let stmt = parse_statement("a && b;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::And);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Variable(a), Expression::Variable(b)) => {
                                assert_eq!(a.name, "a");
                                assert_eq!(b.name, "b");
                            },
                            _ => panic!("Expected a && b, got {:?} && {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod unary_expression_statements {
    use super::*;

    #[test]
    fn test_unary_minus_expression_statement() {
        let stmt = parse_statement("-42;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Unary(unary) => {
                        assert_eq!(unary.operator, UnaryOperator::Minus);
                        match &*unary.operand {
                            Expression::Literal(Literal::Integer(42)) => {},
                            _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                        }
                    },
                    _ => panic!("Expected unary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_unary_plus_expression_statement() {
        let stmt = parse_statement("+x;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Unary(unary) => {
                        assert_eq!(unary.operator, UnaryOperator::Plus);
                        match &*unary.operand {
                            Expression::Variable(var) => assert_eq!(var.name, "x"),
                            _ => panic!("Expected variable x, got {:?}", unary.operand),
                        }
                    },
                    _ => panic!("Expected unary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_logical_not_expression_statement() {
        let stmt = parse_statement("!flag;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Unary(unary) => {
                        assert_eq!(unary.operator, UnaryOperator::Not);
                        match &*unary.operand {
                            Expression::Variable(var) => assert_eq!(var.name, "flag"),
                            _ => panic!("Expected variable flag, got {:?}", unary.operand),
                        }
                    },
                    _ => panic!("Expected unary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod function_call_expression_statements {
    use super::*;

    #[test]
    fn test_function_call_no_args_expression_statement() {
        let stmt = parse_statement("foo();").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Call(call) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 0);
                    },
                    _ => panic!("Expected function call, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_function_call_with_args_expression_statement() {
        let stmt = parse_statement("print(1, 2, 3);").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Call(call) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "print"),
                            _ => panic!("Expected variable print, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 3);
                        match (&call.arguments[0], &call.arguments[1], &call.arguments[2]) {
                            (Expression::Literal(Literal::Integer(1)), 
                             Expression::Literal(Literal::Integer(2)), 
                             Expression::Literal(Literal::Integer(3))) => {},
                            _ => panic!("Expected arguments 1, 2, 3, got {:?}", call.arguments),
                        }
                    },
                    _ => panic!("Expected function call, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_nested_function_call_expression_statement() {
        let stmt = parse_statement("outer(inner());").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Call(outer_call) => {
                        match &*outer_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "outer"),
                            _ => panic!("Expected variable outer, got {:?}", outer_call.callee),
                        }
                        assert_eq!(outer_call.arguments.len(), 1);
                        match &outer_call.arguments[0] {
                            Expression::Call(inner_call) => {
                                match &*inner_call.callee {
                                    Expression::Variable(var) => assert_eq!(var.name, "inner"),
                                    _ => panic!("Expected variable inner, got {:?}", inner_call.callee),
                                }
                                assert_eq!(inner_call.arguments.len(), 0);
                            },
                            _ => panic!("Expected inner function call, got {:?}", outer_call.arguments[0]),
                        }
                    },
                    _ => panic!("Expected outer function call, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod parenthesized_expression_statements {
    use super::*;

    #[test]
    fn test_parenthesized_literal_expression_statement() {
        let stmt = parse_statement("(42);").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Parenthesized(paren) => {
                        match &*paren.expression {
                            Expression::Literal(Literal::Integer(42)) => {},
                            _ => panic!("Expected integer literal 42, got {:?}", paren.expression),
                        }
                    },
                    _ => panic!("Expected parenthesized expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_parenthesized_binary_expression_statement() {
        let stmt = parse_statement("(1 + 2);").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Parenthesized(paren) => {
                        match &*paren.expression {
                            Expression::Binary(binary) => {
                                assert_eq!(binary.operator, BinaryOperator::Add);
                                match (&*binary.left, &*binary.right) {
                                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                                    _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                                }
                            },
                            _ => panic!("Expected binary expression, got {:?}", paren.expression),
                        }
                    },
                    _ => panic!("Expected parenthesized expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_nested_parentheses_expression_statement() {
        let stmt = parse_statement("((x));").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Parenthesized(outer) => {
                        match &*outer.expression {
                            Expression::Parenthesized(inner) => {
                                match &*inner.expression {
                                    Expression::Variable(var) => assert_eq!(var.name, "x"),
                                    _ => panic!("Expected variable x, got {:?}", inner.expression),
                                }
                            },
                            _ => panic!("Expected inner parenthesized expression, got {:?}", outer.expression),
                        }
                    },
                    _ => panic!("Expected outer parenthesized expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod complex_expression_statements {
    use super::*;

    #[test]
    fn test_complex_arithmetic_expression_statement() {
        let stmt = parse_statement("(1 + 2) * 3 - 4;").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                // Just verify it's a binary expression (detailed parsing tested elsewhere)
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Subtract);
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_complex_logical_expression_statement() {
        let stmt = parse_statement("(a || b) && (c || d);").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                // Just verify it's a binary expression (detailed parsing tested elsewhere)
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::And);
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_function_call_with_complex_args_expression_statement() {
        let stmt = parse_statement("func(1 + 2, foo(), (x * y));").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Call(call) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "func"),
                            _ => panic!("Expected variable func, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 3);
                        // Just verify the argument types (detailed parsing tested elsewhere)
                        match (&call.arguments[0], &call.arguments[1], &call.arguments[2]) {
                            (Expression::Binary(_), Expression::Call(_), Expression::Parenthesized(_)) => {},
                            _ => panic!("Expected binary, call, parenthesized arguments, got {:?}", call.arguments),
                        }
                    },
                    _ => panic!("Expected function call, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod multiple_expression_statements {
    use super::*;

    #[test]
    fn test_multiple_expression_statements() {
        let result = parse_string("42; foo(); x + y;");
        match result {
            Ok(program) => {
                assert_eq!(program.statements.len(), 3);
                
                // Check first statement: 42;
                match &program.statements[0] {
                    Statement::Expression(expr_stmt) => {
                        match expr_stmt.expression {
                            Expression::Literal(Literal::Integer(42)) => {},
                            _ => panic!("Expected integer literal 42, got {:?}", expr_stmt.expression),
                        }
                    },
                    _ => panic!("Expected expression statement, got {:?}", program.statements[0]),
                }
                
                // Check second statement: foo();
                match &program.statements[1] {
                    Statement::Expression(expr_stmt) => {
                        match &expr_stmt.expression {
                            Expression::Call(call) => {
                                match &*call.callee {
                                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                                }
                                assert_eq!(call.arguments.len(), 0);
                            },
                            _ => panic!("Expected function call, got {:?}", expr_stmt.expression),
                        }
                    },
                    _ => panic!("Expected expression statement, got {:?}", program.statements[1]),
                }
                
                // Check third statement: x + y;
                match &program.statements[2] {
                    Statement::Expression(expr_stmt) => {
                        match &expr_stmt.expression {
                            Expression::Binary(binary) => {
                                assert_eq!(binary.operator, BinaryOperator::Add);
                                match (&*binary.left, &*binary.right) {
                                    (Expression::Variable(x), Expression::Variable(y)) => {
                                        assert_eq!(x.name, "x");
                                        assert_eq!(y.name, "y");
                                    },
                                    _ => panic!("Expected x + y, got {:?} + {:?}", binary.left, binary.right),
                                }
                            },
                            _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                        }
                    },
                    _ => panic!("Expected expression statement, got {:?}", program.statements[2]),
                }
            },
            Err(e) => panic!("Parse error: {:?}", e),
        }
    }
}

#[cfg(test)]
mod whitespace_handling {
    use super::*;

    #[test]
    fn test_expression_statement_with_spaces() {
        let stmt = parse_statement("  42  ;  ").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Literal(Literal::Integer(42)) => {},
                    _ => panic!("Expected integer literal 42, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_expression_statement_with_newlines() {
        let stmt = parse_statement("\n1 + 2\n;\n").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Add);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                            _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }

    #[test]
    fn test_expression_statement_with_tabs() {
        let stmt = parse_statement("\tfoo()\t;\t").unwrap();
        match stmt {
            Statement::Expression(expr_stmt) => {
                match expr_stmt.expression {
                    Expression::Call(call) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 0);
                    },
                    _ => panic!("Expected function call, got {:?}", expr_stmt.expression),
                }
            },
            _ => panic!("Expected expression statement, got {:?}", stmt),
        }
    }
}