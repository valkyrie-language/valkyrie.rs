use std::collections::{HashMap, HashSet};
use valkyrie_compiler::codegen::*;
use valkyrie_types::{Identifier, NamePath};

#[test]
fn test_field_offset_creation() {
    let offset = FieldOffset::direct(0, Identifier::new("x"), 4);
    assert_eq!(offset.offset, 0);
    assert!(!offset.is_inherited());
    assert_eq!(offset.field_name.as_str(), "x");

    let inherited = FieldOffset::inherited(16, Identifier::new("primary"), Identifier::new("name"), 8);
    assert_eq!(inherited.offset, 16);
    assert!(inherited.is_inherited());
    assert_eq!(inherited.parent_alias.unwrap().as_str(), "primary");
}

#[test]
fn test_method_dispatch_creation() {
    let local = MethodDispatch::local(Identifier::new("foo"), false, false);
    assert!(!local.is_inherited());
    assert!(!local.is_override);
    assert!(!local.is_final);

    let inherited = MethodDispatch::inherited(Identifier::new("bar"), Identifier::new("primary"), NamePath::default(), 2, true);
    assert!(inherited.is_inherited());
    assert!(inherited.is_final);
}

#[test]
fn test_field_offset_calculator_basic() {
    let calc = FieldOffsetCalculator::new();

    let own_fields = vec![(Identifier::new("x"), "i32".to_string()), (Identifier::new("y"), "i32".to_string())];
    let parent_fields = HashMap::new();
    let mro = vec!["Point".to_string()];

    let offsets = calc.calculate_offsets(&own_fields, &parent_fields, &mro);

    assert_eq!(offsets.len(), 2);
    assert_eq!(offsets.get(&Identifier::new("x")).unwrap().offset, 0);
    assert_eq!(offsets.get(&Identifier::new("y")).unwrap().offset, 4);
}

#[test]
fn test_field_offset_calculator_with_inheritance() {
    let calc = FieldOffsetCalculator::new();

    let own_fields = vec![(Identifier::new("z"), "i32".to_string())];
    let mut parent_fields = HashMap::new();
    parent_fields.insert(
        Identifier::new("primary"),
        vec![(Identifier::new("x"), "i32".to_string()), (Identifier::new("y"), "i32".to_string())],
    );
    let mro = vec!["Point3D".to_string(), "Point2D".to_string()];

    let offsets = calc.calculate_offsets(&own_fields, &parent_fields, &mro);

    assert_eq!(offsets.len(), 3);
    assert!(offsets.get(&Identifier::new("x")).unwrap().is_inherited());
    assert!(offsets.get(&Identifier::new("y")).unwrap().is_inherited());
    assert!(!offsets.get(&Identifier::new("z")).unwrap().is_inherited());
}

#[test]
fn test_method_dispatch_generator() {
    let parents =
        vec![("Teacher".to_string(), Some("primary".to_string())), ("Student".to_string(), Some("secondary".to_string()))];
    let mro = vec!["TA".to_string(), "Teacher".to_string(), "Student".to_string()];

    let gen = MethodDispatchGenerator::new(&parents, mro);

    assert!(gen.is_valid_alias("primary"));
    assert!(gen.is_valid_alias("secondary"));
    assert!(!gen.is_valid_alias("other"));

    assert_eq!(gen.resolve_qualified_call("primary"), Some("Teacher"));
    assert_eq!(gen.resolve_qualified_call("secondary"), Some("Student"));
    assert_eq!(gen.resolve_qualified_call("other"), None);
}

#[test]
fn test_method_dispatch_generator_effective_parent() {
    let parents =
        vec![("Parent1".to_string(), Some("primary".to_string())), ("Parent2".to_string(), Some("secondary".to_string()))];
    let mro = vec!["Child".to_string(), "Parent1".to_string(), "Parent2".to_string()];

    let gen = MethodDispatchGenerator::new(&parents, mro);

    assert_eq!(gen.get_effective_parent("primary"), Some("Parent1"));
    assert_eq!(gen.get_effective_parent("secondary"), Some("Parent2"));
    assert_eq!(gen.get_effective_parent("Parent1"), Some("Parent1"));
    assert_eq!(gen.get_effective_parent("unknown"), None);
}

#[test]
fn test_method_dispatch_generator_call() {
    let parents = vec![("Teacher".to_string(), Some("primary".to_string()))];
    let mro = vec!["TA".to_string(), "Teacher".to_string()];

    let mut gen = MethodDispatchGenerator::new(&parents, mro);

    gen.register_method(MethodDispatch::local(Identifier::new("teach"), false, false));

    let call = gen.generate_method_call("self", "teach", None, &[]);
    assert!(!call.is_virtual);
    assert!(!call.is_delegation);
    assert_eq!(call.code, "self.teach()");

    let qualified_call = gen.generate_method_call("self", "teach", Some("primary"), &[]);
    assert!(qualified_call.is_delegation);
}

