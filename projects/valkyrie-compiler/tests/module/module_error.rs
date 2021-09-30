use nyar_types::QualifiedName;
use std::path::PathBuf;
use valkyrie_compiler::module::*;

#[test]
fn test_qualified_name_display() {
    let name = QualifiedName::from("std::collections::HashMap");
    assert_eq!(format!("{}", name), "std::collections::HashMap");
}

#[test]
fn test_module_error_display() {
    let name = QualifiedName::from("missing::module");
    let error = ModuleError::NotFound { name, searched: vec![PathBuf::from("/path/a"), PathBuf::from("/path/b")] };
    let message = format!("{}", error);
    assert!(message.contains("missing::module"));
    assert!(message.contains("/path/a"));
}
