use std::path::Path;
use valkyrie_interpreter::{set_output_enabled, take_output, Interpreter};
use valkyrie_testing::{FnExecutor, InterpreterTestExpected, InterpreterTester, TestError};
use valkyrie_types::ValkyrieErrorKind;

fn convert_error(e: &valkyrie_types::ValkyrieError) -> TestError {
    match &e.kind {
        ValkyrieErrorKind::VmError { code, key, message } => {
            TestError { code: format!("E{:04X}", code), key: key.clone(), message: message.clone(), location: None }
        }
        ValkyrieErrorKind::RuntimeError { message } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.runtime".to_string(),
            message: message.clone(),
            location: None,
        },
        ValkyrieErrorKind::CompileError { message } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.compile".to_string(),
            message: message.clone(),
            location: None,
        },
        ValkyrieErrorKind::ParseError { message } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.parse".to_string(),
            message: message.clone(),
            location: None,
        },
        ValkyrieErrorKind::SyntaxError { message } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.syntax".to_string(),
            message: message.clone(),
            location: None,
        },
        ValkyrieErrorKind::TypeError { expected, found } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.type".to_string(),
            message: format!("expected {}, found {}", expected, found),
            location: None,
        },
        ValkyrieErrorKind::IoError { message, path } => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.io".to_string(),
            message: if let Some(p) = path { format!("{} (path: {})", message, p) } else { message.clone() },
            location: None,
        },
        ValkyrieErrorKind::Unknown => TestError {
            code: format!("E{:04X}", e.code()),
            key: "error.unknown".to_string(),
            message: "Unknown error".to_string(),
            location: None,
        },
    }
}

fn create_executor() -> impl valkyrie_testing::TestExecutor {
    FnExecutor::new(|source: &str| {
        // Disable stdout output during test execution
        set_output_enabled(false);

        // Clear any previous output
        let _ = take_output();

        let mut interpreter = Interpreter::new();
        let result = interpreter.run_script(source);

        // Capture the output from the interpreter
        let stdout = take_output();

        // Re-enable stdout for normal operation
        set_output_enabled(true);

        match result {
            Ok(value) => {
                let result = if value.is_null() { None } else { Some(format!("{:?}", value)) };
                Ok(InterpreterTestExpected { success: true, stdout, stderr: vec![], errors: vec![], result })
            }
            Err(e) => {
                let errors = vec![convert_error(&e)];
                Ok(InterpreterTestExpected { success: false, stdout, stderr: vec![], errors, result: None })
            }
        }
    })
}

#[test]
fn test_control_flow_conditional() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("control_flow").join("conditional"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Conditional tests should pass");
}

#[test]
fn test_control_flow_loop() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("control_flow").join("loop"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Loop tests should pass");
}

#[test]
fn test_control_flow_match() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("control_flow").join("match"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Match tests should pass");
}

#[test]
fn test_control_flow_exception() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("control_flow").join("exception"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Exception tests should pass");
}

#[test]
fn test_control_flow_keyword() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("control_flow").join("keyword"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Keyword tests should pass");
}

#[test]
fn test_oop_class() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("class"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Class tests should pass");
}

#[test]
fn test_oop_inheritance() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("inheritance"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Inheritance tests should pass");
}

#[test]
fn test_oop_trait() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("trait"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Trait tests should pass");
}

#[test]
fn test_oop_property() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("property"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Property tests should pass");
}

#[test]
fn test_oop_value_type() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("value_type"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Value type tests should pass");
}

#[test]
fn test_oop_special_class() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("special_class"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Special class tests should pass");
}

#[test]
fn test_oop_widget() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester = InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("widget"), create_executor())
        .with_extension("valkyrie")
        .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Widget tests should pass");
}

#[test]
fn test_oop_with_expression() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("with_expression"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("With expression tests should pass");
}

#[test]
fn test_oop_escape_analysis() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("escape_analysis"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Escape analysis tests should pass");
}

#[test]
fn test_oop_associated_type() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("associated_type"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Associated type tests should pass");
}

#[test]
fn test_oop_renaming_inheritance() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("renaming_inheritance"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Renaming inheritance tests should pass");
}

#[test]
fn test_oop_witness_serde() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tester =
        InterpreterTester::new(here.join("tests").join("fixtures").join("oop").join("witness_serde"), create_executor())
            .with_extension("valkyrie")
            .with_timeout(std::time::Duration::from_secs(10));

    tester.run_tests().expect("Witness serde tests should pass");
}
