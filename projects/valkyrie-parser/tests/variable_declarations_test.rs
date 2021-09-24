//! 变量声明语句解析单元测试
//! 测试变量声明的解析功能，包括类型注解和初始化表达式

use nyar_ast::*;
use nyar_core::*;
use valkyrie_parser::*;

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
mod simple_variable_declarations {
    use super::*;

    #[test]
    fn test_let_declaration_with_integer() {
        let stmt = parse_statement("let x = 42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_string() {
        let stmt = parse_statement(r#"let name = "hello";""#).unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "name");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::String(s))) => assert_eq!(s, "hello"),
                    _ => panic!("Expected string literal \"hello\", got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_boolean() {
        let stmt = parse_statement("let flag = true;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "flag");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Boolean(true))) => {}
                    _ => panic!("Expected boolean literal true, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_float() {
        let stmt = parse_statement("let pi = 3.14;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "pi");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Float(f))) => assert!((f - 3.14).abs() < f64::EPSILON),
                    _ => panic!("Expected float literal 3.14, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_without_initializer() {
        let stmt = parse_statement("let x;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                assert!(var_decl.type_annotation.is_none());
                assert!(var_decl.initializer.is_none());
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod variable_declarations_with_types {
    use super::*;

    #[test]
    fn test_let_declaration_with_int_type() {
        let stmt = parse_statement("let x: int = 42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Int)) => {}
                    _ => panic!("Expected int type annotation, got {:?}", var_decl.type_annotation),
                }
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_float_type() {
        let stmt = parse_statement("let pi: float = 3.14;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "pi");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Float)) => {}
                    _ => panic!("Expected float type annotation, got {:?}", var_decl.type_annotation),
                }
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Float(f))) => assert!((f - 3.14).abs() < f64::EPSILON),
                    _ => panic!("Expected float literal 3.14, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_bool_type() {
        let stmt = parse_statement("let flag: bool = false;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "flag");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Bool)) => {}
                    _ => panic!("Expected bool type annotation, got {:?}", var_decl.type_annotation),
                }
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Boolean(false))) => {}
                    _ => panic!("Expected boolean literal false, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_custom_type() {
        let stmt = parse_statement("let obj: MyType = value;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "obj");
                match var_decl.type_annotation {
                    Some(Type::Named(name)) => assert_eq!(name, "MyType"),
                    _ => panic!("Expected MyType type annotation, got {:?}", var_decl.type_annotation),
                }
                match var_decl.initializer {
                    Some(Expression::Variable(var)) => assert_eq!(var.name, "value"),
                    _ => panic!("Expected variable value, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_type_only() {
        let stmt = parse_statement("let x: int;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Int)) => {}
                    _ => panic!("Expected int type annotation, got {:?}", var_decl.type_annotation),
                }
                assert!(var_decl.initializer.is_none());
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod variable_declarations_with_expressions {
    use super::*;

    #[test]
    fn test_let_declaration_with_binary_expression() {
        let stmt = parse_statement("let result = 1 + 2;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "result");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Binary(binary)) => {
                        assert_eq!(binary.operator, BinaryOperator::Add);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                            _ => panic!("Expected 1 + 2, got {:?} + {:?}", binary.left, binary.right),
                        }
                    }
                    _ => panic!("Expected binary expression, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_unary_expression() {
        let stmt = parse_statement("let neg = -42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "neg");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Unary(unary)) => {
                        assert_eq!(unary.operator, UnaryOperator::Minus);
                        match &*unary.operand {
                            Expression::Literal(Literal::Integer(42)) => {}
                            _ => panic!("Expected integer literal 42, got {:?}", unary.operand),
                        }
                    }
                    _ => panic!("Expected unary expression, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_function_call() {
        let stmt = parse_statement("let result = foo(1, 2);").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "result");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Call(call)) => {
                        match &*call.callee {
                            Expression::Variable(var) => assert_eq!(var.name, "foo"),
                            _ => panic!("Expected variable foo, got {:?}", call.callee),
                        }
                        assert_eq!(call.arguments.len(), 2);
                        match (&call.arguments[0], &call.arguments[1]) {
                            (Expression::Literal(Literal::Integer(1)), Expression::Literal(Literal::Integer(2))) => {}
                            _ => panic!("Expected arguments 1, 2, got {:?}", call.arguments),
                        }
                    }
                    _ => panic!("Expected function call, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_parenthesized_expression() {
        let stmt = parse_statement("let result = (1 + 2) * 3;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "result");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Binary(binary)) => {
                        assert_eq!(binary.operator, BinaryOperator::Multiply);
                        match (&*binary.left, &*binary.right) {
                            (Expression::Parenthesized(paren), Expression::Literal(Literal::Integer(3))) => {
                                match &*paren.expression {
                                    Expression::Binary(inner_binary) => {
                                        assert_eq!(inner_binary.operator, BinaryOperator::Add);
                                        match (&*inner_binary.left, &*inner_binary.right) {
                                            (
                                                Expression::Literal(Literal::Integer(1)),
                                                Expression::Literal(Literal::Integer(2)),
                                            ) => {}
                                            _ => {
                                                panic!("Expected 1 + 2, got {:?} + {:?}", inner_binary.left, inner_binary.right)
                                            }
                                        }
                                    }
                                    _ => panic!("Expected binary expression in parentheses, got {:?}", paren.expression),
                                }
                            }
                            _ => panic!("Expected (1 + 2) * 3, got {:?} * {:?}", binary.left, binary.right),
                        }
                    }
                    _ => panic!("Expected binary expression, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_let_declaration_with_variable() {
        let stmt = parse_statement("let y = x;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "y");
                assert!(var_decl.type_annotation.is_none());
                match var_decl.initializer {
                    Some(Expression::Variable(var)) => assert_eq!(var.name, "x"),
                    _ => panic!("Expected variable x, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod complex_variable_declarations {
    use super::*;

    #[test]
    fn test_let_declaration_with_complex_expression() {
        let stmt = parse_statement("let result: int = (a + b) * c - foo(x, y);").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "result");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Int)) => {}
                    _ => panic!("Expected int type annotation, got {:?}", var_decl.type_annotation),
                }
                // Just verify it's a binary expression (detailed parsing tested elsewhere)
                match var_decl.initializer {
                    Some(Expression::Binary(binary)) => {
                        assert_eq!(binary.operator, BinaryOperator::Subtract);
                    }
                    _ => panic!("Expected binary expression, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_multiple_variable_declarations() {
        let result = parse_string("let x = 1; let y = 2; let z = x + y;");
        match result {
            Ok(program) => {
                assert_eq!(program.statements.len(), 3);

                // Check first declaration: let x = 1;
                match &program.statements[0] {
                    Statement::VariableDeclaration(var_decl) => {
                        assert_eq!(var_decl.name, "x");
                        match var_decl.initializer {
                            Some(Expression::Literal(Literal::Integer(1))) => {}
                            _ => panic!("Expected integer literal 1, got {:?}", var_decl.initializer),
                        }
                    }
                    _ => panic!("Expected variable declaration, got {:?}", program.statements[0]),
                }

                // Check second declaration: let y = 2;
                match &program.statements[1] {
                    Statement::VariableDeclaration(var_decl) => {
                        assert_eq!(var_decl.name, "y");
                        match var_decl.initializer {
                            Some(Expression::Literal(Literal::Integer(2))) => {}
                            _ => panic!("Expected integer literal 2, got {:?}", var_decl.initializer),
                        }
                    }
                    _ => panic!("Expected variable declaration, got {:?}", program.statements[1]),
                }

                // Check third declaration: let z = x + y;
                match &program.statements[2] {
                    Statement::VariableDeclaration(var_decl) => {
                        assert_eq!(var_decl.name, "z");
                        match var_decl.initializer {
                            Some(Expression::Binary(binary)) => {
                                assert_eq!(binary.operator, BinaryOperator::Add);
                                match (&*binary.left, &*binary.right) {
                                    (Expression::Variable(x_var), Expression::Variable(y_var)) => {
                                        assert_eq!(x_var.name, "x");
                                        assert_eq!(y_var.name, "y");
                                    }
                                    _ => panic!("Expected x + y, got {:?} + {:?}", binary.left, binary.right),
                                }
                            }
                            _ => panic!("Expected binary expression, got {:?}", var_decl.initializer),
                        }
                    }
                    _ => panic!("Expected variable declaration, got {:?}", program.statements[2]),
                }
            }
            Err(e) => panic!("Parse error: {:?}", e),
        }
    }
}

#[cfg(test)]
mod variable_name_patterns {
    use super::*;

    #[test]
    fn test_simple_variable_names() {
        let names = ["x", "y", "z", "a", "variable", "myVar", "my_var", "var123"];
        for name in &names {
            let input = format!("let {} = 42;", name);
            let stmt = parse_statement(&input).unwrap();
            match stmt {
                Statement::VariableDeclaration(var_decl) => {
                    assert_eq!(var_decl.name, *name);
                }
                _ => panic!("Expected variable declaration for {}, got {:?}", name, stmt),
            }
        }
    }

    #[test]
    fn test_underscore_variable_name() {
        let stmt = parse_statement("let _ = 42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "_");
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_camel_case_variable_name() {
        let stmt = parse_statement("let myVariableName = 42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "myVariableName");
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_snake_case_variable_name() {
        let stmt = parse_statement("let my_variable_name = 42;").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "my_variable_name");
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }
}

#[cfg(test)]
mod whitespace_handling {
    use super::*;

    #[test]
    fn test_variable_declaration_with_spaces() {
        let stmt = parse_statement("  let   x   =   42  ;  ").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_variable_declaration_with_newlines() {
        let stmt = parse_statement("\nlet\nx\n=\n42\n;\n").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }

    #[test]
    fn test_variable_declaration_with_type_and_spaces() {
        let stmt = parse_statement("  let   x  :  int   =   42  ;  ").unwrap();
        match stmt {
            Statement::VariableDeclaration(var_decl) => {
                assert_eq!(var_decl.name, "x");
                match var_decl.type_annotation {
                    Some(Type::Primitive(PrimitiveType::Int)) => {}
                    _ => panic!("Expected int type annotation, got {:?}", var_decl.type_annotation),
                }
                match var_decl.initializer {
                    Some(Expression::Literal(Literal::Integer(42))) => {}
                    _ => panic!("Expected integer literal 42, got {:?}", var_decl.initializer),
                }
            }
            _ => panic!("Expected variable declaration, got {:?}", stmt),
        }
    }
}
