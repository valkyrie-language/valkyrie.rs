#![allow(missing_docs)]

mod class_semantics;
mod control_flow_validation;
mod effect_typing;
mod imply_validation;

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use valkyrie_types::{
    hir::{AccessLevel, HirFunction, HirModule, HirProperty, HirStruct, HirWidget, ValkyrieType as HirType},
    Identifier, NamePath, SourceSpan,
};

pub use self::{
    class_semantics::{
        AbstractClassChecker, AbstractClassError, AbstractClassErrorKind, FinalClassChecker, FinalClassError, FinalClassErrorKind,
    },
    control_flow_validation::{InferenceTypeVar, TypeError, TypeInference},
    effect_typing::{
        AssociatedTypeConstraint, ConstraintChainNode, ConstraintError, ConstraintErrorKind, ConstraintPropagator, ConstraintReport,
        ConstraintSolver, FixSuggestion, Lifetime, LifetimeConstraint, LifetimeConstraintKind, MultiTraitBound, TraitBoundChecker, TraitImpl,
        TypeConstraint, TypeVar, WhereBound, WhereClause,
    },
    imply_validation::{ImplyChecker, ImplyError, ImplyErrorKind},
};

fn display_path(path: &NamePath) -> String {
    path.parts().iter().map(|part| part.to_string()).collect::<Vec<_>>().join("::")
}