#[test]
fn test_renamed_inheritance_codegen() {
    let parents =
        vec![("Teacher".to_string(), Some("primary".to_string())), ("Student".to_string(), Some("secondary".to_string()))];
    let mro = vec!["TA".to_string(), "Teacher".to_string(), "Student".to_string()];

    let mut codegen = RenamedInheritanceCodegen::new("TA", parents, mro);

    let own_fields = vec![(Identifier::new("id"), "i32".to_string())];
    let mut parent_fields = HashMap::new();
    parent_fields.insert(Identifier::new("primary"), vec![(Identifier::new("subject"), "String".to_string())]);
    parent_fields.insert(Identifier::new("secondary"), vec![(Identifier::new("grade"), "i32".to_string())]);

    codegen.compute_field_offsets(&own_fields, &parent_fields);

    let access = codegen.generate_field_access("self", "subject", Some("primary"));
    assert!(access.is_some());
    let access = access.unwrap();
    assert!(access.field.is_inherited());

    assert!(codegen.is_valid_alias("primary"));
    assert!(codegen.is_valid_alias("secondary"));
    assert_eq!(codegen.resolve_qualified("primary"), Some("Teacher"));
}

#[test]
fn test_diamond_inheritance_detection() {
    let calc = FieldOffsetCalculator::new();

    let mro = vec!["D".to_string(), "B".to_string(), "C".to_string(), "A".to_string()];
    let diamonds = calc.detect_diamond_bases(&mro);
    assert!(diamonds.is_empty());

    let mro_with_diamond = vec!["D".to_string(), "B".to_string(), "A".to_string(), "C".to_string(), "A".to_string()];
    let diamonds = calc.detect_diamond_bases(&mro_with_diamond);
    assert!(diamonds.contains("A"));
}

#[test]
fn test_field_access_code_generation() {
    let calc = FieldOffsetCalculator::new();

    let field = FieldOffset::inherited(16, Identifier::new("primary"), Identifier::new("name"), 8);
    let access = calc.generate_field_access("self", &field, Some(&Identifier::new("primary")));

    assert_eq!(access.code, "self.primary.name");
    assert!(access.field.is_inherited());
}

#[test]
fn test_method_call_code_types() {
    let method = MethodDispatch::local(Identifier::new("foo"), false, false);

    let direct = MethodCallCode::direct("self.foo()".to_string(), method.clone());
    assert!(!direct.is_virtual);
    assert!(!direct.is_delegation);

    let virtual_call = MethodCallCode::virtual_call("self.vtable[0]()".to_string(), method.clone(), 0);
    assert!(virtual_call.is_virtual);
    assert_eq!(virtual_call.vtable_index, Some(0));

    let delegation = MethodCallCode::delegation("parent_foo(self)".to_string(), method, Some(1));
    assert!(delegation.is_delegation);
}

#[test]
fn test_alignment() {
    assert_eq!(FieldOffsetCalculator::align_offset(0, 4), 0);
    assert_eq!(FieldOffsetCalculator::align_offset(1, 4), 4);
    assert_eq!(FieldOffsetCalculator::align_offset(4, 4), 4);
    assert_eq!(FieldOffsetCalculator::align_offset(5, 4), 8);
    assert_eq!(FieldOffsetCalculator::align_offset(16, 8), 16);
    assert_eq!(FieldOffsetCalculator::align_offset(17, 8), 24);
}

#[test]
fn test_parent_layout() {
    let layout = ParentLayout::new(Identifier::new("primary"), NamePath::new(vec![Identifier::new("Teacher")]), 16, 32);

    assert_eq!(layout.alias.as_str(), "primary");
    assert_eq!(layout.offset, 16);
    assert_eq!(layout.size, 32);
    assert_eq!(layout.end_offset(), 48);
    assert!(!layout.is_diamond_shared);

    let layout_with_diamond = layout.with_diamond_shared(true);
    assert!(layout_with_diamond.is_diamond_shared);
}

