use valkyrie_compiler::type_checker::*;
use valkyrie_types::{
    hir::{
        HirDocumentation, HirFunction, HirGeneric, HirIdentifier, HirModule, HirParam, HirType, HirVisibility, HirWidget, HirWidgetLifecycle,
    },
    Identifier, NamePath, SourceSpan,
};

fn create_test_widget(name: &str, methods: Vec<HirFunction>) -> HirWidget {
    HirWidget {
        name: Identifier::new(name),
        doc: HirDocumentation::default(),
        generics: Vec::<HirGeneric>::new(),
        fields: vec![],
        methods,
        visibility: HirVisibility::public(),
        state_fields: vec![],
        initial_state: vec![],
        lifecycle: HirWidgetLifecycle::default(),
    }
}

fn create_render_method(return_type: HirType) -> HirFunction {
    HirFunction {
        name: Identifier::new("render"),
        doc: HirDocumentation::default(),
        annotations: vec![],
        generics: vec![],
        params: vec![HirParam {
            name: HirIdentifier {
                name: Identifier::new("self"),
                shadow_index: 0,
                span: SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
            },
            ty: HirType::Named(Identifier::new("Self")),
        }],
        return_type,
        body: valkyrie_types::hir::HirBlock {
            statements: vec![],
            expr: None,
            span: SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
        },
        span: SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
        visibility: HirVisibility::public(),
        is_abstract: false,
        is_final: false,
    }
}

fn create_event_handler_method(name: &str) -> HirFunction {
    HirFunction {
        name: Identifier::new(name),
        doc: HirDocumentation::default(),
        annotations: vec![],
        generics: vec![],
        params: vec![],
        return_type: HirType::Unit,
        body: valkyrie_types::hir::HirBlock {
            statements: vec![],
            expr: None,
            span: SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
        },
        span: SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
        visibility: HirVisibility::public(),
        is_abstract: false,
        is_final: false,
    }
}

fn create_test_module(widgets: Vec<HirWidget>) -> HirModule {
    HirModule {
        name: NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets,
        singletons: vec![],
        statements: vec![],
    }
}

#[test]
fn test_widget_with_valid_render_method() {
    let mut checker = WidgetChecker::new();
    let widget = create_test_widget("MyWidget", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);
    let module = create_test_module(vec![widget]);

    let errors = checker.check_module(&module);
    let render_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, WidgetErrorKind::MissingRenderMethod { .. })).collect();
    assert!(render_errors.is_empty());
}

#[test]
fn test_widget_missing_render_method() {
    let mut checker = WidgetChecker::new();
    let widget = create_test_widget("MyWidget", vec![]);
    let module = create_test_module(vec![widget]);

    let errors = checker.check_module(&module);
    let missing_render_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, WidgetErrorKind::MissingRenderMethod { .. })).collect();
    assert_eq!(missing_render_errors.len(), 1);
}

#[test]
fn test_widget_invalid_render_return_type() {
    let mut checker = WidgetChecker::new();
    let widget = create_test_widget("MyWidget", vec![create_render_method(HirType::Integer64)]);
    let module = create_test_module(vec![widget]);

    let errors = checker.check_module(&module);
    let return_type_errors: Vec<_> = errors.iter().filter(|e| matches!(e.kind, WidgetErrorKind::InvalidRenderReturnType { .. })).collect();
    assert_eq!(return_type_errors.len(), 1);
}

#[test]
fn test_widget_error_display() {
    let err = WidgetError::missing_render_method(Identifier::new("MyWidget"), None);
    assert!(err.to_string().contains("MyWidget"));
    assert!(err.to_string().contains("render"));

    let err = WidgetError::invalid_render_return_type(Identifier::new("MyWidget"), HirType::Integer64, None);
    assert!(err.to_string().contains("Element"));

    let err = WidgetError::invalid_state_update(Identifier::new("MyWidget"), Identifier::new("_count"), None);
    assert!(err.to_string().contains("_count"));
}

#[test]
fn test_is_element_type() {
    let checker = WidgetChecker::new();

    assert!(checker.is_element_type(&HirType::Named(Identifier::new("Element"))));
    assert!(!checker.is_element_type(&HirType::Named(Identifier::new("String"))));
    assert!(!checker.is_element_type(&HirType::Integer64));
}