fn last_name(path: &NamePath) -> Option<Identifier> {
    path.parts().last().cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessContext {
    pub current_module: NamePath,
    pub current_class: Option<Identifier>,
    pub current_method: Option<Identifier>,
}

impl AccessContext {
    pub fn new(current_module: NamePath) -> Self {
        Self { current_module, current_class: None, current_method: None }
    }

    pub fn in_class(current_module: NamePath, current_class: Identifier) -> Self {
        Self { current_module, current_class: Some(current_class), current_method: None }
    }

    pub fn in_method(current_module: NamePath, current_class: Identifier, current_method: Identifier) -> Self {
        Self { current_module, current_class: Some(current_class), current_method: Some(current_method) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccessControlErrorKind {
    PrivateMemberAccess { owner: Identifier, member: Identifier, accessor: Option<Identifier> },
    ProtectedMemberAccess { owner: Identifier, member: Identifier, accessor: Identifier },
    InternalMemberAccess { member: Identifier, declared_module: NamePath, current_module: NamePath },
    ReadonlyFieldWrite { owner: Identifier, field: Identifier },
    PrivateConstructorInstantiation { class_name: Identifier },
    VisibilityReduction { class_name: Identifier, member: Identifier, from: AccessLevel, to: AccessLevel },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessControlError {
    pub kind: AccessControlErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl AccessControlError {
    pub fn private_member_access(owner: Identifier, member: Identifier, accessor: Option<Identifier>, span: Option<SourceSpan>) -> Self {
        let accessor_text = accessor.as_ref().map(|name| format!(" by {}", name)).unwrap_or_default();
        Self {
            kind: AccessControlErrorKind::PrivateMemberAccess { owner: owner.clone(), member: member.clone(), accessor },
            message: format!("private member access: {}.{}{}", owner, member, accessor_text),
            span,
        }
    }

    pub fn protected_member_access(owner: Identifier, member: Identifier, accessor: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AccessControlErrorKind::ProtectedMemberAccess { owner: owner.clone(), member: member.clone(), accessor: accessor.clone() },
            message: format!("protected member access denied: {}.{} from {}", owner, member, accessor),
            span,
        }
    }

    pub fn internal_member_access(member: Identifier, declared_module: NamePath, current_module: NamePath, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AccessControlErrorKind::InternalMemberAccess {
                member: member.clone(),
                declared_module: declared_module.clone(),
                current_module: current_module.clone(),
            },
            message: format!(
                "internal member access denied: {} declared in {} from {}",
                member,
                display_path(&declared_module),
                display_path(&current_module)
            ),
            span,
        }
    }

    pub fn readonly_field_write(owner: Identifier, field: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AccessControlErrorKind::ReadonlyFieldWrite { owner: owner.clone(), field: field.clone() },
            message: format!("readonly field write denied: {}.{}", owner, field),
            span,
        }
    }

    pub fn private_constructor_instantiation(class_name: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AccessControlErrorKind::PrivateConstructorInstantiation { class_name: class_name.clone() },
            message: format!("private constructor instantiation denied: {} requires a factory method", class_name),
            span,
        }
    }

    pub fn visibility_reduction(
        class_name: Identifier,
        member: Identifier,
        from: AccessLevel,
        to: AccessLevel,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind: AccessControlErrorKind::VisibilityReduction { class_name: class_name.clone(), member: member.clone(), from, to },
            message: format!("reduced visibility is not allowed: {}.{} from {} to {}", class_name, member, from.as_str(), to.as_str()),
            span,
        }
    }
}

impl fmt::Display for AccessControlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AccessControlError {}

#[derive(Debug, Default)]
pub struct AccessControlChecker {
    errors: Vec<AccessControlError>,
    classes: BTreeMap<Identifier, HirStruct>,
    inheritance_map: BTreeMap<Identifier, Vec<Identifier>>,
}

impl AccessControlChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn errors(&self) -> &[AccessControlError] {
        &self.errors
    }

    pub fn classes(&self) -> &BTreeMap<Identifier, HirStruct> {
        &self.classes
    }

    pub fn inheritance_map(&self) -> &BTreeMap<Identifier, Vec<Identifier>> {
        &self.inheritance_map
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructorChainErrorKind {
    MissingSuperCall { class_name: Identifier, parent: Identifier },
    SuperCallArgumentMismatch { class_name: Identifier, parent: Identifier, expected: usize, got: usize },
    InvalidSuperCallOrder { class_name: Identifier, expected: Vec<Identifier>, actual: Vec<Identifier> },
    DuplicateSuperCall { class_name: Identifier, parent: Identifier },
    InvalidSuperCallMethod { class_name: Identifier, method: Identifier },
    SuperCallArgumentTypeMismatch { class_name: Identifier, parent: Identifier, index: usize, expected: HirType, got: HirType },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstructorChainError {
    pub kind: ConstructorChainErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl ConstructorChainError {
    pub fn missing_super_call(class_name: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstructorChainErrorKind::MissingSuperCall { class_name: class_name.clone(), parent: parent.clone() },
            message: format!("{} is missing required super call to {}", class_name, parent),
            span,
        }
    }

    pub fn super_call_argument_mismatch(
        class_name: Identifier,
        parent: Identifier,
        expected: usize,
        got: usize,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind: ConstructorChainErrorKind::SuperCallArgumentMismatch {
                class_name: class_name.clone(),
                parent: parent.clone(),
                expected,
                got,
            },
            message: format!("Expected {} arguments for {} super call in {}, got {}", expected, parent, class_name, got),
            span,
        }
    }

    pub fn invalid_super_call_order(
        class_name: Identifier,
        expected: Vec<Identifier>,
        actual: Vec<Identifier>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind: ConstructorChainErrorKind::InvalidSuperCallOrder {
                class_name: class_name.clone(),
                expected: expected.clone(),
                actual: actual.clone(),
            },
            message: format!("Invalid MRO super call order in {}", class_name),
            span,
        }
    }

    pub fn duplicate_super_call(class_name: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstructorChainErrorKind::DuplicateSuperCall { class_name: class_name.clone(), parent: parent.clone() },
            message: format!("Duplicate super call to {} in {}", parent, class_name),
            span,
        }
    }

    pub fn invalid_super_call_method(class_name: Identifier, method: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstructorChainErrorKind::InvalidSuperCallMethod { class_name: class_name.clone(), method: method.clone() },
            message: format!("{} may only call super from initiate, not {}", class_name, method),
            span,
        }
    }

    pub fn super_call_argument_type_mismatch(
        class_name: Identifier,
        parent: Identifier,
        index: usize,
        expected: HirType,
        got: HirType,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind: ConstructorChainErrorKind::SuperCallArgumentTypeMismatch {
                class_name: class_name.clone(),
                parent: parent.clone(),
                index,
                expected: expected.clone(),
                got: got.clone(),
            },
            message: format!(
                "super call argument type mismatch in {} -> {} at {}: expected {:?}, got {:?}",
                class_name, parent, index, expected, got
            ),
            span,
        }
    }
}

impl fmt::Display for ConstructorChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConstructorChainError {}

#[derive(Debug, Default)]
pub struct ConstructorChainChecker {
    errors: Vec<ConstructorChainError>,
}

