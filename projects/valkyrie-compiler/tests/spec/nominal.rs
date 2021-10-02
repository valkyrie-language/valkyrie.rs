use valkyrie_compiler::{
    hir::nominal::{
        lower_unite, matches_nominal_parameter, validate_unite_definition, NominalModuleError, NominalModuleView, UniteCoverageError,
        UniteDefinitionError, UniteLayout,
    },
    ValkyrieCompiler,
};
use valkyrie_types::{
    hir::{GenericType, HirDocumentation, HirEnum, HirField, HirKind, HirModule, HirStruct, HirVariant, HirVisibility, ValkyrieType},
    Identifier, NamePath, SourceID,
};

#[test]
fn class_parameters_use_nominal_matching() {
    let animal = class("Animal");
    let dog = subclass("Dog", "Animal");
    let corgi = subclass("Corgi", "Dog");
    let declared = vec![animal.clone(), dog.clone(), corgi.clone()];

    assert!(matches_nominal_parameter(&animal, &animal, &declared));
    assert!(matches_nominal_parameter(&dog, &animal, &declared));
    assert!(matches_nominal_parameter(&corgi, &animal, &declared));
    assert!(!matches_nominal_parameter(&animal, &dog, &declared));
}

#[test]
fn nominal_type_is_not_satisfied_by_shape_alone() {
    let animal = class("Animal");
    let robot_dog = class("RobotDog");
    let declared = vec![animal.clone(), robot_dog.clone()];

    assert!(!matches_nominal_parameter(&robot_dog, &animal, &declared));
}

#[test]
fn nominal_cycles_do_not_create_false_subtyping() {
    let mut alpha = class("Alpha");
    let mut beta = class("Beta");
    alpha.parents = vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new("Beta")]))];
    beta.parents = vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new("Alpha")]))];
    let gamma = class("Gamma");
    let declared = vec![alpha.clone(), beta.clone(), gamma.clone()];

    assert!(matches_nominal_parameter(&alpha, &beta, &declared));
    assert!(matches_nominal_parameter(&beta, &alpha, &declared));
    assert!(!matches_nominal_parameter(&alpha, &gamma, &declared));
    assert!(!matches_nominal_parameter(&gamma, &alpha, &declared));
}

#[test]
fn unite_defaults_to_abstract_base_with_closed_variants() {
    let option = option_unite();
    let lowered = lower_unite(&option, UniteLayout::Untagged);

    assert_eq!(lowered.base.name, Identifier::new("Option"));
    assert!(lowered.base.is_abstract);
    assert!(lowered.base.is_sealed);
    assert!(!lowered.base.is_open);
    assert_eq!(lowered.layout, UniteLayout::Untagged);

    assert_eq!(lowered.variant_names(), vec![Identifier::new("Some"), Identifier::new("None")]);
    assert!(lowered.variants.iter().all(|variant| variant.is_sealed));
    assert!(lowered.variants.iter().all(|variant| variant.is_final));
    assert_eq!(lowered.variants[0].parents[0].name, NamePath::new(vec![Identifier::new("Option")]));
    assert_eq!(lowered.variants[1].parents[0].name, NamePath::new(vec![Identifier::new("Option")]));
}

#[test]
fn unite_variants_preserve_base_generics_in_parent_edge() {
    let either = generic_unite();
    let lowered = lower_unite(&either, UniteLayout::Untagged);

    assert_eq!(lowered.base.generics, either.generics);
    assert_eq!(lowered.variants.len(), 2);
    assert_eq!(lowered.variants[0].generics, either.generics);
    assert_eq!(lowered.variants[1].generics, either.generics);

    let left_parent = &lowered.variants[0].parents[0];
    assert_eq!(left_parent.name, NamePath::new(vec![Identifier::new("Either")]));
    assert_eq!(
        left_parent.generics,
        vec![
            ValkyrieType::Generic(GenericType { name: Identifier::new("L"), kind: HirKind::Type, bounds: vec![] }),
            ValkyrieType::Generic(GenericType { name: Identifier::new("R"), kind: HirKind::Type, bounds: vec![] }),
        ]
    );
}

