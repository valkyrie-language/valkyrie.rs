use valkyrie_compiler::hir::{
    row::{RowMethodSignature, RowRequirement},
    type_relation::{ParameterMatchResult, TypeRelationContext},
};
use valkyrie_types::{
    hir::{
        HirBlock, HirDocumentation, HirFunction, HirIdentifier, HirImpl, HirModule, HirParam, HirStruct, HirTrait, HirVisibility,
        RowMethodType, RowType, ValkyrieType,
    },
    Identifier, NamePath, SourceID, SourceSpan,
};

#[test]
fn type_relation_reports_nominal_exact_and_subtype() {
    let animal = class("Animal");
    let dog = subclass("Dog", "Animal");
    let module = module_with_traits(vec![], vec![animal, dog], vec![]);
    let relations = TypeRelationContext::from_module(&module);

    assert_eq!(
        relations.match_parameter(&ValkyrieType::Named(Identifier::new("Animal")), &ValkyrieType::Named(Identifier::new("Animal"))),
        ParameterMatchResult::NominalExact
    );
    assert_eq!(
        relations.match_parameter(&ValkyrieType::Named(Identifier::new("Dog")), &ValkyrieType::Named(Identifier::new("Animal"))),
        ParameterMatchResult::NominalSubtype { distance: 1 }
    );
}

#[test]
fn type_relation_reports_named_trait_witness() {
    let writer_trait = trait_with_methods("Writer", vec![method("write", vec![ValkyrieType::Utf8], ValkyrieType::Unit)]);
    let console = struct_with_methods("Console", vec![method("write", vec![ValkyrieType::Utf8], ValkyrieType::Unit)]);
    let module = module_with_traits(vec![writer_trait], vec![console], vec![]);
    let relations = TypeRelationContext::from_module(&module);

    let result = relations.match_named_trait_parameter(&Identifier::new("Console"), &Identifier::new("Writer"));
    assert!(matches!(result, Some(ParameterMatchResult::Trait { .. })));
}

#[test]
fn type_relation_reports_row_satisfaction() {
    let candidate = struct_with_methods("Clock", vec![method("now", vec![], ValkyrieType::Integer64 { signed: true })]);
    let module = module_with_traits(vec![], vec![candidate], vec![]);
    let relations = TypeRelationContext::from_module(&module);
    let requirement = RowRequirement::from_methods(vec![RowMethodSignature::new("now", vec![], ValkyrieType::Integer64 { signed: true })]);

    assert_eq!(relations.match_row_requirement(&Identifier::new("Clock"), &requirement), ParameterMatchResult::Row);
}

#[test]
fn type_relation_matches_row_parameter_from_valkyrie_type() {
    let candidate = struct_with_methods("Clock", vec![method("now", vec![], ValkyrieType::Integer64 { signed: true })]);
    let module = module_with_traits(vec![], vec![candidate], vec![]);
    let relations = TypeRelationContext::from_module(&module);
    let expected = ValkyrieType::Row(RowType {
        methods: vec![RowMethodType { name: Identifier::new("now"), params: vec![], return_type: ValkyrieType::Integer64 { signed: true } }],
    });

    assert_eq!(relations.match_parameter(&ValkyrieType::Named(Identifier::new("Clock")), &expected), ParameterMatchResult::Row);
}

#[test]
fn type_relation_reports_no_match_with_seed() {
    let animal = class("Animal");
    let robot = class("Robot");
    let module = module_with_traits(vec![], vec![animal, robot], vec![]);
    let relations = TypeRelationContext::from_module(&module);

    let result = relations.match_parameter(&ValkyrieType::Named(Identifier::new("Robot")), &ValkyrieType::Named(Identifier::new("Animal")));
    assert!(matches!(result, ParameterMatchResult::NoMatch { .. }));
}

fn trait_with_methods(name: &str, methods: Vec<HirFunction>) -> HirTrait {
    HirTrait {
        name: Identifier::new(name),
        doc: HirDocumentation::default(),
        generics: vec![],
        methods,
        associated_types: vec![],
        associated_constants: vec![],
        super_traits: vec![],
        default_methods: vec![],
        visibility: HirVisibility::public(),
    }
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
    let mut item = class(name);
    item.parents = vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new(parent)]))];
    item
}

fn struct_with_methods(name: &str, methods: Vec<HirFunction>) -> HirStruct {
    let mut item = class(name);
    item.methods = methods;
    item
}

fn module_with_traits(traits: Vec<HirTrait>, structs: Vec<HirStruct>, impls: Vec<HirImpl>) -> HirModule {
    HirModule {
        name: NamePath::new(vec![Identifier::new("spec")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs,
        enums: vec![],
        flags: vec![],
        traits,
        impls,
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![],
        singletons: vec![],
        statements: vec![],
    }
}

fn method(name: &str, params: Vec<ValkyrieType>, return_type: ValkyrieType) -> HirFunction {
    HirFunction {
        name: Identifier::new(name),
        doc: HirDocumentation::default(),
        annotations: vec![],
        generics: vec![],
        params: params
            .into_iter()
            .enumerate()
            .map(|(index, ty)| HirParam {
                name: HirIdentifier { name: Identifier::new(&format!("arg{index}")), shadow_index: 0, span: span() },
                ty,
            })
            .collect(),
        return_type,
        body: HirBlock { statements: vec![], expr: None, span: span() },
        span: span(),
        visibility: HirVisibility::public(),
        is_abstract: false,
        is_final: false,
    }
}

fn span() -> SourceSpan {
    SourceSpan::new(SourceID::default(), 0, 0)
}