impl ConstructorChainChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn errors(&self) -> &[ConstructorChainError] {
        &self.errors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyErrorKind {
    VirtualStaticConflict { property: Identifier },
    StaticWithSelf { property: Identifier },
    LazyPropertyWithSetter { property: Identifier },
    InvalidOverride { class_name: Identifier, property: Identifier },
    AbstractPropertyWithBody { class_name: Identifier, property: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyError {
    pub kind: PropertyErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl PropertyError {
    pub fn virtual_static_conflict(property: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: PropertyErrorKind::VirtualStaticConflict { property: property.clone() },
            message: format!("property {} cannot be both virtual and static", property),
            span,
        }
    }

    pub fn static_with_self(property: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: PropertyErrorKind::StaticWithSelf { property: property.clone() },
            message: format!("static property {} cannot use self", property),
            span,
        }
    }

    pub fn lazy_with_setter(property: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: PropertyErrorKind::LazyPropertyWithSetter { property: property.clone() },
            message: format!("lazy property {} cannot define a setter", property),
            span,
        }
    }

    pub fn invalid_override(class_name: Identifier, property: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: PropertyErrorKind::InvalidOverride { class_name: class_name.clone(), property: property.clone() },
            message: format!("property {} in {} cannot override without a parent", property, class_name),
            span,
        }
    }

    pub fn abstract_property_with_body(class_name: Identifier, property: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: PropertyErrorKind::AbstractPropertyWithBody { class_name: class_name.clone(), property: property.clone() },
            message: format!("abstract property {} in {} cannot have a concrete body", property, class_name),
            span,
        }
    }
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PropertyError {}

#[derive(Debug, Default)]
pub struct PropertyChecker {
    errors: Vec<PropertyError>,
}

impl PropertyChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<PropertyError> {
        self.errors.clear();
        for class in &module.structs {
            for property in &class.properties {
                self.check_property(class, property);
            }
        }
        self.errors.clone()
    }

    fn check_property(&mut self, class: &HirStruct, property: &HirProperty) {
        if property.is_static && property.is_virtual {
            self.errors.push(PropertyError::virtual_static_conflict(property.name.clone(), None));
        }
        if property.is_lazy && property.setter.is_some() {
            self.errors.push(PropertyError::lazy_with_setter(property.name.clone(), None));
        }
        if property.is_override && class.parents.is_empty() {
            self.errors.push(PropertyError::invalid_override(class.name.clone(), property.name.clone(), None));
        }
        if property.is_abstract && !class.is_abstract {
            self.errors.push(PropertyError::abstract_property_with_body(class.name.clone(), property.name.clone(), None));
        }
    }
}

#[derive(Debug, Default)]
pub struct SetterValidationAnalyzer;