#[test]
fn gadt_variant_result_type_refines_parent_generics() {
    let expr = gadt_unite();
    let lowered = lower_unite(&expr, UniteLayout::Tagged);

    assert_eq!(lowered.variants.len(), 2);

    let literal = &lowered.variants[0];
    assert!(literal.generics.is_empty());
    assert_eq!(literal.parents[0].generics, vec![ValkyrieType::Float64]);

    let branch = &lowered.variants[1];
    assert_eq!(branch.generics, vec![GenericType { name: Identifier::new("T"), kind: HirKind::Type, bounds: vec![] }]);
    assert_eq!(
        branch.parents[0].generics,
        vec![ValkyrieType::Generic(GenericType { name: Identifier::new("T"), kind: HirKind::Type, bounds: vec![] })]
    );
}

#[test]
fn unite_exhaustiveness_is_independent_of_runtime_layout() {
    let option = option_unite();
    let tagged = lower_unite(&option, UniteLayout::Tagged);
    let untagged = lower_unite(&option, UniteLayout::Untagged);
    let complete = vec![Identifier::new("Some"), Identifier::new("None")];
    let incomplete = vec![Identifier::new("Some")];

    assert!(tagged.is_exhaustive_over(&complete));
    assert!(untagged.is_exhaustive_over(&complete));
    assert!(!tagged.is_exhaustive_over(&incomplete));
    assert!(!untagged.is_exhaustive_over(&incomplete));
    assert_eq!(tagged.variant_names(), untagged.variant_names());
}

#[test]
fn unite_coverage_rejects_missing_variants() {
    let option = option_unite();
    let lowered = lower_unite(&option, UniteLayout::Untagged);
    let error = lowered.check_exhaustiveness(&[Identifier::new("Some")]).unwrap_err();

    assert_eq!(error, UniteCoverageError::MissingVariants { names: vec![Identifier::new("None")] });
}

#[test]
fn unite_coverage_rejects_unknown_variants() {
    let option = option_unite();
    let lowered = lower_unite(&option, UniteLayout::Untagged);
    let error = lowered.check_exhaustiveness(&[Identifier::new("Some"), Identifier::new("None"), Identifier::new("Later")]).unwrap_err();

    assert_eq!(error, UniteCoverageError::UnknownVariants { names: vec![Identifier::new("Later")] });
}

#[test]
fn unite_coverage_rejects_duplicate_variants() {
    let option = option_unite();
    let lowered = lower_unite(&option, UniteLayout::Tagged);
    let error = lowered.check_exhaustiveness(&[Identifier::new("Some"), Identifier::new("Some"), Identifier::new("None")]).unwrap_err();

    assert_eq!(error, UniteCoverageError::DuplicateVariants { names: vec![Identifier::new("Some")] });
}

#[test]
fn unite_definition_rejects_empty_family() {
    let empty = HirEnum::new_unity(Identifier::new("Never"));

    let error = validate_unite_definition(&empty).unwrap_err();

    assert_eq!(error, UniteDefinitionError::EmptyVariants);
}

#[test]
fn unite_definition_rejects_duplicate_variant_names() {
    let mut duplicate = HirEnum::new_unity(Identifier::new("Bad"));
    duplicate.visibility = HirVisibility::public();
    duplicate.variants = vec![
        HirVariant { name: Identifier::new("Same"), doc: HirDocumentation::default(), fields: vec![], tuple_types: vec![], result_type: None },
        HirVariant { name: Identifier::new("Same"), doc: HirDocumentation::default(), fields: vec![], tuple_types: vec![], result_type: None },
    ];

    let error = validate_unite_definition(&duplicate).unwrap_err();

    assert_eq!(error, UniteDefinitionError::DuplicateVariants { names: vec![Identifier::new("Same")] });
}

