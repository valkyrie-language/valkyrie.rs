#![allow(missing_docs)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use valkyrie_types::{
    hir::{
        AccessLevel, HirEnum, HirExpr, HirExprKind, HirFunction, HirImpl, HirLiteral, HirModule, HirProperty, HirStruct, HirTrait, HirWidget,
        ValkyrieType as HirType,
    },
    Identifier, NamePath, SourceSpan,
};

fn display_path(path: &NamePath) -> String {
    path.parts().iter().map(|part| part.to_string()).collect::<Vec<_>>().join("::")
}

fn last_name(path: &NamePath) -> Option<Identifier> {
    path.parts().last().cloned()
}

fn signed_int32_type() -> HirType {
    HirType::Integer32 { signed: true }
}

fn signed_int64_type() -> HirType {
    HirType::Integer64 { signed: true }
}

fn bool_type() -> HirType {
    HirType::Boolean
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
pub enum FinalClassErrorKind {
    FinalClassInheritance { class_name: Identifier, parent: Identifier },
    FinalMethodOverride { class_name: Identifier, method: Identifier, parent: Identifier },
    FinalPropertyOverride { class_name: Identifier, property: Identifier, parent: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalClassError {
    pub kind: FinalClassErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl FinalClassError {
    pub fn final_class_inheritance(class_name: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: FinalClassErrorKind::FinalClassInheritance { class_name: class_name.clone(), parent: parent.clone() },
            message: format!("{} 不能继承 final 类 {}", class_name, parent),
            span,
        }
    }

    pub fn final_method_override(class_name: Identifier, method: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: FinalClassErrorKind::FinalMethodOverride { class_name: class_name.clone(), method: method.clone(), parent: parent.clone() },
            message: format!("{} 不能重写 final 方法 {}::{}", class_name, parent, method),
            span,
        }
    }

    pub fn final_property_override(class_name: Identifier, property: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: FinalClassErrorKind::FinalPropertyOverride {
                class_name: class_name.clone(),
                property: property.clone(),
                parent: parent.clone(),
            },
            message: format!("{} 不能重写 final 属性 {}::{}", class_name, parent, property),
            span,
        }
    }
}

impl fmt::Display for FinalClassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for FinalClassError {}

#[derive(Debug, Default)]
pub struct FinalClassChecker {
    class_map: BTreeMap<Identifier, HirStruct>,
    inheritance_map: BTreeMap<Identifier, Vec<Identifier>>,
    final_class_names: BTreeSet<Identifier>,
    final_methods: BTreeMap<Identifier, BTreeSet<Identifier>>,
    final_properties: BTreeMap<Identifier, BTreeSet<Identifier>>,
    errors: Vec<FinalClassError>,
}

impl FinalClassChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<FinalClassError> {
        self.clear();
        for class in &module.structs {
            self.class_map.insert(class.name.clone(), class.clone());
            self.inheritance_map.insert(class.name.clone(), class.parents.iter().filter_map(|parent| last_name(&parent.name)).collect());
            if class.is_final {
                self.final_class_names.insert(class.name.clone());
            }
            self.final_methods
                .insert(class.name.clone(), class.methods.iter().filter(|method| method.is_final).map(|method| method.name.clone()).collect());
            self.final_properties.insert(
                class.name.clone(),
                class.properties.iter().filter(|property| property.is_final).map(|property| property.name.clone()).collect(),
            );
        }

        for class in &module.structs {
            let parents = self.ancestor_chain(&class.name);
            for parent in &parents {
                if self.final_class_names.contains(parent) {
                    self.errors.push(FinalClassError::final_class_inheritance(class.name.clone(), parent.clone(), None));
                }
                if let Some(methods) = self.final_methods.get(parent) {
                    for method in &class.methods {
                        if methods.contains(&method.name) {
                            self.errors.push(FinalClassError::final_method_override(
                                class.name.clone(),
                                method.name.clone(),
                                parent.clone(),
                                None,
                            ));
                        }
                    }
                }
                if let Some(properties) = self.final_properties.get(parent) {
                    for property in &class.properties {
                        if properties.contains(&property.name) {
                            self.errors.push(FinalClassError::final_property_override(
                                class.name.clone(),
                                property.name.clone(),
                                parent.clone(),
                                None,
                            ));
                        }
                    }
                }
            }
        }

        self.errors.clone()
    }

    fn ancestor_chain(&self, class_name: &Identifier) -> Vec<Identifier> {
        let mut result = Vec::new();
        let mut seen = BTreeSet::new();
        self.collect_ancestors(class_name, &mut seen, &mut result);
        result
    }

    fn collect_ancestors(&self, class_name: &Identifier, seen: &mut BTreeSet<Identifier>, out: &mut Vec<Identifier>) {
        if let Some(parents) = self.inheritance_map.get(class_name) {
            for parent in parents {
                if seen.insert(parent.clone()) {
                    out.push(parent.clone());
                    self.collect_ancestors(parent, seen, out);
                }
            }
        }
    }

    pub fn is_final_class(&self, name: &Identifier) -> bool {
        self.final_class_names.contains(name)
    }

    pub fn get_final_class_names(&self) -> Vec<Identifier> {
        self.final_class_names.iter().cloned().collect()
    }

    pub fn get_final_methods(&self, name: &Identifier) -> Option<&BTreeSet<Identifier>> {
        self.final_methods.get(name)
    }

    pub fn get_final_properties(&self, name: &Identifier) -> Option<&BTreeSet<Identifier>> {
        self.final_properties.get(name)
    }

    pub fn class_map(&self) -> &BTreeMap<Identifier, HirStruct> {
        &self.class_map
    }

    pub fn inheritance_map(&self) -> &BTreeMap<Identifier, Vec<Identifier>> {
        &self.inheritance_map
    }

    pub fn errors(&self) -> &[FinalClassError] {
        &self.errors
    }

    pub fn clear(&mut self) {
        self.class_map.clear();
        self.inheritance_map.clear();
        self.final_class_names.clear();
        self.final_methods.clear();
        self.final_properties.clear();
        self.errors.clear();
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InferenceTypeVar(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
    UnboundVariable { name: Identifier },
    Mismatch { expected: HirType, found: HirType },
    UnsupportedExpression,
}

#[derive(Debug, Default)]
pub struct TypeInference {
    next_var: usize,
    variables: BTreeMap<Identifier, HirType>,
}

impl TypeInference {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fresh_var(&mut self) -> InferenceTypeVar {
        let current = self.next_var;
        self.next_var += 1;
        InferenceTypeVar(current)
    }

    pub fn bind_variable(&mut self, name: Identifier, ty: HirType) {
        self.variables.insert(name, ty);
    }

    pub fn get_variable_type(&self, name: &Identifier) -> Option<&HirType> {
        self.variables.get(name)
    }

    pub fn infer(&mut self, expr: &HirExpr) -> Result<HirType, TypeError> {
        match &expr.kind {
            HirExprKind::Literal(HirLiteral::Integer64(_)) => Ok(signed_int64_type()),
            HirExprKind::Literal(HirLiteral::Float64(_)) => Ok(HirType::Float64),
            HirExprKind::Literal(HirLiteral::String(_)) => Ok(HirType::Utf8),
            HirExprKind::Literal(HirLiteral::Bool(_)) => Ok(bool_type()),
            HirExprKind::Literal(HirLiteral::Unit) => Ok(HirType::Unit),
            HirExprKind::Variable(identifier) => {
                self.variables.get(&identifier.name).cloned().ok_or_else(|| TypeError::UnboundVariable { name: identifier.name.clone() })
            }
            HirExprKind::Call { callee, args, resolved } => self.infer_call(callee, args, resolved.as_ref()),
            HirExprKind::If { condition, then_branch, else_branch } => {
                let condition_ty = self.infer(condition)?;
                self.unify(&condition_ty, &bool_type())?;
                let then_ty = infer_block_type(self, then_branch)?;
                let else_ty = else_branch.as_deref().map(|branch| infer_block_type(self, branch)).transpose()?.unwrap_or(HirType::Unit);
                self.unify(&then_ty, &else_ty)?;
                Ok(then_ty)
            }
            _ => Err(TypeError::UnsupportedExpression),
        }
    }

    fn infer_call(
        &mut self,
        callee: &HirExpr,
        args: &[HirExpr],
        resolved: Option<&valkyrie_types::hir::HirResolvedCall>,
    ) -> Result<HirType, TypeError> {
        if let Some(resolved) = resolved {
            return Ok(resolved.return_type.clone());
        }
        let arg_types: Vec<HirType> = args.iter().map(|arg| self.infer(arg)).collect::<Result<_, _>>()?;
        let HirExprKind::Path(path) = &callee.kind
        else {
            return Err(TypeError::UnsupportedExpression);
        };
        if path.parts().len() != 1 {
            return Err(TypeError::UnsupportedExpression);
        }

        match (path.parts()[0].as_str(), arg_types.as_slice()) {
            ("infix +" | "infix -" | "infix *" | "infix /" | "infix %", [lhs, rhs]) => {
                self.unify(lhs, rhs)?;
                Ok(lhs.clone())
            }
            ("infix ==" | "infix !=" | "infix <" | "infix <=" | "infix >" | "infix >=", [lhs, rhs]) => {
                self.unify(lhs, rhs)?;
                Ok(bool_type())
            }
            ("prefix -", [inner]) => {
                if self.is_numeric(inner) {
                    Ok(inner.clone())
                }
                else {
                    Err(TypeError::Mismatch { expected: signed_int64_type(), found: inner.clone() })
                }
            }
            ("prefix !", [inner]) => {
                self.unify(inner, &bool_type())?;
                Ok(bool_type())
            }
            _ => Err(TypeError::UnsupportedExpression),
        }
    }

    pub fn unify(&mut self, left: &HirType, right: &HirType) -> Result<(), TypeError> {
        if left == &HirType::AutoType || right == &HirType::AutoType {
            return Ok(());
        }
        match (left, right) {
            (HirType::Array(lhs), HirType::Array(rhs)) => self.unify(lhs, rhs),
            (HirType::Function(lhs_fn), HirType::Function(rhs_fn)) => {
                let lhs_params = &lhs_fn.params;
                let rhs_params = &rhs_fn.params;
                if lhs_params.len() != rhs_params.len() {
                    return Err(TypeError::Mismatch { expected: left.clone(), found: right.clone() });
                }
                for (lhs, rhs) in lhs_params.iter().zip(rhs_params) {
                    self.unify(lhs, rhs)?;
                }
                self.unify(&lhs_fn.return_type, &rhs_fn.return_type)
            }
            (HirType::Tuple(lhs), HirType::Tuple(rhs)) => {
                if lhs.len() != rhs.len() {
                    return Err(TypeError::Mismatch { expected: left.clone(), found: right.clone() });
                }
                for (lhs, rhs) in lhs.iter().zip(rhs) {
                    self.unify(lhs, rhs)?;
                }
                Ok(())
            }
            _ if left == right => Ok(()),
            _ => Err(TypeError::Mismatch { expected: left.clone(), found: right.clone() }),
        }
    }

    pub fn apply_subst(&self, ty: &HirType) -> HirType {
        ty.clone()
    }

    pub fn is_numeric(&self, ty: &HirType) -> bool {
        matches!(ty, HirType::Integer32 { signed: _ } | HirType::Integer64 { signed: _ } | HirType::Float32 | HirType::Float64)
    }

    pub fn is_integer(&self, ty: &HirType) -> bool {
        matches!(ty, HirType::Integer32 { signed: _ } | HirType::Integer64 { signed: _ })
    }

    pub fn clear(&mut self) {
        self.next_var = 0;
        self.variables.clear();
    }
}

fn infer_block_type(inference: &mut TypeInference, block: &valkyrie_types::hir::HirBlock) -> Result<HirType, TypeError> {
    for statement in &block.statements {
        if let valkyrie_types::hir::HirStatementKind::Expr(expr) = &statement.kind {
            let _ = inference.infer(expr)?;
        }
    }

    match &block.expr {
        Some(expr) => inference.infer(expr),
        None => Ok(HirType::Unit),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbstractClassErrorKind {
    AbstractClassInstantiation { class_name: Identifier },
    AbstractMethodNotImplemented { class_name: Identifier, method: Identifier, parent: Identifier },
    AbstractMethodWithBody { class_name: Identifier, method: Identifier },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractClassError {
    pub kind: AbstractClassErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl AbstractClassError {
    pub fn abstract_class_instantiation(class_name: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AbstractClassErrorKind::AbstractClassInstantiation { class_name: class_name.clone() },
            message: format!("不能实例化抽象类 {}", class_name),
            span,
        }
    }

    pub fn abstract_method_not_implemented(class_name: Identifier, method: Identifier, parent: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AbstractClassErrorKind::AbstractMethodNotImplemented {
                class_name: class_name.clone(),
                method: method.clone(),
                parent: parent.clone(),
            },
            message: format!("{} 未实现来自 {} 的抽象方法 {}", class_name, parent, method),
            span,
        }
    }

    pub fn abstract_method_with_body(class_name: Identifier, method: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: AbstractClassErrorKind::AbstractMethodWithBody { class_name: class_name.clone(), method: method.clone() },
            message: format!("{} 的抽象方法 {} 不能带方法体", class_name, method),
            span,
        }
    }
}

impl fmt::Display for AbstractClassError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for AbstractClassError {}

#[derive(Debug, Default)]
pub struct AbstractClassChecker {
    abstract_classes: BTreeSet<Identifier>,
    inheritance_map: BTreeMap<Identifier, Vec<Identifier>>,
    class_methods: BTreeMap<Identifier, BTreeSet<Identifier>>,
    abstract_methods: BTreeMap<Identifier, BTreeSet<Identifier>>,
    errors: Vec<AbstractClassError>,
}

impl AbstractClassChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<AbstractClassError> {
        self.clear();
        let class_map = module.structs.iter().map(|class| (class.name.clone(), class)).collect::<BTreeMap<_, _>>();

        for class in &module.structs {
            self.inheritance_map.insert(class.name.clone(), class.parents.iter().filter_map(|parent| last_name(&parent.name)).collect());
            self.class_methods.insert(class.name.clone(), class.methods.iter().map(|method| method.name.clone()).collect());
            if class.is_abstract {
                self.abstract_classes.insert(class.name.clone());
            }
            self.abstract_methods.insert(class.name.clone(), class.abstract_methods.iter().cloned().collect());

            for method in &class.methods {
                if method.is_abstract && (method.body.expr.is_some() || !method.body.statements.is_empty()) {
                    self.errors.push(AbstractClassError::abstract_method_with_body(class.name.clone(), method.name.clone(), None));
                }
            }
        }

        for class in &module.structs {
            if class.is_abstract {
                continue;
            }
            for parent in self.ancestor_chain(&class.name) {
                if let Some(parent_struct) = class_map.get(&parent) {
                    let required = if !parent_struct.abstract_methods.is_empty() {
                        parent_struct.abstract_methods.clone()
                    }
                    else {
                        parent_struct.methods.iter().filter(|method| method.is_abstract).map(|method| method.name.clone()).collect()
                    };
                    for method in required {
                        if !class.methods.iter().any(|item| item.name == method) {
                            self.errors.push(AbstractClassError::abstract_method_not_implemented(
                                class.name.clone(),
                                method,
                                parent.clone(),
                                None,
                            ));
                        }
                    }
                }
            }
        }

        self.errors.clone()
    }

    fn ancestor_chain(&self, class_name: &Identifier) -> Vec<Identifier> {
        let mut result = Vec::new();
        let mut seen = BTreeSet::new();
        self.collect_ancestors(class_name, &mut seen, &mut result);
        result
    }

    fn collect_ancestors(&self, class_name: &Identifier, seen: &mut BTreeSet<Identifier>, out: &mut Vec<Identifier>) {
        if let Some(parents) = self.inheritance_map.get(class_name) {
            for parent in parents {
                if seen.insert(parent.clone()) {
                    out.push(parent.clone());
                    self.collect_ancestors(parent, seen, out);
                }
            }
        }
    }

    pub fn is_abstract_class(&self, name: &Identifier) -> bool {
        self.abstract_classes.contains(name)
    }

    pub fn get_abstract_class_names(&self) -> Vec<Identifier> {
        self.abstract_classes.iter().cloned().collect()
    }

    pub fn get_abstract_methods(&self, name: &Identifier) -> Option<&BTreeSet<Identifier>> {
        self.abstract_methods.get(name)
    }

    pub fn abstract_classes(&self) -> &BTreeSet<Identifier> {
        &self.abstract_classes
    }

    pub fn inheritance_map(&self) -> &BTreeMap<Identifier, Vec<Identifier>> {
        &self.inheritance_map
    }

    pub fn class_methods(&self) -> &BTreeMap<Identifier, BTreeSet<Identifier>> {
        &self.class_methods
    }

    pub fn errors(&self) -> &[AbstractClassError] {
        &self.errors
    }

    pub fn clear(&mut self) {
        self.abstract_classes.clear();
        self.inheritance_map.clear();
        self.class_methods.clear();
        self.abstract_methods.clear();
        self.errors.clear();
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeVar(pub usize);

impl fmt::Display for TypeVar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "?T{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Lifetime(String);

impl Lifetime {
    pub fn new(name: &str) -> Self {
        Self(name.to_string())
    }

    pub fn static_lifetime() -> Self {
        Self("static".to_string())
    }
}

impl fmt::Display for Lifetime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeConstraint {
    TraitBound { var: TypeVar, trait_name: NamePath, span: Option<SourceSpan> },
    Equality { left: HirType, right: HirType, span: Option<SourceSpan> },
    Subtype { sub: HirType, sup: HirType, span: Option<SourceSpan> },
}

impl TypeConstraint {
    pub fn trait_bound(var: TypeVar, trait_name: NamePath, span: Option<SourceSpan>) -> Self {
        Self::TraitBound { var, trait_name, span }
    }

    pub fn equality(left: HirType, right: HirType, span: Option<SourceSpan>) -> Self {
        Self::Equality { left, right, span }
    }

    pub fn subtype(sub: HirType, sup: HirType, span: Option<SourceSpan>) -> Self {
        Self::Subtype { sub, sup, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitImpl {
    pub ty: HirType,
    pub trait_name: NamePath,
    pub type_args: Vec<HirType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiTraitBound {
    pub var: TypeVar,
    pub traits: Vec<NamePath>,
    pub span: Option<SourceSpan>,
}

impl MultiTraitBound {
    pub fn new(var: TypeVar, traits: Vec<NamePath>, span: Option<SourceSpan>) -> Self {
        Self { var, traits, span }
    }

    pub fn is_empty(&self) -> bool {
        self.traits.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereBound {
    pub ty: HirType,
    pub traits: Vec<NamePath>,
    pub span: Option<SourceSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhereClause {
    pub bounds: Vec<WhereBound>,
    pub span: Option<SourceSpan>,
}

impl WhereClause {
    pub fn new(bounds: Vec<WhereBound>, span: Option<SourceSpan>) -> Self {
        Self { bounds, span }
    }

    pub fn empty() -> Self {
        Self { bounds: vec![], span: None }
    }

    pub fn add_bound(&mut self, bound: WhereBound) {
        self.bounds.push(bound);
    }

    pub fn is_empty(&self) -> bool {
        self.bounds.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LifetimeConstraintKind {
    Outlives { longer: Lifetime, shorter: Lifetime },
    Equality { left: Lifetime, right: Lifetime },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifetimeConstraint {
    pub kind: LifetimeConstraintKind,
    pub span: Option<SourceSpan>,
}

impl LifetimeConstraint {
    pub fn outlives(longer: Lifetime, shorter: Lifetime, span: Option<SourceSpan>) -> Self {
        Self { kind: LifetimeConstraintKind::Outlives { longer, shorter }, span }
    }

    pub fn equality(left: Lifetime, right: Lifetime, span: Option<SourceSpan>) -> Self {
        Self { kind: LifetimeConstraintKind::Equality { left, right }, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssociatedTypeConstraint {
    pub base_type: HirType,
    pub trait_name: NamePath,
    pub assoc_name: Identifier,
    pub expected: HirType,
    pub span: Option<SourceSpan>,
}

impl AssociatedTypeConstraint {
    pub fn new(base_type: HirType, trait_name: NamePath, assoc_name: Identifier, expected: HirType, span: Option<SourceSpan>) -> Self {
        Self { base_type, trait_name, assoc_name, expected, span }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstraintErrorKind {
    TraitNotImplemented { type_name: String, trait_name: String },
    TypeMismatch { expected: HirType, found: HirType },
    InfiniteType { var: TypeVar, ty: HirType },
    AmbiguousType { var: TypeVar },
    AssociatedTypeNotFound { type_name: String, trait_name: String, assoc_name: String },
    PropagationFailed { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintError {
    pub kind: ConstraintErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl ConstraintError {
    pub fn trait_not_implemented(type_name: &str, trait_name: &str, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstraintErrorKind::TraitNotImplemented { type_name: type_name.to_string(), trait_name: trait_name.to_string() },
            message: format!("类型 {} 未实现 trait {}", type_name, trait_name),
            span,
        }
    }

    pub fn type_mismatch(expected: HirType, found: HirType, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstraintErrorKind::TypeMismatch { expected: expected.clone(), found: found.clone() },
            message: format!("类型不匹配: expected {:?}, found {:?}", expected, found),
            span,
        }
    }

    pub fn infinite_type(var: TypeVar, ty: HirType, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstraintErrorKind::InfiniteType { var: var.clone(), ty: ty.clone() },
            message: format!("无限类型: {} occurs in {:?}", var, ty),
            span,
        }
    }

    pub fn ambiguous_type(var: TypeVar, span: Option<SourceSpan>) -> Self {
        Self { kind: ConstraintErrorKind::AmbiguousType { var: var.clone() }, message: format!("类型歧义: {}", var), span }
    }

    pub fn associated_type_not_found(base_type: HirType, trait_name: NamePath, assoc_name: Identifier, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstraintErrorKind::AssociatedTypeNotFound {
                type_name: format!("{:?}", base_type),
                trait_name: display_path(&trait_name),
                assoc_name: assoc_name.to_string(),
            },
            message: "关联类型未找到".to_string(),
            span,
        }
    }

    pub fn propagation_failed(reason: &str, span: Option<SourceSpan>) -> Self {
        Self {
            kind: ConstraintErrorKind::PropagationFailed { reason: reason.to_string() }, message: format!("约束传播失败: {}", reason), span
        }
    }
}

impl fmt::Display for ConstraintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ConstraintError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixSuggestion {
    pub message: String,
    pub priority: i32,
    pub code_example: Option<String>,
}

impl FixSuggestion {
    pub fn new(message: impl Into<String>, priority: i32) -> Self {
        Self { message: message.into(), priority, code_example: None }
    }

    pub fn with_code_example(mut self, code_example: impl Into<String>) -> Self {
        self.code_example = Some(code_example.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintChainNode {
    pub description: String,
    pub span: Option<SourceSpan>,
    pub origin: Option<String>,
}

impl ConstraintChainNode {
    pub fn new(description: String, span: Option<SourceSpan>, origin: Option<String>) -> Self {
        Self { description, span, origin }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstraintReport {
    pub kind: ConstraintErrorKind,
    pub span: Option<SourceSpan>,
    pub primary_message: String,
    pub suggestions: Vec<FixSuggestion>,
    pub related_types: Vec<String>,
    pub chain: Vec<ConstraintChainNode>,
}

impl ConstraintReport {
    pub fn new(kind: ConstraintErrorKind, span: Option<SourceSpan>, primary_message: impl Into<String>) -> Self {
        Self { kind, span, primary_message: primary_message.into(), suggestions: vec![], related_types: vec![], chain: vec![] }
    }

    pub fn add_chain_node(&mut self, node: ConstraintChainNode) {
        self.chain.push(node);
    }

    pub fn add_suggestion(&mut self, suggestion: FixSuggestion) {
        self.suggestions.push(suggestion);
    }

    pub fn to_detailed_message(&self) -> String {
        format!(
            r#"错误: {}
约束追溯: {}
建议修复: {}"#,
            self.primary_message,
            self.chain.len(),
            self.suggestions.len()
        )
    }
}

#[derive(Debug, Default)]
pub struct TraitBoundChecker {
    trait_impls: Vec<TraitImpl>,
}

impl TraitBoundChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_builtin_trait(&self, trait_name: &NamePath) -> bool {
        matches!(display_path(trait_name).as_str(), "Copy" | "Clone" | "Eq" | "Send" | "Sync" | "Default")
    }

    pub fn register_trait_impl(&mut self, item: TraitImpl) {
        self.trait_impls.push(item);
    }

    pub fn check_trait_bound(&self, ty: &HirType, trait_name: &NamePath) -> Result<bool, ConstraintError> {
        if self.is_builtin_trait(trait_name)
            && matches!(
                ty,
                HirType::Integer32 { signed: _ } | HirType::Integer64 { signed: _ } | HirType::Float32 | HirType::Float64 | HirType::Boolean
            )
        {
            return Ok(true);
        }
        Ok(self.trait_impls.iter().any(|item| &item.ty == ty && item.trait_name == *trait_name))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImplyErrorKind {
    UnknownTrait { trait_name: NamePath },
    DuplicateImpl { target: HirType, trait_name: Option<NamePath> },
    OverlappingImpl { target: HirType, trait_name: Option<NamePath> },
    MissingSuperTraitImpl { trait_name: NamePath, super_trait: NamePath, target: HirType },
    MissingMethod { trait_name: NamePath, method: Identifier },
    DuplicateMethod { trait_name: NamePath, method: Identifier },
    ExtraMethod { trait_name: NamePath, method: Identifier },
    MethodSignatureMismatch { trait_name: NamePath, method: Identifier, expected: String, found: String },
    MissingAssociatedType { trait_name: NamePath, name: Identifier },
    DuplicateAssociatedType { trait_name: NamePath, name: Identifier },
    UnknownAssociatedType { trait_name: NamePath, name: Identifier },
    MissingAssociatedConst { trait_name: NamePath, name: Identifier },
    DuplicateAssociatedConst { trait_name: NamePath, name: Identifier },
    UnknownAssociatedConst { trait_name: NamePath, name: Identifier },
    AssociatedConstTypeMismatch { trait_name: NamePath, name: Identifier, expected: HirType, found: HirType },
    AssociatedConstValueTypeMismatch { trait_name: NamePath, name: Identifier, expected: HirType, found: HirType },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImplyError {
    pub kind: ImplyErrorKind,
    pub message: String,
    pub span: Option<SourceSpan>,
}

impl fmt::Display for ImplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ImplyError {}

#[derive(Debug, Default)]
pub struct ImplyChecker {
    traits: BTreeMap<Identifier, HirTrait>,
    errors: Vec<ImplyError>,
}

impl ImplyChecker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn errors(&self) -> &[ImplyError] {
        &self.errors
    }

    pub fn check_module(&mut self, module: &HirModule) -> Vec<ImplyError> {
        self.clear();
        for trait_def in &module.traits {
            self.traits.insert(trait_def.name.clone(), trait_def.clone());
        }
        for (index, impl_block) in module.impls.iter().enumerate() {
            self.check_duplicate_impl(index, &module.impls);
            self.check_impl(impl_block, &module.impls);
        }
        self.errors.clone()
    }

    pub fn clear(&mut self) {
        self.traits.clear();
        self.errors.clear();
    }

    fn check_duplicate_impl(&mut self, index: usize, impls: &[HirImpl]) {
        let impl_block = &impls[index];
        for existing in impls[..index].iter().filter(|existing| same_trait_impl_head(existing, impl_block)) {
            match compare_impl_specificity(existing, impl_block) {
                ImplSpecificity::Equivalent => self.errors.push(ImplyError {
                    kind: ImplyErrorKind::DuplicateImpl { target: impl_block.target.clone(), trait_name: impl_block.trait_path.clone() },
                    message: format!(
                        "重复的 imply 实现: target={:?}, trait={}",
                        impl_block.target,
                        impl_block.trait_path.as_ref().map(display_path).unwrap_or_else(|| "<inherent>".to_string())
                    ),
                    span: None,
                }),
                ImplSpecificity::Incomparable => self.errors.push(ImplyError {
                    kind: ImplyErrorKind::OverlappingImpl { target: impl_block.target.clone(), trait_name: impl_block.trait_path.clone() },
                    message: format!(
                        "重叠的 imply 实现: target={:?}, trait={}",
                        impl_block.target,
                        impl_block.trait_path.as_ref().map(display_path).unwrap_or_else(|| "<inherent>".to_string())
                    ),
                    span: None,
                }),
                ImplSpecificity::MoreSpecific | ImplSpecificity::LessSpecific => {}
            }
        }
    }

    fn check_impl(&mut self, impl_block: &HirImpl, impls: &[HirImpl]) {
        let Some(trait_path) = &impl_block.trait_path
        else {
            return;
        };
        let Some(trait_name) = last_name(trait_path)
        else {
            self.errors.push(ImplyError {
                kind: ImplyErrorKind::UnknownTrait { trait_name: trait_path.clone() },
                message: format!("未知 trait {}", display_path(trait_path)),
                span: None,
            });
            return;
        };
        let Some(trait_def) = self.traits.get(&trait_name).cloned()
        else {
            self.errors.push(ImplyError {
                kind: ImplyErrorKind::UnknownTrait { trait_name: trait_path.clone() },
                message: format!("未知 trait {}", display_path(trait_path)),
                span: None,
            });
            return;
        };

        self.check_super_traits(impl_block, &trait_def, trait_path, impls);
        self.check_trait_methods(impl_block, &trait_def, trait_path);
        self.check_associated_types(impl_block, &trait_def, trait_path);
        self.check_associated_consts(impl_block, &trait_def, trait_path);
    }

    fn check_super_traits(&mut self, impl_block: &HirImpl, trait_def: &HirTrait, trait_path: &NamePath, impls: &[HirImpl]) {
        for super_trait_path in collect_super_trait_paths(&self.traits, trait_def) {
            if !impls.iter().any(|candidate| candidate.target == impl_block.target && candidate.trait_path.as_ref() == Some(&super_trait_path))
            {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::MissingSuperTraitImpl {
                        trait_name: trait_path.clone(),
                        super_trait: super_trait_path.clone(),
                        target: impl_block.target.clone(),
                    },
                    message: format!("trait impl {} 缺少 super trait {} 的显式实现", display_path(trait_path), display_path(&super_trait_path)),
                    span: None,
                });
            }
        }
    }

    fn check_trait_methods(&mut self, impl_block: &HirImpl, trait_def: &HirTrait, trait_path: &NamePath) {
        let mut trait_methods = BTreeMap::new();
        for method in &trait_def.methods {
            trait_methods.insert(method.name.clone(), method);
        }
        for method in &trait_def.default_methods {
            trait_methods.insert(method.name.clone(), method);
        }

        let mut impl_methods = BTreeMap::<Identifier, Vec<&HirFunction>>::new();
        for method in &impl_block.methods {
            impl_methods.entry(method.name.clone()).or_default().push(method);
        }

        for (name, items) in &impl_methods {
            if items.len() > 1 {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::DuplicateMethod { trait_name: trait_path.clone(), method: name.clone() },
                    message: format!("trait impl {} 中的方法 {} 重复定义", display_path(trait_path), name),
                    span: None,
                });
            }
            if !trait_methods.contains_key(name) {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::ExtraMethod { trait_name: trait_path.clone(), method: name.clone() },
                    message: format!("trait impl {} 中的方法 {} 不在 trait 声明内", display_path(trait_path), name),
                    span: None,
                });
            }
        }

        for method in &trait_def.methods {
            match impl_methods.get(&method.name) {
                None => self.errors.push(ImplyError {
                    kind: ImplyErrorKind::MissingMethod { trait_name: trait_path.clone(), method: method.name.clone() },
                    message: format!("trait impl {} 缺少必需方法 {}", display_path(trait_path), method.name),
                    span: None,
                }),
                Some(items) if items.len() == 1 => {
                    let found = items[0];
                    if !same_method_signature(method, found) {
                        self.errors.push(ImplyError {
                            kind: ImplyErrorKind::MethodSignatureMismatch {
                                trait_name: trait_path.clone(),
                                method: method.name.clone(),
                                expected: render_method_signature(method),
                                found: render_method_signature(found),
                            },
                            message: format!("trait impl {} 的方法 {} 签名不匹配", display_path(trait_path), method.name),
                            span: Some(found.span.clone()),
                        });
                    }
                }
                Some(_) => {}
            }
        }

        for method in &trait_def.default_methods {
            if let Some(items) = impl_methods.get(&method.name) {
                if items.len() == 1 && !same_method_signature(method, items[0]) {
                    self.errors.push(ImplyError {
                        kind: ImplyErrorKind::MethodSignatureMismatch {
                            trait_name: trait_path.clone(),
                            method: method.name.clone(),
                            expected: render_method_signature(method),
                            found: render_method_signature(items[0]),
                        },
                        message: format!("trait impl {} 重写默认方法 {} 时签名不匹配", display_path(trait_path), method.name),
                        span: Some(items[0].span.clone()),
                    });
                }
            }
        }
    }

    fn check_associated_types(&mut self, impl_block: &HirImpl, trait_def: &HirTrait, trait_path: &NamePath) {
        let declared = trait_def.associated_types.iter().map(|item| (item.name.clone(), item)).collect::<BTreeMap<_, _>>();
        let mut bound_counts = BTreeMap::<Identifier, usize>::new();

        for binding in &impl_block.associated_type_impls {
            *bound_counts.entry(binding.name.clone()).or_default() += 1;
            if !declared.contains_key(&binding.name) {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::UnknownAssociatedType { trait_name: trait_path.clone(), name: binding.name.clone() },
                    message: format!("trait impl {} 提供了未知关联类型 {}", display_path(trait_path), binding.name),
                    span: Some(binding.span.clone()),
                });
            }
        }

        for (name, count) in &bound_counts {
            if *count > 1 {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::DuplicateAssociatedType { trait_name: trait_path.clone(), name: name.clone() },
                    message: format!("trait impl {} 的关联类型 {} 重复绑定", display_path(trait_path), name),
                    span: None,
                });
            }
        }

        for item in &trait_def.associated_types {
            if item.default.is_none() && !bound_counts.contains_key(&item.name) {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::MissingAssociatedType { trait_name: trait_path.clone(), name: item.name.clone() },
                    message: format!("trait impl {} 缺少关联类型 {}", display_path(trait_path), item.name),
                    span: None,
                });
            }
        }
    }

    fn check_associated_consts(&mut self, impl_block: &HirImpl, trait_def: &HirTrait, trait_path: &NamePath) {
        let declared = trait_def.associated_constants.iter().map(|item| (item.name.clone(), item)).collect::<BTreeMap<_, _>>();
        let mut bound_counts = BTreeMap::<Identifier, usize>::new();

        for binding in &impl_block.associated_const_impls {
            *bound_counts.entry(binding.name.clone()).or_default() += 1;
            let Some(declared_item) = declared.get(&binding.name)
            else {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::UnknownAssociatedConst { trait_name: trait_path.clone(), name: binding.name.clone() },
                    message: format!("trait impl {} 提供了未知关联常量 {}", display_path(trait_path), binding.name),
                    span: Some(binding.span.clone()),
                });
                continue;
            };
            if let Some(found) = &binding.const_type {
                if found != &declared_item.const_type {
                    self.errors.push(ImplyError {
                        kind: ImplyErrorKind::AssociatedConstTypeMismatch {
                            trait_name: trait_path.clone(),
                            name: binding.name.clone(),
                            expected: declared_item.const_type.clone(),
                            found: found.clone(),
                        },
                        message: format!("trait impl {} 的关联常量 {} 类型不匹配", display_path(trait_path), binding.name),
                        span: Some(binding.span.clone()),
                    });
                }
            }
            let mut inference = TypeInference::new();
            if let Ok(found_value_type) = inference.infer(&binding.value) {
                if found_value_type != declared_item.const_type {
                    self.errors.push(ImplyError {
                        kind: ImplyErrorKind::AssociatedConstValueTypeMismatch {
                            trait_name: trait_path.clone(),
                            name: binding.name.clone(),
                            expected: declared_item.const_type.clone(),
                            found: found_value_type.clone(),
                        },
                        message: format!("trait impl {} 的关联常量 {} 值类型不匹配", display_path(trait_path), binding.name),
                        span: Some(binding.span.clone()),
                    });
                }
            }
        }

        for (name, count) in &bound_counts {
            if *count > 1 {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::DuplicateAssociatedConst { trait_name: trait_path.clone(), name: name.clone() },
                    message: format!("trait impl {} 的关联常量 {} 重复绑定", display_path(trait_path), name),
                    span: None,
                });
            }
        }

        for item in &trait_def.associated_constants {
            if item.default_value.is_none() && !bound_counts.contains_key(&item.name) {
                self.errors.push(ImplyError {
                    kind: ImplyErrorKind::MissingAssociatedConst { trait_name: trait_path.clone(), name: item.name.clone() },
                    message: format!("trait impl {} 缺少关联常量 {}", display_path(trait_path), item.name),
                    span: None,
                });
            }
        }
    }
}

fn same_trait_impl_head(left: &HirImpl, right: &HirImpl) -> bool {
    match (&left.trait_path, &right.trait_path) {
        (Some(left_trait), Some(right_trait)) => left.target == right.target && left_trait == right_trait,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImplSpecificity {
    Equivalent,
    MoreSpecific,
    LessSpecific,
    Incomparable,
}

fn compare_impl_specificity(left: &HirImpl, right: &HirImpl) -> ImplSpecificity {
    let left_pairs = impl_where_constraint_pairs(left);
    let right_pairs = impl_where_constraint_pairs(right);

    match (left_pairs == right_pairs, left_pairs.is_superset(&right_pairs), right_pairs.is_superset(&left_pairs)) {
        (true, _, _) => ImplSpecificity::Equivalent,
        (false, true, false) => ImplSpecificity::MoreSpecific,
        (false, false, true) => ImplSpecificity::LessSpecific,
        _ => ImplSpecificity::Incomparable,
    }
}

fn impl_where_constraint_pairs(impl_block: &HirImpl) -> BTreeSet<(HirType, NamePath)> {
    impl_block
        .where_constraints
        .iter()
        .flat_map(|constraint| constraint.bounds.iter().cloned().map(|bound| (constraint.target.clone(), bound)))
        .collect()
}

fn same_method_signature(expected: &HirFunction, found: &HirFunction) -> bool {
    expected.generics.len() == found.generics.len()
        && expected.return_type == found.return_type
        && expected.params.len() == found.params.len()
        && expected.params.iter().zip(&found.params).all(|(left, right)| left.ty == right.ty)
}

fn render_method_signature(method: &HirFunction) -> String {
    let params = method.params.iter().map(|param| format!("{:?}", param.ty)).collect::<Vec<_>>().join(", ");
    format!("{}({}) -> {:?}", method.name, params, method.return_type)
}

fn trait_type_to_path(ty: &HirType) -> Option<NamePath> {
    match ty {
        HirType::Named(name) => Some(NamePath::new(vec![name.clone()])),
        HirType::Apply(base, _) => match base.as_ref() {
            HirType::Named(name) => Some(NamePath::new(vec![name.clone()])),
            _ => None,
        },
        _ => None,
    }
}

fn collect_super_trait_paths(traits: &BTreeMap<Identifier, HirTrait>, trait_def: &HirTrait) -> Vec<NamePath> {
    let mut visited = BTreeSet::new();
    let mut collected = Vec::new();
    collect_super_trait_paths_inner(traits, trait_def, &mut visited, &mut collected);
    collected
}

fn collect_super_trait_paths_inner(
    traits: &BTreeMap<Identifier, HirTrait>,
    trait_def: &HirTrait,
    visited: &mut BTreeSet<NamePath>,
    collected: &mut Vec<NamePath>,
) {
    for super_trait in &trait_def.super_traits {
        let Some(super_trait_path) = trait_type_to_path(super_trait)
        else {
            continue;
        };
        if !visited.insert(super_trait_path.clone()) {
            continue;
        }
        collected.push(super_trait_path.clone());
        if let Some(super_trait_name) = last_name(&super_trait_path) {
            if let Some(super_trait_def) = traits.get(&super_trait_name) {
                collect_super_trait_paths_inner(traits, super_trait_def, visited, collected);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct ConstraintPropagator {
    trait_bounds: BTreeMap<TypeVar, Vec<NamePath>>,
}

impl ConstraintPropagator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn propagate_to_caller(&mut self, caller_var: TypeVar, constraints: &[TypeConstraint], _origin: Option<String>) {
        for constraint in constraints {
            if let TypeConstraint::TraitBound { trait_name, .. } = constraint {
                self.trait_bounds.entry(caller_var.clone()).or_default().push(trait_name.clone());
            }
        }
    }

    pub fn compute_transitive_closure(&mut self) -> Result<(), ConstraintError> {
        Ok(())
    }

    pub fn get_trait_bounds(&self, var: &TypeVar) -> Option<&Vec<NamePath>> {
        self.trait_bounds.get(var)
    }
}

#[derive(Debug, Clone)]
struct RegisteredUnityVariant {
    name: Identifier,
    result_type: Option<HirType>,
}

#[derive(Debug, Clone)]
struct RegisteredUnity {
    base: Identifier,
    generics_len: usize,
    variants: Vec<RegisteredUnityVariant>,
}

#[derive(Debug, Default)]
pub struct ConstraintSolver {
    constraints: Vec<TypeConstraint>,
    substitutions: BTreeMap<TypeVar, HirType>,
    generic_bindings: BTreeMap<Identifier, HirType>,
    trait_bounds: BTreeMap<TypeVar, Vec<NamePath>>,
    trait_impls: Vec<TraitImpl>,
    multi_trait_bounds: Vec<MultiTraitBound>,
    where_clauses: Vec<WhereClause>,
    lifetime_constraints: Vec<LifetimeConstraint>,
    associated_type_constraints: Vec<AssociatedTypeConstraint>,
    associated_type_impls: Vec<(HirType, NamePath, Identifier, HirType)>,
    propagator: ConstraintPropagator,
    unity_types: Vec<RegisteredUnity>,
}

impl ConstraintSolver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn constraints(&self) -> &[TypeConstraint] {
        &self.constraints
    }

    pub fn substitutions(&self) -> &BTreeMap<TypeVar, HirType> {
        &self.substitutions
    }

    pub fn generic_bindings(&self) -> &BTreeMap<Identifier, HirType> {
        &self.generic_bindings
    }

    pub fn trait_bounds(&self) -> &BTreeMap<TypeVar, Vec<NamePath>> {
        &self.trait_bounds
    }

    pub fn get_substitutions(&self) -> &BTreeMap<TypeVar, HirType> {
        &self.substitutions
    }

    pub fn add_constraint(&mut self, constraint: TypeConstraint) {
        if let TypeConstraint::TraitBound { var, trait_name, .. } = &constraint {
            self.trait_bounds.entry(var.clone()).or_default().push(trait_name.clone());
        }
        self.constraints.push(constraint);
    }

    pub fn add_substitution(&mut self, var: TypeVar, ty: HirType) {
        self.substitutions.insert(var, ty);
    }

    pub fn add_multi_trait_bound(&mut self, bound: MultiTraitBound) {
        self.multi_trait_bounds.push(bound);
    }

    pub fn add_where_clause(&mut self, clause: WhereClause) {
        self.where_clauses.push(clause);
    }

    pub fn add_lifetime_constraint(&mut self, constraint: LifetimeConstraint) {
        self.lifetime_constraints.push(constraint);
    }

    pub fn add_associated_type_constraint(&mut self, constraint: AssociatedTypeConstraint) {
        self.associated_type_constraints.push(constraint);
    }

    pub fn register_trait_impl(&mut self, item: TraitImpl) {
        self.trait_impls.push(item);
    }

    pub fn register_associated_type_impl(&mut self, base_type: HirType, trait_name: NamePath, assoc_name: Identifier, concrete_type: HirType) {
        self.associated_type_impls.push((base_type, trait_name, assoc_name, concrete_type));
    }

    pub fn resolve_associated_type(&self, base_type: &HirType, trait_name: &NamePath, assoc_name: &Identifier) -> Option<HirType> {
        self.associated_type_impls
            .iter()
            .find(|(base, trait_path, assoc, _)| base == base_type && trait_path == trait_name && assoc == assoc_name)
            .map(|(_, _, _, concrete)| concrete.clone())
    }

    pub fn register_unity_type(&mut self, enum_def: &HirEnum) {
        self.unity_types.push(RegisteredUnity {
            base: enum_def.name.clone(),
            generics_len: enum_def.generics.len(),
            variants: enum_def
                .variants
                .iter()
                .map(|variant| RegisteredUnityVariant { name: variant.name.clone(), result_type: variant.result_type.clone() })
                .collect(),
        });
    }

    fn apply_generic_bindings(&self, ty: &HirType) -> HirType {
        self.apply_generic_bindings_with_seen(ty, &mut BTreeSet::new())
    }

    fn apply_generic_bindings_with_seen(&self, ty: &HirType, seen: &mut BTreeSet<Identifier>) -> HirType {
        match ty {
            HirType::Generic(generic) => {
                let Some(bound) = self.generic_bindings.get(&generic.name)
                else {
                    return ty.clone();
                };
                if !seen.insert(generic.name.clone()) {
                    return ty.clone();
                }
                let resolved = self.apply_generic_bindings_with_seen(bound, seen);
                seen.remove(&generic.name);
                resolved
            }
            HirType::Apply(base, args) => HirType::Apply(
                Box::new(self.apply_generic_bindings_with_seen(base, seen)),
                args.iter().map(|arg| self.apply_generic_bindings_with_seen(arg, seen)).collect(),
            ),
            HirType::Tuple(items) => HirType::Tuple(items.iter().map(|item| self.apply_generic_bindings_with_seen(item, seen)).collect()),
            HirType::Array(item) => HirType::Array(Box::new(self.apply_generic_bindings_with_seen(item, seen))),
            HirType::Function(function) => HirType::Function(Box::new(valkyrie_types::hir::FunctionType {
                params: function.params.iter().map(|param| self.apply_generic_bindings_with_seen(param, seen)).collect(),
                return_type: self.apply_generic_bindings_with_seen(&function.return_type, seen),
            })),
            HirType::Associated(associated) => HirType::Associated(Box::new(valkyrie_types::hir::AssociatedType {
                base: self.apply_generic_bindings_with_seen(&associated.base, seen),
                name: associated.name.clone(),
                type_arguments: associated.type_arguments.iter().map(|arg| self.apply_generic_bindings_with_seen(arg, seen)).collect(),
            })),
            _ => ty.clone(),
        }
    }

    fn has_unresolved_generics(&self, ty: &HirType) -> bool {
        match ty {
            HirType::Generic(_) => true,
            HirType::Apply(base, args) => self.has_unresolved_generics(base) || args.iter().any(|arg| self.has_unresolved_generics(arg)),
            HirType::Tuple(items) => items.iter().any(|item| self.has_unresolved_generics(item)),
            HirType::Array(item) => self.has_unresolved_generics(item),
            HirType::Function(function) => {
                function.params.iter().any(|param| self.has_unresolved_generics(param)) || self.has_unresolved_generics(&function.return_type)
            }
            HirType::Associated(associated) => {
                self.has_unresolved_generics(&associated.base) || associated.type_arguments.iter().any(|arg| self.has_unresolved_generics(arg))
            }
            _ => false,
        }
    }

    fn bind_generic(&mut self, name: &Identifier, actual: &HirType, span: Option<SourceSpan>) -> Result<(), ConstraintError> {
        let actual = self.apply_generic_bindings(actual);
        if matches!(&actual, HirType::Generic(generic) if &generic.name == name) {
            return Ok(());
        }
        if let Some(bound) = self.generic_bindings.get(name).cloned() {
            if let HirType::Generic(actual_generic) = &actual {
                if !self.generic_bindings.contains_key(&actual_generic.name) {
                    self.generic_bindings.insert(actual_generic.name.clone(), bound);
                    return Ok(());
                }
            }
            return self.unify(&self.apply_generic_bindings(&bound), &actual, span);
        }
        if let HirType::Generic(actual_generic) = &actual {
            if let Some(bound) = self.generic_bindings.get(&actual_generic.name).cloned() {
                self.generic_bindings.insert(name.clone(), bound);
                return Ok(());
            }
        }
        self.generic_bindings.insert(name.clone(), actual);
        Ok(())
    }

    fn collect_generic_bindings(&mut self, left: &HirType, right: &HirType, span: Option<SourceSpan>) -> Result<(), ConstraintError> {
        let left = self.apply_generic_bindings(left);
        let right = self.apply_generic_bindings(right);
        match (&left, &right) {
            (HirType::Generic(generic), actual) => self.bind_generic(&generic.name, actual, span),
            (actual, HirType::Generic(generic)) => self.bind_generic(&generic.name, actual, span),
            (HirType::Apply(lhs_base, lhs_args), HirType::Apply(rhs_base, rhs_args)) => {
                self.collect_generic_bindings(lhs_base, rhs_base, span.clone())?;
                for (lhs, rhs) in lhs_args.iter().zip(rhs_args) {
                    self.collect_generic_bindings(lhs, rhs, span.clone())?;
                }
                Ok(())
            }
            (HirType::Tuple(lhs_items), HirType::Tuple(rhs_items)) => {
                for (lhs, rhs) in lhs_items.iter().zip(rhs_items) {
                    self.collect_generic_bindings(lhs, rhs, span.clone())?;
                }
                Ok(())
            }
            (HirType::Array(lhs_item), HirType::Array(rhs_item)) => self.collect_generic_bindings(lhs_item, rhs_item, span),
            (HirType::Function(lhs_fn), HirType::Function(rhs_fn)) => {
                let lhs_params = &lhs_fn.params;
                let rhs_params = &rhs_fn.params;
                for (lhs, rhs) in lhs_params.iter().zip(rhs_params) {
                    self.collect_generic_bindings(lhs, rhs, span.clone())?;
                }
                self.collect_generic_bindings(&lhs_fn.return_type, &rhs_fn.return_type, span)
            }
            (HirType::Associated(lhs_assoc), HirType::Associated(rhs_assoc))
                if lhs_assoc.name == rhs_assoc.name && lhs_assoc.type_arguments.len() == rhs_assoc.type_arguments.len() =>
            {
                self.collect_generic_bindings(&lhs_assoc.base, &rhs_assoc.base, span.clone())?;
                for (lhs, rhs) in lhs_assoc.type_arguments.iter().zip(&rhs_assoc.type_arguments) {
                    self.collect_generic_bindings(lhs, rhs, span.clone())?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn solve_where_clauses(&self) -> Result<(), ConstraintError> {
        for clause in &self.where_clauses {
            for bound in &clause.bounds {
                let resolved_ty = self.apply_generic_bindings(&bound.ty);
                if self.has_unresolved_generics(&resolved_ty) {
                    continue;
                }
                for trait_name in &bound.traits {
                    if !self.check_trait_bound(&resolved_ty, trait_name)? {
                        return Err(ConstraintError::trait_not_implemented(
                            &format!("{:?}", resolved_ty),
                            &display_path(trait_name),
                            bound.span.clone().or_else(|| clause.span.clone()),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn unify(&mut self, left: &HirType, right: &HirType, span: Option<SourceSpan>) -> Result<(), ConstraintError> {
        if left == &HirType::AutoType || right == &HirType::AutoType {
            return Ok(());
        }
        match (left, right) {
            (HirType::Array(lhs), HirType::Array(rhs)) => self.unify(lhs, rhs, span),
            (HirType::Tuple(lhs), HirType::Tuple(rhs)) => {
                if lhs.len() != rhs.len() {
                    return Err(ConstraintError::type_mismatch(left.clone(), right.clone(), span));
                }
                for (lhs, rhs) in lhs.iter().zip(rhs) {
                    self.unify(lhs, rhs, span.clone())?;
                }
                Ok(())
            }
            (HirType::Function(lhs_fn), HirType::Function(rhs_fn)) => {
                let lhs_params = &lhs_fn.params;
                let rhs_params = &rhs_fn.params;
                if lhs_params.len() != rhs_params.len() {
                    return Err(ConstraintError::type_mismatch(left.clone(), right.clone(), span));
                }
                for (lhs, rhs) in lhs_params.iter().zip(rhs_params) {
                    self.unify(lhs, rhs, span.clone())?;
                }
                self.unify(&lhs_fn.return_type, &rhs_fn.return_type, span)
            }
            (HirType::Associated(lhs_assoc), HirType::Associated(rhs_assoc)) => {
                if lhs_assoc.name != rhs_assoc.name || lhs_assoc.type_arguments.len() != rhs_assoc.type_arguments.len() {
                    return Err(ConstraintError::type_mismatch(left.clone(), right.clone(), span));
                }
                self.unify(&lhs_assoc.base, &rhs_assoc.base, span.clone())?;
                for (lhs, rhs) in lhs_assoc.type_arguments.iter().zip(&rhs_assoc.type_arguments) {
                    self.unify(lhs, rhs, span.clone())?;
                }
                Ok(())
            }
            _ if left == right => Ok(()),
            _ => Err(ConstraintError::type_mismatch(left.clone(), right.clone(), span)),
        }
    }

    pub fn check_trait_bound(&self, ty: &HirType, trait_name: &NamePath) -> Result<bool, ConstraintError> {
        if matches!(display_path(trait_name).as_str(), "Copy" | "Clone" | "Eq" | "Send" | "Sync" | "Default")
            && matches!(
                ty,
                HirType::Integer32 { signed: _ } | HirType::Integer64 { signed: _ } | HirType::Float32 | HirType::Float64 | HirType::Boolean
            )
        {
            return Ok(true);
        }
        Ok(self.trait_impls.iter().any(|item| &item.ty == ty && item.trait_name == *trait_name))
    }

    pub fn is_subtype(&self, sub: &HirType, sup: &HirType) -> bool {
        if sub == sup {
            return true;
        }
        if matches!((sub, sup), (HirType::Integer32 { signed: _ }, HirType::Integer64 { signed: _ }) | (HirType::Float32, HirType::Float64)) {
            return true;
        }
        self.is_unity_variant_subtype(sub, sup)
    }

    fn is_unity_variant_subtype(&self, sub: &HirType, sup: &HirType) -> bool {
        let HirType::Named(variant_name) = sub
        else {
            return false;
        };

        for unity in &self.unity_types {
            for variant in &unity.variants {
                if &variant.name != variant_name {
                    continue;
                }
                if let Some(result_type) = &variant.result_type {
                    return self.matches_type_pattern(result_type, sup, &mut BTreeMap::new());
                }
                return match sup {
                    HirType::Named(base) => unity.generics_len == 0 && base == &unity.base,
                    HirType::Apply(base, args) => {
                        matches!(base.as_ref(), HirType::Named(base_name) if base_name == &unity.base) && args.len() == unity.generics_len
                    }
                    _ => false,
                };
            }
        }

        false
    }

    fn matches_type_pattern(&self, pattern: &HirType, concrete: &HirType, bindings: &mut BTreeMap<Identifier, HirType>) -> bool {
        match (pattern, concrete) {
            (HirType::Generic(generic), actual) => match bindings.get(&generic.name) {
                Some(bound) => bound == actual,
                None => {
                    bindings.insert(generic.name.clone(), actual.clone());
                    true
                }
            },
            (HirType::Named(lhs), HirType::Named(rhs)) => lhs == rhs,
            (HirType::Apply(lhs_base, lhs_args), HirType::Apply(rhs_base, rhs_args)) => {
                lhs_args.len() == rhs_args.len()
                    && self.matches_type_pattern(lhs_base, rhs_base, bindings)
                    && lhs_args.iter().zip(rhs_args).all(|(lhs, rhs)| self.matches_type_pattern(lhs, rhs, bindings))
            }
            (HirType::Tuple(lhs_items), HirType::Tuple(rhs_items)) => {
                lhs_items.len() == rhs_items.len()
                    && lhs_items.iter().zip(rhs_items).all(|(lhs, rhs)| self.matches_type_pattern(lhs, rhs, bindings))
            }
            (HirType::Array(lhs_item), HirType::Array(rhs_item)) => self.matches_type_pattern(lhs_item, rhs_item, bindings),
            (HirType::Function(lhs_fn), HirType::Function(rhs_fn)) => {
                lhs_fn.params.len() == rhs_fn.params.len()
                    && lhs_fn.params.iter().zip(&rhs_fn.params).all(|(lhs, rhs)| self.matches_type_pattern(lhs, rhs, bindings))
                    && self.matches_type_pattern(&lhs_fn.return_type, &rhs_fn.return_type, bindings)
            }
            (HirType::Associated(lhs_assoc), HirType::Associated(rhs_assoc)) => {
                lhs_assoc.name == rhs_assoc.name
                    && lhs_assoc.type_arguments.len() == rhs_assoc.type_arguments.len()
                    && self.matches_type_pattern(&lhs_assoc.base, &rhs_assoc.base, bindings)
                    && lhs_assoc
                        .type_arguments
                        .iter()
                        .zip(&rhs_assoc.type_arguments)
                        .all(|(lhs, rhs)| self.matches_type_pattern(lhs, rhs, bindings))
            }
            _ => pattern == concrete,
        }
    }

    pub fn solve(&mut self) -> Result<(), ConstraintError> {
        for constraint in self.constraints.clone() {
            match constraint {
                TypeConstraint::TraitBound { var, trait_name, span } => {
                    if let Some(ty) = self.substitutions.get(&var) {
                        let ty = self.apply_generic_bindings(ty);
                        if !self.check_trait_bound(&ty, &trait_name)? {
                            return Err(ConstraintError::trait_not_implemented(&format!("{:?}", ty), &display_path(&trait_name), span));
                        }
                    }
                }
                TypeConstraint::Equality { left, right, span } => {
                    self.collect_generic_bindings(&left, &right, span.clone())?;
                    let left = self.apply_generic_bindings(&left);
                    let right = self.apply_generic_bindings(&right);
                    self.unify(&left, &right, span)?;
                }
                TypeConstraint::Subtype { sub, sup, span } => {
                    self.collect_generic_bindings(&sub, &sup, span.clone())?;
                    let sub = self.apply_generic_bindings(&sub);
                    let sup = self.apply_generic_bindings(&sup);
                    if !self.is_subtype(&sub, &sup) {
                        return Err(ConstraintError::type_mismatch(sup, sub, span));
                    }
                }
            }
        }

        for constraint in &self.lifetime_constraints {
            if let LifetimeConstraintKind::Outlives { longer, shorter } = &constraint.kind {
                if shorter == &Lifetime::static_lifetime() && longer != &Lifetime::static_lifetime() {
                    return Err(ConstraintError::propagation_failed("lifetime does not outlive 'static", constraint.span.clone()));
                }
            }
        }

        for constraint in self.associated_type_constraints.clone() {
            let base_type = self.apply_generic_bindings(&constraint.base_type);
            let expected = self.apply_generic_bindings(&constraint.expected);
            let Some(actual) = self.resolve_associated_type(&base_type, &constraint.trait_name, &constraint.assoc_name)
            else {
                return Err(ConstraintError::associated_type_not_found(
                    base_type,
                    constraint.trait_name.clone(),
                    constraint.assoc_name.clone(),
                    constraint.span,
                ));
            };
            let actual = self.apply_generic_bindings(&actual);
            self.collect_generic_bindings(&expected, &actual, constraint.span.clone())?;
            let expected = self.apply_generic_bindings(&expected);
            self.unify(&actual, &expected, constraint.span)?;
        }

        self.solve_where_clauses()?;

        Ok(())
    }

    pub fn propagate_to_caller(&mut self, caller_var: TypeVar, constraints: &[TypeConstraint], origin: Option<String>) {
        self.propagator.propagate_to_caller(caller_var.clone(), constraints, origin);
        if let Some(bounds) = self.propagator.get_trait_bounds(&caller_var) {
            self.trait_bounds.insert(caller_var, bounds.clone());
        }
    }

    pub fn suggest_fixes(&self) -> Vec<FixSuggestion> {
        if !self.constraints.is_empty() || !self.trait_bounds.is_empty() {
            vec![FixSuggestion::new("补充 trait 实现或调整类型约束", 1)]
        }
        else {
            vec![]
        }
    }

    pub fn generate_error_report(&self, error: &ConstraintError) -> ConstraintReport {
        let mut report = ConstraintReport::new(error.kind.clone(), error.span.clone(), error.message.clone());
        report.add_suggestion(FixSuggestion::new("检查相关类型与约束", 1));
        if let ConstraintErrorKind::TypeMismatch { expected, found } = &error.kind {
            report.related_types.push(format!("{:?}", expected));
            report.related_types.push(format!("{:?}", found));
        }
        report
    }

    pub fn clear(&mut self) {
        self.constraints.clear();
        self.substitutions.clear();
        self.generic_bindings.clear();
        self.trait_bounds.clear();
        self.trait_impls.clear();
        self.multi_trait_bounds.clear();
        self.where_clauses.clear();
        self.lifetime_constraints.clear();
        self.associated_type_constraints.clear();
        self.associated_type_impls.clear();
        self.unity_types.clear();
    }
}