#[test]
fn test_is_event_handler_method() {
    let checker = WidgetChecker::new();

    assert!(checker.is_event_handler_method("onClick"));
    assert!(checker.is_event_handler_method("on_click"));
    assert!(checker.is_event_handler_method("onChange"));
    assert!(!checker.is_event_handler_method("render"));
    assert!(!checker.is_event_handler_method("helper"));
}

#[test]
fn test_is_lifecycle_method() {
    let checker = WidgetChecker::new();

    assert!(checker.is_lifecycle_method("on_mount"));
    assert!(checker.is_lifecycle_method("on_unmount"));
    assert!(checker.is_lifecycle_method("on_update"));
    assert!(!checker.is_lifecycle_method("render"));
    assert!(!checker.is_lifecycle_method("onClick"));
}

#[test]
fn test_is_state_field() {
    let checker = WidgetChecker::new();

    assert!(checker.is_state_field(&Identifier::new("_count")));
    assert!(checker.is_state_field(&Identifier::new("state_name")));
    assert!(!checker.is_state_field(&Identifier::new("name")));
    assert!(!checker.is_state_field(&Identifier::new("count")));
}

#[test]
fn test_is_valid_state_update_context() {
    let mut checker = WidgetChecker::new();
    assert!(!checker.is_valid_state_update_context());

    checker.set_in_event_handler(true);
    assert!(checker.is_valid_state_update_context());

    checker.set_in_event_handler(false);
    checker.set_in_lifecycle_method(true);
    assert!(checker.is_valid_state_update_context());
}

#[test]
fn test_get_widget_names() {
    let mut checker = WidgetChecker::new();
    let widget1 = create_test_widget("Widget1", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);
    let widget2 = create_test_widget("Widget2", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);
    let module = create_test_module(vec![widget1, widget2]);

    checker.check_module(&module);
    let names = checker.get_widget_names();
    assert_eq!(names.len(), 2);
}

#[test]
fn test_checker_clear() {
    let mut checker = WidgetChecker::new();
    let widget = create_test_widget("MyWidget", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);
    let module = create_test_module(vec![widget]);

    checker.check_module(&module);
    assert!(!checker.widgets().is_empty());

    checker.clear();
    assert!(checker.widgets().is_empty());
    assert!(checker.errors().is_empty());
    assert!(checker.current_widget().is_none());
}

#[test]
fn test_collect_widgets_from_submodules() {
    let mut checker = WidgetChecker::new();
    let widget1 = create_test_widget("Widget1", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);
    let widget2 = create_test_widget("Widget2", vec![create_render_method(HirType::Named(Identifier::new("Element")))]);

    let submodule = HirModule {
        name: NamePath::new(vec![Identifier::new("submodule")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![],
        functions: vec![],
        structs: vec![],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![widget2],
        singletons: vec![],
        statements: vec![],
    };

    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("test")]),
        doc: HirDocumentation::default(),
        imports: vec![],
        submodules: vec![submodule],
        functions: vec![],
        structs: vec![],
        enums: vec![],
        flags: vec![],
        traits: vec![],
        impls: vec![],
        type_functions: vec![],
        type_families: vec![],
        widgets: vec![widget1],
        singletons: vec![],
        statements: vec![],
    };

    checker.collect_widgets(&module);
    assert_eq!(checker.widgets().len(), 2);
}

#[test]
fn test_find_render_method() {
    let checker = WidgetChecker::new();
    let widget = create_test_widget(
        "MyWidget",
        vec![create_render_method(HirType::Named(Identifier::new("Element"))), create_event_handler_method("onClick")],
    );

    let render = checker.find_render_method(&widget);
    assert!(render.is_some());
    assert_eq!(render.unwrap().name.as_str(), "render");
}

#[test]
fn test_find_render_method_not_found() {
    let checker = WidgetChecker::new();
    let widget = create_test_widget("MyWidget", vec![create_event_handler_method("onClick")]);

    let render = checker.find_render_method(&widget);
    assert!(render.is_none());
}
