//! Integration tests for Valkyrie parser

use nyar_ast::*;
use nyar_core::*;
use valkyrie_parser::*;

#[test]
fn test_parse_empty_program() {
    let result = parse_string("");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert!(program.statements.is_empty());
}

#[test]
fn test_parse_integer_literal() {
    let result = parse_string("42");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Literal(Literal::Integer(n)) => assert_eq!(*n, 42),
            _ => panic!("Expected integer literal"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_string_literal() {
    let result = parse_string(r#""hello world""#);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Literal(Literal::String(s)) => assert_eq!(s, "hello world"),
            _ => panic!("Expected string literal"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_boolean_literals() {
    let result = parse_string("true");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Literal(Literal::Boolean(b)) => assert!(*b),
            _ => panic!("Expected boolean literal"),
        },
        _ => panic!("Expected expression statement"),
    }

    let result = parse_string("false");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Literal(Literal::Boolean(b)) => assert!(!*b),
            _ => panic!("Expected boolean literal"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_identifier() {
    let result = parse_string("hello");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Identifier(id) => assert_eq!(id.name, "hello"),
            _ => panic!("Expected identifier"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_binary_expressions() {
    // Addition
    let result = parse_string("1 + 2");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Binary(bin) => {
                assert_eq!(bin.operator, BinaryOperator::Add);
                match (&*bin.left, &*bin.right) {
                    (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                    _ => panic!("Expected 1 + 2"),
                }
            }
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected expression statement"),
    }

    // Multiplication
    let result = parse_string("3 * 4");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Binary(bin) => {
                assert_eq!(bin.operator, BinaryOperator::Multiply);
            }
            _ => panic!("Expected binary expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_unary_expressions() {
    // Negation
    let result = parse_string("-42");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Minus);
                match &*unary.operand {
                    Expression::Literal(Literal::Integer(42)) => {}
                    _ => panic!("Expected -42"),
                }
            }
            _ => panic!("Expected unary expression"),
        },
        _ => panic!("Expected expression statement"),
    }

    // Logical not
    let result = parse_string("!true");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Unary(unary) => {
                assert_eq!(unary.operator, UnaryOperator::Not);
            }
            _ => panic!("Expected unary expression"),
        },
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_parenthesized_expressions() {
    let result = parse_string("(42)");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => match &expr_stmt.expression {
            Expression::Literal(Literal::Integer(42)) => {}
            _ => panic!("Expected integer literal"),
        },
        _ => panic!("Expected expression statement"),
    }

    let result = parse_string("(1 + 2) * 3");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Expression(expr_stmt) => {
            match &expr_stmt.expression {
                Expression::Binary(bin) => {
                    assert_eq!(bin.operator, BinaryOperator::Multiply);
                    // Left side should be (1 + 2)
                    match &*bin.left {
                        Expression::Binary(inner_bin) => {
                            assert_eq!(inner_bin.operator, BinaryOperator::Add);
                        }
                        _ => panic!("Expected binary expression on left"),
                    }
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }
}

#[test]
fn test_parse_variable_declarations() {
    // Simple let statement
    let result = parse_string("let x = 42;");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::VariableDeclaration(var_decl) => {
            assert_eq!(var_decl.name.name, "x");
            assert!(!var_decl.is_mutable);
            assert!(var_decl.initializer.is_some());
        }
        _ => panic!("Expected variable declaration"),
    }

    // Mutable let statement
    let result = parse_string("let mut y = \"hello\";");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::VariableDeclaration(var_decl) => {
            assert_eq!(var_decl.name.name, "y");
            assert!(var_decl.is_mutable);
        }
        _ => panic!("Expected variable declaration"),
    }
}

#[test]
fn test_parse_function_declarations() {
    let result = parse_string("fn hello() {}");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::FunctionDeclaration(func_decl) => {
            assert_eq!(func_decl.name.name, "hello");
            assert!(func_decl.parameters.is_empty());
            assert!(func_decl.body.statements.is_empty());
        }
        _ => panic!("Expected function declaration"),
    }
}

#[test]
fn test_parse_if_statements() {
    let result = parse_string("if true {}");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::If(if_stmt) => {
            match &if_stmt.condition {
                Expression::Literal(Literal::Boolean(true)) => {}
                _ => panic!("Expected true condition"),
            }
            assert!(if_stmt.then_branch.statements.is_empty());
            assert!(if_stmt.else_branch.is_none());
        }
        _ => panic!("Expected if statement"),
    }
}

#[test]
fn test_parse_while_statements() {
    let result = parse_string("while x > 0 {}");
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 1);

    match &program.statements[0] {
        Statement::While(while_stmt) => {
            match &while_stmt.condition {
                Expression::Binary(bin) => {
                    assert_eq!(bin.operator, BinaryOperator::Greater);
                }
                _ => panic!("Expected binary condition"),
            }
            assert!(while_stmt.body.statements.is_empty());
        }
        _ => panic!("Expected while statement"),
    }
}