#[test]
fn unite_definition_rejects_variant_result_type_outside_family() {
    let mut expr = gadt_unite();
    expr.variants[0].result_type = Some(ValkyrieType::Named(Identifier::new("Other")));

    let error = validate_unite_definition(&expr).unwrap_err();

    assert_eq!(error, UniteDefinitionError::InvalidVariantResultType { variant: Identifier::new("Literal") });
}

#[test]
fn unite_definition_rejects_variant_result_type_with_undeclared_generic() {
    let mut expr = gadt_unite();
    expr.variants[1].result_type = Some(ValkyrieType::Apply(
        Box::new(ValkyrieType::Named(Identifier::new("Expr"))),
        vec![ValkyrieType::Generic(GenericType { name: Identifier::new("U"), kind: HirKind::Type, bounds: vec![] })],
    ));

    let error = validate_unite_definition(&expr).unwrap_err();

    assert_eq!(error, UniteDefinitionError::InvalidVariantResultType { variant: Identifier::new("If") });
}

#[test]
fn unite_definition_rejects_bare_result_type_for_generic_family() {
    let mut expr = gadt_unite();
    expr.variants[1].result_type = Some(ValkyrieType::Named(Identifier::new("Expr")));

    let error = validate_unite_definition(&expr).unwrap_err();

    assert_eq!(error, UniteDefinitionError::InvalidVariantResultType { variant: Identifier::new("If") });
}

#[test]
fn nominal_module_view_resolves_unite_variants_as_real_named_types() {
    let view = NominalModuleView::from_module(&module_with_unite(option_unite()));

    assert!(view.matches_nominal_parameter(&Identifier::new("Some"), &Identifier::new("Option")).unwrap());
    assert!(view.matches_nominal_parameter(&Identifier::new("None"), &Identifier::new("Option")).unwrap());
}

#[test]
fn nominal_module_view_reads_real_hir_module_objects_from_compiler() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 31 });
    let module = compiler
        .compile_source(
            r#"class Animal {}
class Dog(Animal) {}
"#,
        )
        .unwrap();
    let view = NominalModuleView::from_module(&module);

    assert!(view.matches_nominal_parameter(&Identifier::new("Dog"), &Identifier::new("Animal")).unwrap());
}

#[test]
fn nominal_module_view_reads_real_unite_family_from_compiler() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 33 });
    let module = compiler
        .compile_source(
            r#"unite Option {
    Some {
        value: i64,
    }
    None
}
"#,
        )
        .unwrap();
    let view = NominalModuleView::from_module(&module);

    assert!(view.matches_nominal_parameter(&Identifier::new("Some"), &Identifier::new("Option")).unwrap());
    assert!(view.matches_nominal_parameter(&Identifier::new("None"), &Identifier::new("Option")).unwrap());
}

#[test]
fn nominal_module_view_surfaces_invalid_unite_definition() {
    let mut invalid = gadt_unite();
    invalid.variants[1].result_type = Some(ValkyrieType::Named(Identifier::new("Other")));
    let view = NominalModuleView::from_module(&module_with_unite(invalid));

    let error = view.lower_unite(&Identifier::new("Expr"), UniteLayout::Tagged).unwrap_err();

    assert_eq!(
        error,
        NominalModuleError::InvalidUnite {
            name: Identifier::new("Expr"),
            error: UniteDefinitionError::InvalidVariantResultType { variant: Identifier::new("If") },
        }
    );
}

fn class(name: &str) -> HirStruct {
    HirStruct {
        name: Identifier::new(name),
        namespace: vec![],
        doc: HirDocumentation::default(),
        generics: vec![],
        parents: vec![],
        fields: vec![],
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

fn subclass(name: &str, parent: &str) -> HirStruct {
    let mut class = class(name);
    class.parents = vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new(parent)]))];
    class
}