impl SetterValidationAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&mut self, property: &HirProperty) -> Vec<String> {
        vec![format!("validate setter for {}", property.name)]
    }

    pub fn generate_panic_code(property_name: &str, condition: &str) -> String {
        format!("if !({}) {{ panic!(\"invalid setter value for {}\"); }}", condition, property_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValueTypeErrorKind {
    ValueTypeInheritance { class_name: Identifier, parent: Identifier },
    ValueTypeFieldMutation { class_name: Identifier, field: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueTypeError {
    pub kind: ValueTypeErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl ValueTypeError {
    pub fn value_type_inheritance(class_name: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ValueTypeErrorKind::ValueTypeInheritance { class_name: class_name.clone(), parent: parent.clone() },
            message: format!("值类型 {} 不能继承 {}", class_name, parent),
            span,
        }
    }

    pub fn value_type_field_mutation(class_name: Identifier, field: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ValueTypeErrorKind::ValueTypeFieldMutation { class_name: class_name.clone(), field: field.clone() },
            message: format!("值类型 {} 的字段 {} 不能被原地修改", class_name, field),
            span,
        }
    }
}

impl fmt::Display for ValueTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValueTypeError {}

#[derive(Debug, Default)]
pub struct ValueTypeChecker {
    value_types: BTreeSet<Identifier>,
    errors: Vec<ValueTypeError>,
}

impl ValueTypeChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<ValueTypeError> {
        self.clear();
        for class in &module.structs {
            if class.is_value_type {
                self.value_types.insert(class.name.clone());
                if let Some(parent) = class.parents.first().and_then(|parent| last_name(&parent.name)) {
                    self.errors.push(ValueTypeError::value_type_inheritance(class.name.clone(), parent, None));
                }
            }
        }
        self.errors.clone()
    }

    pub fn is_value_type(&self, ty: &HirType) -> bool {
        match ty {
            HirType::Named(name) => self.value_types.contains(name),
            _ => false,
        }
    }

    pub fn get_value_type_names(&self) -> Vec<Identifier> {
        self.value_types.iter().cloned().collect()
    }

    pub fn value_types(&self) -> &BTreeSet<Identifier> {
        &self.value_types
    }

    pub fn errors(&self) -> &[ValueTypeError] {
        &self.errors
    }

    pub fn clear(&mut self) {
        self.value_types.clear();
        self.errors.clear();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentSemantics {
    Copy,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParameterSemantics {
    Copy,
    Reference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReturnSemantics {
    Copy,
    Reference,
}

#[derive(Debug, Default)]
pub struct CopySemanticsValidator {
    value_types: BTreeSet<Identifier>,
}

impl CopySemanticsValidator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_value_type(&mut self, class: &HirStruct) {
        if class.is_value_type {
            self.value_types.insert(class.name.clone());
        }
    }

    fn is_value(&self, ty: &HirType) -> bool {
        matches!(ty, HirType::Named(name) if self.value_types.contains(name))
    }

    pub fn validate_assignment(&self, ty: &HirType) -> AssignmentSemantics {
        if self.is_value(ty) {
            AssignmentSemantics::Copy
        }
        else {
            AssignmentSemantics::Reference
        }
    }

    pub fn validate_parameter_passing(&self, ty: &HirType) -> ParameterSemantics {
        if self.is_value(ty) {
            ParameterSemantics::Copy
        }
        else {
            ParameterSemantics::Reference
        }
    }

    pub fn validate_return(&self, ty: &HirType) -> ReturnSemantics {
        if self.is_value(ty) {
            ReturnSemantics::Copy
        }
        else {
            ReturnSemantics::Reference
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SealedClassErrorKind {
    NonExhaustiveMatch,
    UnknownSealedClass,
    NotSealedBase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedClassError {
    pub kind: SealedClassErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl fmt::Display for SealedClassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SealedClassError {}

#[derive(Debug, Default, Clone)]
pub struct SealedClassRegistry {
    sealed_classes: BTreeSet<Identifier>,
    subclasses: BTreeMap<Identifier, BTreeSet<Identifier>>,
}

impl SealedClassRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_sealed_class(&mut self, class: &HirStruct) {
        if class.is_sealed {
            self.sealed_classes.insert(class.name.clone());
        }
    }

    pub fn register_subclass(&mut self, sealed_name: &Identifier, subclass: &Identifier) -> Result<(), SealedClassError> {
        if !self.sealed_classes.contains(sealed_name) {
            return Err(SealedClassError {
                kind: SealedClassErrorKind::NotSealedBase,
                message: format!("{} is not a sealed class", sealed_name),
                span: None,
            });
        }
        self.subclasses.entry(sealed_name.clone()).or_default().insert(subclass.clone());
        Ok(())
    }

    pub fn is_sealed_class(&self, name: &Identifier) -> bool {
        self.sealed_classes.contains(name)
    }

    pub fn get_permitted_subclasses(&self, name: &Identifier) -> Vec<Identifier> {
        self.subclasses.get(name).map(|items| items.iter().cloned().collect()).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct ExhaustivenessChecker {
    registry: SealedClassRegistry,
}

impl ExhaustivenessChecker {
    pub fn new(registry: SealedClassRegistry) -> Self {
        Self { registry }
    }

    pub fn check_exhaustiveness(&self, sealed_name: &Identifier, covered: &[Identifier]) -> Result<(), SealedClassError> {
        let declared = self.registry.get_permitted_subclasses(sealed_name);
        let covered_set = covered.iter().cloned().collect::<BTreeSet<_>>();
        let missing = declared.into_iter().filter(|name| !covered_set.contains(name)).collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        }
        else {
            Err(SealedClassError {
                kind: SealedClassErrorKind::NonExhaustiveMatch,
                message: format!(
                    "non exhaustive match for {}: missing {}",
                    sealed_name,
                    missing.iter().map(|item| item.to_string()).collect::<Vec<_>>().join(", ")
                ),
                span: None,
            })
        }
    }

    pub fn is_wildcard_exhaustive(&self, _type_name: &Identifier, has_wildcard: bool) -> bool {
        has_wildcard
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidgetErrorKind {
    MissingRenderMethod { widget: Identifier },
    InvalidRenderReturnType { widget: Identifier, found: HirType },
    InvalidStateUpdate { widget: Identifier, field: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetError {
    pub kind: WidgetErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl WidgetError {
    pub fn missing_render_method(widget: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: WidgetErrorKind::MissingRenderMethod { widget: widget.clone() },
            message: format!("widget {} is missing render method", widget),
            span,
        }
    }

    pub fn invalid_render_return_type(widget: Identifier, found: HirType, span: Option<SourceSpan>) -> Self {
        Self {
            kind: WidgetErrorKind::InvalidRenderReturnType { widget: widget.clone(), found: found.clone() },
            message: format!("widget {} render must return Element, found {:?}", widget, found),
            span,
        }
    }

    pub fn invalid_state_update(widget: Identifier, field: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: WidgetErrorKind::InvalidStateUpdate { widget: widget.clone(), field: field.clone() },
            message: format!("widget {} has invalid state update for {}", widget, field),
            span,
        }
    }
}

impl fmt::Display for WidgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for WidgetError {}

#[derive(Debug, Default)]
pub struct WidgetChecker {
    widgets: BTreeMap<Identifier, HirWidget>,
    errors: Vec<WidgetError>,
    current_widget: Option<Identifier>,
    in_event_handler: bool,
    in_lifecycle_method: bool,
}

impl WidgetChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn collect_widgets(&mut self, module: &HirModule) {
        for widget in &module.widgets {
            self.widgets.insert(widget.name.clone(), widget.clone());
        }
        for submodule in &module.submodules {
            self.collect_widgets(submodule);
        }
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<WidgetError> {
        self.clear();
        self.collect_widgets(module);
        for widget in module.widgets.iter().chain(module.submodules.iter().flat_map(|module| module.widgets.iter())) {
            self.current_widget = Some(widget.name.clone());
            if let Some(render) = self.find_render_method(widget) {
                if !self.is_element_type(&render.return_type) {
                    self.errors.push(WidgetError::invalid_render_return_type(widget.name.clone(), render.return_type.clone(), None));
                }
            }
            else {
                self.errors.push(WidgetError::missing_render_method(widget.name.clone(), None));
            }
        }
        self.current_widget = None;
        self.errors.clone()
    }

    pub fn is_element_type(&self, ty: &HirType) -> bool {
        matches!(ty, HirType::Named(name) if name == &Identifier::new("Element"))
    }

    pub fn is_event_handler_method(&self, name: &str) -> bool {
        name.starts_with("on") && !matches!(name, "on_mount" | "on_unmount" | "on_update" | "before_update" | "after_update")
    }

    pub fn is_lifecycle_method(&self, name: &str) -> bool {
        matches!(name, "on_mount" | "on_unmount" | "on_update" | "before_update" | "after_update")
    }

    pub fn is_state_field(&self, name: &Identifier) -> bool {
        let text = name.as_str();
        text.starts_with('_') || text.starts_with("state_")
    }

    pub fn is_valid_state_update_context(&self) -> bool {
        self.in_event_handler || self.in_lifecycle_method
    }

    pub fn set_in_event_handler(&mut self, value: bool) {
        self.in_event_handler = value;
    }

    pub fn set_in_lifecycle_method(&mut self, value: bool) {
        self.in_lifecycle_method = value;
    }

    pub fn get_widget_names(&self) -> Vec<Identifier> {
        self.widgets.keys().cloned().collect()
    }

    pub fn widgets(&self) -> &BTreeMap<Identifier, HirWidget> {
        &self.widgets
    }

    pub fn errors(&self) -> &[WidgetError] {
        &self.errors
    }

    pub fn current_widget(&self) -> Option<&Identifier> {
        self.current_widget.as_ref()
    }

    pub fn clear(&mut self) {
        self.widgets.clear();
        self.errors.clear();
        self.current_widget = None;
        self.in_event_handler = false;
        self.in_lifecycle_method = false;
    }

    pub fn find_render_method<'a>(&self, widget: &'a HirWidget) -> Option<&'a HirFunction> {
        widget.methods.iter().find(|method| method.name.as_str() == "render")
    }
}
