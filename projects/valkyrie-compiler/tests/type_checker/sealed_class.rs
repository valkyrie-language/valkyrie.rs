use valkyrie_compiler::type_checker::*;

use valkyrie_types::{hir::HirStruct, Identifier};

fn create_test_identifier(name: &str) -> Identifier {
    Identifier::new(name)
}

#[test]
fn test_sealed_class_registry() {
    let mut registry = SealedClassRegistry::new();

    let sealed_name = create_test_identifier("Shape");
    let sealed_class = HirStruct { name: sealed_name.clone(), is_sealed: true, ..Default::default() };

    registry.register_sealed_class(&sealed_class);

    assert!(registry.is_sealed_class(&sealed_name));

    let subclass1 = create_test_identifier("Circle");
    let subclass2 = create_test_identifier("Rectangle");

    registry.register_subclass(&sealed_name, &subclass1).unwrap();
    registry.register_subclass(&sealed_name, &subclass2).unwrap();

    let subclasses = registry.get_permitted_subclasses(&sealed_name);
    assert_eq!(subclasses.len(), 2);
}

#[test]
fn test_exhaustiveness_check() {
    let mut registry = SealedClassRegistry::new();

    let sealed_name = create_test_identifier("Shape");
    let sealed_class = HirStruct { name: sealed_name.clone(), is_sealed: true, ..Default::default() };

    registry.register_sealed_class(&sealed_class);

    let subclass1 = create_test_identifier("Circle");
    let subclass2 = create_test_identifier("Rectangle");

    registry.register_subclass(&sealed_name, &subclass1).unwrap();
    registry.register_subclass(&sealed_name, &subclass2).unwrap();

    let checker = ExhaustivenessChecker::new(registry);

    let result = checker.check_exhaustiveness(&sealed_name, &[subclass1.clone(), subclass2.clone()]);
    assert!(result.is_ok());

    let result = checker.check_exhaustiveness(&sealed_name, &[subclass1]);
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.kind, SealedClassErrorKind::NonExhaustiveMatch);
        assert!(err.message.contains("Rectangle"));
    }
}

#[test]
fn test_wildcard_exhaustiveness() {
    let registry = SealedClassRegistry::new();
    let checker = ExhaustivenessChecker::new(registry);

    let type_name = create_test_identifier("SomeType");

    assert!(checker.is_wildcard_exhaustive(&type_name, true));
    assert!(!checker.is_wildcard_exhaustive(&type_name, false));
}