fn option_unite() -> HirEnum {
    let mut enum_def = HirEnum::new_unity(Identifier::new("Option"));
    enum_def.visibility = HirVisibility::public();
    enum_def.variants = vec![
        HirVariant {
            name: Identifier::new("Some"),
            doc: HirDocumentation::default(),
            fields: vec![HirField {
                name: Identifier::new("value"),
                doc: HirDocumentation::default(),
                ty: int32(),
                visibility: HirVisibility::public(),
                is_readonly: false,
            }],
            tuple_types: vec![],
            result_type: None,
        },
        HirVariant { name: Identifier::new("None"), doc: HirDocumentation::default(), fields: vec![], tuple_types: vec![], result_type: None },
    ];
    enum_def
}

fn module_with_unite(enum_def: HirEnum) -> HirModule {
    HirModule {
        name: NamePath::new(vec![Identifier::new("spec")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![],
        enums: vec![enum_def],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    }
}

fn generic_unite() -> HirEnum {
    let mut enum_def = HirEnum::new_unity(Identifier::new("Either"));
    enum_def.visibility = HirVisibility::public();
    enum_def.generics = vec![
        GenericType { name: Identifier::new("L"), kind: HirKind::Type, bounds: vec![] },
        GenericType { name: Identifier::new("R"), kind: HirKind::Type, bounds: vec![] },
    ];
    enum_def.variants = vec![
        HirVariant {
            name: Identifier::new("Left"),
            doc: HirDocumentation::default(),
            fields: vec![HirField {
                name: Identifier::new("value"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Generic(GenericType { name: Identifier::new("L"), kind: HirKind::Type, bounds: vec![] }),
                visibility: HirVisibility::public(),
                is_readonly: false,
            }],
            tuple_types: vec![],
            result_type: None,
        },
        HirVariant {
            name: Identifier::new("Right"),
            doc: HirDocumentation::default(),
            fields: vec![HirField {
                name: Identifier::new("value"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Generic(GenericType { name: Identifier::new("R"), kind: HirKind::Type, bounds: vec![] }),
                visibility: HirVisibility::public(),
                is_readonly: false,
            }],
            tuple_types: vec![],
            result_type: None,
        },
    ];
    enum_def
}

fn gadt_unite() -> HirEnum {
    let mut enum_def = HirEnum::new_unity(Identifier::new("Expr"));
    enum_def.visibility = HirVisibility::public();
    enum_def.generics = vec![GenericType { name: Identifier::new("T"), kind: HirKind::Type, bounds: vec![] }];
    enum_def.variants = vec![
        HirVariant {
            name: Identifier::new("Literal"),
            doc: HirDocumentation::default(),
            fields: vec![HirField {
                name: Identifier::new("value"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Float64,
                visibility: HirVisibility::public(),
                is_readonly: false,
            }],
            tuple_types: vec![],
            result_type: Some(ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Expr"))), vec![ValkyrieType::Float64])),
        },
        HirVariant {
            name: Identifier::new("If"),
            doc: HirDocumentation::default(),
            fields: vec![
                HirField {
                    name: Identifier::new("condition"),
                    doc: HirDocumentation::default(),
                    ty: ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Expr"))), vec![ValkyrieType::Boolean]),
                    visibility: HirVisibility::public(),
                    is_readonly: false,
                },
                HirField {
                    name: Identifier::new("then_branch"),
                    doc: HirDocumentation::default(),
                    ty: ValkyrieType::Apply(
                        Box::new(ValkyrieType::Named(Identifier::new("Expr"))),
                        vec![ValkyrieType::Generic(GenericType { name: Identifier::new("T"), kind: HirKind::Type, bounds: vec![] })],
                    ),
                    visibility: HirVisibility::public(),
                    is_readonly: false,
                },
            ],
            tuple_types: vec![],
            result_type: Some(ValkyrieType::Apply(
                Box::new(ValkyrieType::Named(Identifier::new("Expr"))),
                vec![ValkyrieType::Generic(GenericType { name: Identifier::new("T"), kind: HirKind::Type, bounds: vec![] })],
            )),
        },
    ];
    enum_def
}

fn int32() -> ValkyrieType {
    ValkyrieType::Integer32 { signed: true }
}