#[test]
fn test_parse_return_statements() {
    // Return with value
    let result = parse_string("return 42;");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Return(ret_stmt) => {
            assert!(ret_stmt.value.is_some());
            match ret_stmt.value.as_ref().unwrap() {
                Expression::Literal(Literal::Integer(42)) => {}
                _ => panic!("Expected return 42"),
            }
        }
        _ => panic!("Expected return statement"),
    }

    // Return without value
    let result = parse_string("return;");
    assert!(result.is_ok());
    let program = result.unwrap();
    match &program.statements[0] {
        Statement::Return(ret_stmt) => {
            assert!(ret_stmt.value.is_none());
        }
        _ => panic!("Expected return statement"),
    }
}

#[test]
fn test_parse_multiple_statements() {
    let program_text = r#"
let x = 42;
let y = x + 1;
return y;
"#;

    let result = parse_string(program_text);
    assert!(result.is_ok());
    let program = result.unwrap();
    assert_eq!(program.statements.len(), 3);

    // First statement: let x = 42;
    match &program.statements[0] {
        Statement::VariableDeclaration(var_decl) => {
            assert_eq!(var_decl.name.name, "x");
        }
        _ => panic!("Expected variable declaration"),
    }

    // Second statement: let y = x + 1;
    match &program.statements[1] {
        Statement::VariableDeclaration(var_decl) => {
            assert_eq!(var_decl.name.name, "y");
        }
        _ => panic!("Expected variable declaration"),
    }

    // Third statement: return y;
    match &program.statements[2] {
        Statement::Return(_) => {}
        _ => panic!("Expected return statement"),
    }
}

#[test]
fn test_parser_configurations() {
    let parser_with_recovery = ValkyrieParser::new().with_error_recovery(true);
    let parser_without_recovery = ValkyrieParser::new().with_error_recovery(false);
    let incremental_parser = ValkyrieParser::new().with_incremental(true);

    assert!(parser_with_recovery.error_recovery);
    assert!(!parser_without_recovery.error_recovery);
    assert!(incremental_parser.incremental);
}

#[test]
fn test_tokenizer_configurations() {
    let tokenizer = ValkyrieTokenizer::new();
    let debug_tokenizer = ValkyrieTokenizer::new().with_debug(true);

    assert!(!tokenizer.debug);
    assert!(debug_tokenizer.debug);
}

#[test]
fn test_error_handling() {
    // Test parsing invalid syntax
    let result = parse_string("let = 42"); // Missing identifier
    assert!(result.is_err());

    let result = parse_string("1 + + 2"); // Invalid operator sequence
    assert!(result.is_err());

    let result = parse_string("if {}"); // Missing condition
    assert!(result.is_err());
}

#[test]
fn test_whitespace_handling() {
    let programs = ["let x=42;", "let x = 42;", "let   x   =   42  ;", "\nlet x = 42;\n", "\n\n  let x = 42;  \n\n"];

    for program in &programs {
        let result = parse_string(program);
        assert!(result.is_ok(), "Failed to parse: {}", program);
        let ast = result.unwrap();
        assert_eq!(ast.statements.len(), 1);

        match &ast.statements[0] {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name.name, "x");
            }
            _ => panic!("Expected variable declaration for: {}", program),
        }
    }
}