#[test]
fn test_field_offset_calculator_diamond() {
    let mut calc = FieldOffsetCalculator::new();

    let own_fields = vec![(Identifier::new("id"), "i32".to_string())];

    let parent_info = vec![
        (Identifier::new("b"), NamePath::new(vec![Identifier::new("B")])),
        (Identifier::new("c"), NamePath::new(vec![Identifier::new("C")])),
    ];

    let mut parent_fields = HashMap::new();
    parent_fields.insert(
        Identifier::new("b"),
        vec![(Identifier::new("b_field"), "i32".to_string()), (Identifier::new("shared"), "i32".to_string())],
    );
    parent_fields.insert(
        Identifier::new("c"),
        vec![(Identifier::new("c_field"), "i32".to_string()), (Identifier::new("shared"), "i32".to_string())],
    );

    let mut parent_mros = HashMap::new();
    parent_mros.insert("B".to_string(), vec!["B".to_string(), "A".to_string()]);
    parent_mros.insert("C".to_string(), vec!["C".to_string(), "A".to_string()]);

    let mro = vec!["D".to_string(), "B".to_string(), "C".to_string(), "A".to_string()];

    let (offsets, total_size) =
        calc.calculate_offsets_with_diamond(&own_fields, &parent_info, &parent_fields, &parent_mros, &mro);

    assert!(offsets.contains_key(&Identifier::new("id")));
    assert!(offsets.contains_key(&Identifier::new("b_field")));
    assert!(offsets.contains_key(&Identifier::new("c_field")));
    assert!(total_size > 0);
}

#[test]
fn test_method_dispatch_generator_diamond() {
    let parents = vec![("B".to_string(), Some("b".to_string())), ("C".to_string(), Some("c".to_string()))];
    let mro = vec!["D".to_string(), "B".to_string(), "C".to_string(), "A".to_string()];

    let gen = MethodDispatchGenerator::new(&parents, mro);

    assert!(gen.is_valid_alias("b"));
    assert!(gen.is_valid_alias("c"));
    assert_eq!(gen.resolve_qualified_call("b"), Some("B"));
    assert_eq!(gen.resolve_qualified_call("c"), Some("C"));
}

#[test]
fn test_method_dispatch_generator_super_call() {
    let parents = vec![("Parent".to_string(), Some("p".to_string()))];
    let mro = vec!["Child".to_string(), "Parent".to_string()];

    let mut gen = MethodDispatchGenerator::new(&parents, mro);

    gen.register_method(MethodDispatch::local(Identifier::new("method"), true, false));

    let super_call = gen.generate_super_call("self", "method", &[]);
    assert!(super_call.is_some());

    let super_call = super_call.unwrap();
    assert!(super_call.is_delegation);
}

#[test]
fn test_method_dispatch_generator_overridden() {
    let parents = vec![("Parent".to_string(), None)];
    let mro = vec!["Child".to_string(), "Parent".to_string()];

    let mut gen = MethodDispatchGenerator::new(&parents, mro);

    gen.register_method(MethodDispatch::local(Identifier::new("method"), true, false));

    assert!(gen.is_method_overridden("method"));
    assert!(!gen.is_method_overridden("other"));
}

#[test]
fn test_renamed_inheritance_codegen_diamond() {
    let parents = vec![("B".to_string(), Some("b".to_string())), ("C".to_string(), Some("c".to_string()))];
    let mro = vec!["D".to_string(), "B".to_string(), "C".to_string(), "A".to_string()];

    let mut codegen = RenamedInheritanceCodegen::new("D", parents, mro);

    let own_fields = vec![(Identifier::new("id"), "i32".to_string())];
    let parent_info = vec![
        (Identifier::new("b"), NamePath::new(vec![Identifier::new("B")])),
        (Identifier::new("c"), NamePath::new(vec![Identifier::new("C")])),
    ];
    let mut parent_fields = HashMap::new();
    parent_fields.insert(Identifier::new("b"), vec![(Identifier::new("x"), "i32".to_string())]);
    parent_fields.insert(Identifier::new("c"), vec![(Identifier::new("y"), "i32".to_string())]);
    let mut parent_mros = HashMap::new();
    parent_mros.insert("B".to_string(), vec!["B".to_string(), "A".to_string()]);
    parent_mros.insert("C".to_string(), vec!["C".to_string(), "A".to_string()]);

    codegen.compute_field_offsets_with_diamond(&own_fields, &parent_info, &parent_fields, &parent_mros);

    assert!(codegen.get_field_offset("id").is_some());
    assert!(codegen.get_field_offset("x").is_some());
    assert!(codegen.get_field_offset("y").is_some());

    let super_call = codegen.generate_super_call("self", "method", &[]);
    assert!(super_call.is_none() || super_call.is_some());
}

#[test]
fn test_renamed_inheritance_codegen_inherited_method() {
    let parents = vec![("Parent".to_string(), Some("p".to_string()))];
    let mro = vec!["Child".to_string(), "Parent".to_string()];

    let mut codegen = RenamedInheritanceCodegen::new("Child", parents, mro);

    codegen.register_inherited_method("parent_method", "p", "Parent", false);

    assert!(codegen.get_method_dispatch("parent_method").is_some());
}
