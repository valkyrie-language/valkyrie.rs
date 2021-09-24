//! 函数调用表达式解析单元测试
//! 测试函数调用的解析功能，包括参数列表和嵌套调用

use valkyrie_parser::*;
use nyar_ast::*;
use nyar_core::*;

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
mod simple_function_calls {
    use super::*;

    #[test]
    fn test_function_call_no_args() {
        let expr = parse_expression("foo()").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 0);
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_one_arg() {
        let expr = parse_expression("foo(42)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 1);
                match &call.arguments[0] {
                    Expression::Literal(Literal::Integer(42)) => {},
                    _ => panic!("Expected integer literal 42, got {:?}", call.arguments[0]),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_multiple_args() {
        let expr = parse_expression("add(1, 2, 3)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "add"),
                    _ => panic!("Expected variable add, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 3);
                match (&call.arguments[0], &call.arguments[1], &call.arguments[2]) {
                    (Expression::Literal(Literal::Integer(1)), 
                     Expression::Literal(Literal::Integer(2)), 
                     Expression::Literal(Literal::Integer(3))) => {},
                    _ => panic!("Expected arguments 1, 2, 3, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_mixed_args() {
        let expr = parse_expression(r#"func(42, "hello", true)"").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "func"),
                    _ => panic!("Expected variable func, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 3);
                match (&call.arguments[0], &call.arguments[1], &call.arguments[2]) {
                    (Expression::Literal(Literal::Integer(42)), 
                     Expression::Literal(Literal::String(s)), 
                     Expression::Literal(Literal::Boolean(true))) => {
                        assert_eq!(s, "hello");
                    },
                    _ => panic!("Expected mixed arguments, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod function_calls_with_expressions {
    use super::*;

    #[test]
    fn test_function_call_with_variable_args() {
        let expr = parse_expression("func(x, y)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "func"),
                    _ => panic!("Expected variable func, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 2);
                match (&call.arguments[0], &call.arguments[1]) {
                    (Expression::Variable(x), Expression::Variable(y)) => {
                        assert_eq!(x.name, "x");
                        assert_eq!(y.name, "y");
                    },
                    _ => panic!("Expected variable arguments x, y, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_with_binary_expression_arg() {
        let expr = parse_expression("func(1 + 2)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "func"),
                    _ => panic!("Expected variable func, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 1);
                match &call.arguments[0] {
                    Expression::Binary(binary) => {
                        assert_eq!(binary.operator, BinaryOperator::Add);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                            _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                        }
                    },
                    _ => panic!("Expected binary expression argument, got {:?}", call.arguments[0]),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_with_unary_expression_arg() {
        let expr = parse_expression("func(-x)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "func"),
                    _ => panic!("Expected variable func, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 1);
                match &call.arguments[0] {
                    Expression::Unary(unary) => {
                        assert_eq!(unary.operator, UnaryOperator::Minus);
                        match &*unary.operand {
                            Expression::Variable(var) => assert_eq!(var.name, "x"),
                            _ => panic!("Expected variable x, got {:?}", unary.operand),
                        }
                    },
                    _ => panic!("Expected unary expression argument, got {:?}", call.arguments[0]),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_with_parenthesized_arg() {
        let expr = parse_expression("func((1 + 2))").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "func"),
                    _ => panic!("Expected variable func, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 1);
                match &call.arguments[0] {
                    Expression::Parenthesized(paren) => {
                        match &*paren.expression {
                            Expression::Binary(binary) => {
                                assert_eq!(binary.operator, BinaryOperator::Add);
                                match (&*binary.left, &*binary.right) {
                                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                                    _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                                }
                            },
                            _ => panic!("Expected binary expression in parentheses, got {:?}", paren.expression),
                        }
                    },
                    _ => panic!("Expected parenthesized expression argument, got {:?}", call.arguments[0]),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod nested_function_calls {
    use super::*;

    #[test]
    fn test_function_call_as_argument() {
        let expr = parse_expression("outer(inner())").unwrap();
        match expr {
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
            _ => panic!("Expected outer function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_chained_function_calls() {
        let expr = parse_expression("foo()()").unwrap();
        match expr {
            Expression::Call(outer_call) => {
                assert_eq!(outer_call.arguments.len(), 0);
                match &*outer_call.callee {
                    Expression::Call(inner_call) => {
                        match &*inner_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", inner_call.callee),
                        }
                        assert_eq!(inner_call.arguments.len(), 0);
                    },
                    _ => panic!("Expected inner function call, got {:?}", outer_call.callee),
                }
            },
            _ => panic!("Expected chained function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_multiple_nested_calls() {
        let expr = parse_expression("f(g(1), h(2))").unwrap();
        match expr {
            Expression::Call(f_call) => {
                match &*f_call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "f"),
                    _ => panic!("Expected variable f, got {:?}", f_call.callee),
                }
                assert_eq!(f_call.arguments.len(), 2);
                
                // Check first argument: g(1)
                match &f_call.arguments[0] {
                    Expression::Call(g_call) => {
                        match &*g_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "g"),
                            _ => panic!("Expected variable g, got {:?}", g_call.callee),
                        }
                        assert_eq!(g_call.arguments.len(), 1);
                        match &g_call.arguments[0] {
                            Expression::Literal(Literal::Integer(1)) => {},
                            _ => panic!("Expected integer literal 1, got {:?}", g_call.arguments[0]),
                        }
                    },
                    _ => panic!("Expected g function call, got {:?}", f_call.arguments[0]),
                }
                
                // Check second argument: h(2)
                match &f_call.arguments[1] {
                    Expression::Call(h_call) => {
                        match &*h_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "h"),
                            _ => panic!("Expected variable h, got {:?}", h_call.callee),
                        }
                        assert_eq!(h_call.arguments.len(), 1);
                        match &h_call.arguments[0] {
                            Expression::Literal(Literal::Integer(2)) => {},
                            _ => panic!("Expected integer literal 2, got {:?}", h_call.arguments[0]),
                        }
                    },
                    _ => panic!("Expected h function call, got {:?}", f_call.arguments[1]),
                }
            },
            _ => panic!("Expected f function call, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod function_calls_with_operators {
    use super::*;

    #[test]
    fn test_function_call_in_binary_expression() {
        let expr = parse_expression("foo() + bar()").unwrap();
        match expr {
            Expression::Binary(binary) => {
                assert_eq!(binary.operator, BinaryOperator::Add);
                
                // Check left side: foo()
                match &*binary.left {
                    Expression::Call(foo_call) => {
                        match &*foo_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", foo_call.callee),
                        }
                        assert_eq!(foo_call.arguments.len(), 0);
                    },
                    _ => panic!("Expected foo function call, got {:?}", binary.left),
                }
                
                // Check right side: bar()
                match &*binary.right {
                    Expression::Call(bar_call) => {
                        match &*bar_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "bar"),
                            _ => panic!("Expected variable bar, got {:?}", bar_call.callee),
                        }
                        assert_eq!(bar_call.arguments.len(), 0);
                    },
                    _ => panic!("Expected bar function call, got {:?}", binary.right),
                }
            },
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_with_unary_operator() {
        let expr = parse_expression("-foo()").unwrap();
        match expr {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Call(call) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 0);
                    },
                    _ => panic!("Expected function call, got {:?}", unary.operand),
                }
            },
            _ => panic!("Expected unary expression, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_precedence() {
        let expr = parse_expression("foo() * 2 + bar()").unwrap();
        match expr {
            Expression::Binary(add_binary) => {
                assert_eq!(add_binary.operator, BinaryOperator::Add);
                
                // Left side should be: foo() * 2
                match &*add_binary.left {
                    Expression::Binary(mul_binary) => {
                        assert_eq!(mul_binary.operator, BinaryOperator::Multiply);
                        match (&*mul_binary.left, &*mul_binary.right) {
                            (Expression::Call(foo_call), Expression::Literal(Literal::Integer(2))) => {
                                match &*foo_call.callee {
                                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                                    _ => panic!("Expected variable foo, got {:?}", foo_call.callee),
                                }
                                assert_eq!(foo_call.arguments.len(), 0);
                            },
                            _ => panic!("Expected foo() * 2, got {:?} * {:?}", mul_binary.left, mul_binary.right),
                        }
                    },
                    _ => panic!("Expected multiplication, got {:?}", add_binary.left),
                }
                
                // Right side should be: bar()
                match &*add_binary.right {
                    Expression::Call(bar_call) => {
                        match &*bar_call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "bar"),
                            _ => panic!("Expected variable bar, got {:?}", bar_call.callee),
                        }
                        assert_eq!(bar_call.arguments.len(), 0);
                    },
                    _ => panic!("Expected bar function call, got {:?}", add_binary.right),
                }
            },
            _ => panic!("Expected binary expression, got {:?}", expr),
        }
    }
}

#[cfg(test)]
mod whitespace_handling {
    use super::*;

    #[test]
    fn test_function_call_with_spaces() {
        let expr = parse_expression("  foo  (  1  ,  2  )  ").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 2);
                match (&call.arguments[0], &call.arguments[1]) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                    _ => panic!("Expected arguments 1, 2, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_with_newlines() {
        let expr = parse_expression("\nfoo(\n  1,\n  2\n)\n").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 2);
                match (&call.arguments[0], &call.arguments[1]) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                    _ => panic!("Expected arguments 1, 2, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }

    #[test]
    fn test_function_call_trailing_comma() {
        let expr = parse_expression("foo(1, 2,)").unwrap();
        match expr {
            Expression::Call(call) => {
                match &*call.callee {
                    Expression::Variable(var) => assert_eq!(var.name, "foo"),
                    _ => panic!("Expected variable foo, got {:?}", call.callee),
                }
                assert_eq!(call.arguments.len(), 2);
                match (&call.arguments[0], &call.arguments[1]) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {},
                    _ => panic!("Expected arguments 1, 2, got {:?}", call.arguments),
                }
            },
            _ => panic!("Expected function call, got {:?}", expr),
        }
    }
}