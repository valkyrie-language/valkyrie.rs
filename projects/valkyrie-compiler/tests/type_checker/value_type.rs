use valkyrie_compiler::type_checker::*;
use valkyrie_types::{
    hir::{HirDocumentation, HirField, HirModule, HirStruct, HirType, HirVisibility},
    Identifier,
};

fn create_value_type_struct(name: &str, parents: Vec<valkyrie_types::hir::HirParent>) -> HirStruct {
    HirStruct {
        name: Identifier::new(name),
        namespace: vec![],
        doc: HirDocumentation::default(),
        generics: vec![],
        parents,
        fields: vec![HirField {
            name: Identifier::new("x"),
            doc: HirDocumentation::default(),
            ty: HirType::Integer64,
            visibility: HirVisibility::public(),
            is_readonly: false,
        }],
        methods: vec![],
        properties: vec![],
        visibility: HirVisibility::public(),
        is_value_type: true,
        is_abstract: false,
        is_sealed: false,
        is_final: false,
        is_open: false,
        abstract_methods: vec![],
        abstract_properties: vec![],
        derives: vec![],
    }
}

fn create_reference_type_struct(name: &str, parents: Vec<valkyrie_types::hir::HirParent>) -> HirStruct {
    HirStruct {
        name: Identifier::new(name),
        namespace: vec![],
        doc: HirDocumentation::default(),
        generics: vec![],
        parents,
        fields: vec![HirField {
            name: Identifier::new("x"),
            doc: HirDocumentation::default(),
            ty: HirType::Integer64,
            visibility: HirVisibility::public(),
            is_readonly: false,
        }],
        methods: vec![],
        properties: vec![],
        visibility: HirVisibility::public(),
        is_value_type: false,
        is_abstract: false,
        is_sealed: false,
        is_final: false,
        is_open: false,
        abstract_methods: vec![],
        abstract_properties: vec![],
        derives: vec![],
    }
}

#[test]
fn test_value_type_no_inheritance() {
    let mut checker = ValueTypeChecker::new();
    let struct_def = create_value_type_struct("Point", vec![]);
    assert!(!struct_def.is_value_type && struct_def.parents.is_empty() || struct_def.is_value_type);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    let errors = checker.check_module(&module);
    let inheritance_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, ValueTypeErrorKind::ValueTypeInheritance { .. })).collect();
    assert!(inheritance_errors.is_empty());
}

#[test]
fn test_value_type_with_inheritance_error() {
    let mut checker = ValueTypeChecker::new();
    let parent = valkyrie_types::hir::HirParent {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("BaseClass")]),
        alias: None,
        generics: vec![],
        offset: None,
    };
    let struct_def = create_value_type_struct("Point", vec![parent]);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    let errors = checker.check_module(&module);
    let inheritance_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, ValueTypeErrorKind::ValueTypeInheritance { .. })).collect();
    assert_eq!(inheritance_errors.len(), 1);
}

#[test]
fn test_reference_type_with_inheritance_ok() {
    let mut checker = ValueTypeChecker::new();
    let parent = valkyrie_types::hir::HirParent {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("BaseClass")]),
        alias: None,
        generics: vec![],
        offset: None,
    };
    let struct_def = create_reference_type_struct("MyClass", vec![parent]);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    let errors = checker.check_module(&module);
    let inheritance_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, ValueTypeErrorKind::ValueTypeInheritance { .. })).collect();
    assert!(inheritance_errors.is_empty());
}

#[test]
fn test_value_type_error_display() {
    let err = ValueTypeError::value_type_inheritance(Identifier::new("Point"), Identifier::new("Base"), None);
    assert!(err.to_string().contains("Point"));
    assert!(err.to_string().contains("Base"));

    let err = ValueTypeError::value_type_field_mutation(Identifier::new("Point"), Identifier::new("x"), None);
    assert!(err.to_string().contains("Point"));
    assert!(err.to_string().contains("x"));
}

#[test]
fn test_copy_semantics_validator() {
    let mut validator = CopySemanticsValidator::new();
    let struct_def = create_value_type_struct("Point", vec![]);
    validator.register_value_type(&struct_def);

    let ty = HirType::Named(Identifier::new("Point"));
    assert_eq!(validator.validate_assignment(&ty), AssignmentSemantics::Copy);
    assert_eq!(validator.validate_parameter_passing(&ty), ParameterSemantics::Copy);
    assert_eq!(validator.validate_return(&ty), ReturnSemantics::Copy);

    let ref_ty = HirType::Named(Identifier::new("MyClass"));
    assert_eq!(validator.validate_assignment(&ref_ty), AssignmentSemantics::Reference);
}

#[test]
fn test_is_value_type() {
    let mut checker = ValueTypeChecker::new();
    let struct_def = create_value_type_struct("Point", vec![]);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    checker.check_module(&module);

    assert!(checker.is_value_type(&HirType::Named(Identifier::new("Point"))));
    assert!(!checker.is_value_type(&HirType::Named(Identifier::new("Unknown"))));
    assert!(!checker.is_value_type(&HirType::Integer64));
}

#[test]
fn test_get_value_type_names() {
    let mut checker = ValueTypeChecker::new();
    let struct_def1 = create_value_type_struct("Point", vec![]);
    let struct_def2 = create_value_type_struct("Vector", vec![]);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def1, struct_def2],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    checker.check_module(&module);
    let names = checker.get_value_type_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn test_checker_clear() {
    let mut checker = ValueTypeChecker::new();
    let struct_def = create_value_type_struct("Point", vec![]);

    let module = HirModule {
        name: valkyrie_types::NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![struct_def],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    };

    checker.check_module(&module);
    assert!(!checker.value_types().is_empty());

    checker.clear();
    assert!(checker.value_types().is_empty());
    assert!(checker.errors().is_empty());
}
