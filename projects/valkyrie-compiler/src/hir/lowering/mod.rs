use std::{ops::Range, path::Path};

use crate::{
    hir::{lower_type_expression, overload::resolve_hir_calls, render_type_expression, validate_ast_root, BuiltinTypeAliasScope},
    nyar_bridge::hir_module_to_artifact_plan,
    validation::ControlFlowScheduler,
    ArtifactPartitionPlan, CanonicalTarget,
};
use ordered_float::OrderedFloat;
use valkyrie_parser::{
    ast::{PatternExpression, SubscriptKind},
    AstParser, AttributeItem, BinaryOperator, ClassDeclaration, DeclarationBody, DeclarationStatement, FunctionDeclaration, FunctionParameter,
    GenericParameterDeclaration, ImplyAssociatedConstBinding, ImplyAssociatedTypeBinding, ImplyDeclaration, InheritanceItem, LetStatement,
    LiteralExpression, NamePath as AstNamePath, NamespaceDeclaration, ObjectFieldDeclaration, ObjectMethodDeclaration, ParseError, Statement,
    StringLiteral as AstStringLiteral, StringSegment as AstStringSegment, TermExpression, TraitAssociatedConstDeclaration,
    TraitAssociatedTypeDeclaration, TraitDeclaration, TypeExpression, UnaryOperator, UniteDeclaration, UniteVariantDeclaration, UsingStatement,
    ValkyrieRoot,
};
use valkyrie_types::{
    hir::{
        GenericType, HirArgument, HirAssociatedConst, HirAssociatedConstImpl, HirAssociatedType, HirAssociatedTypeImpl, HirAttribute, HirBlock,
        HirDocumentation, HirEnum, HirExpr, HirExprKind, HirField, HirFunction, HirIdentifier, HirImpl, HirKind, HirLiteral, HirMatchArm,
        HirModule, HirParam, HirParent, HirPattern, HirProperty, HirStatement, HirStatementKind, HirStruct, HirTrait, HirVariant,
        HirVisibility, HirWhereConstraint, ValkyrieType,
    },
    Identifier, NamePath, SourceID, SourceSpan,
};

mod expr_lowering;

pub use super::CaptureAnalyzer;
use expr_lowering::{extract_name_path, lower_block, lower_term_expression};

/// Minimal compiler facade that lowers parser output into HIR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValkyrieCompiler {
    /// Source id attached to synthesized spans during lowering.
    pub source_id: SourceID,
}

impl Default for ValkyrieCompiler {
    fn default() -> Self {
        Self::new(SourceID::default())
    }
}

impl ValkyrieCompiler {
    /// Creates a compiler facade bound to a source id.
    pub fn new(source_id: SourceID) -> Self {
        Self { source_id }
    }

    /// Parses source text and lowers it into a minimal HIR module.
    pub fn compile_source(&self, source: &str) -> Result<HirModule, ParseError> {
        let root = AstParser::parse_root(source)?;
        let hir = self.lower_root(&root)?;
        ControlFlowScheduler::validate_hir_module(&hir)?;
        Ok(hir)
    }

    /// Parses a source file and lowers it into a minimal HIR module.
    pub fn compile_path(&self, path: &Path) -> Result<HirModule, ParseError> {
        let root = AstParser::parse_path(&path.to_path_buf())?;
        let hir = self.lower_root(&root)?;
        ControlFlowScheduler::validate_hir_module(&hir)?;
        Ok(hir)
    }

    /// Parses source text and lowers it into a minimal `nyar` artifact plan.
    pub fn compile_source_to_artifact_plan(&self, source: &str, target: CanonicalTarget) -> Result<ArtifactPartitionPlan, ParseError> {
        let hir = self.compile_source(source)?;
        Ok(hir_module_to_artifact_plan(&hir, target))
    }

    /// Parses a source file and lowers it into a minimal `nyar` artifact plan.
    pub fn compile_path_to_artifact_plan(&self, path: &Path, target: CanonicalTarget) -> Result<ArtifactPartitionPlan, ParseError> {
        let hir = self.compile_path(path)?;
        Ok(hir_module_to_artifact_plan(&hir, target))
    }

    /// Lowers parser output into a HIR module.
    pub fn lower_root(&self, root: &ValkyrieRoot) -> Result<HirModule, ParseError> {
        AstToHir::new(self.source_id).lower_root(root)
    }
}

/// Lowers `ValkyrieRoot` into `HirModule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AstToHir {
    /// Source id attached to lowered items.
    pub source_id: SourceID,
}

impl AstToHir {
    /// Creates a new lowerer bound to a source id.
    pub fn new(source_id: SourceID) -> Self {
        Self { source_id }
    }

    /// Lowers a parser root into a module-shaped HIR view.
    pub fn lower_root(&self, root: &ValkyrieRoot) -> Result<HirModule, ParseError> {
        validate_ast_root(root)?;
        let _builtin_type_alias_scope = BuiltinTypeAliasScope::enter(root);
        let module_name = root
            .statements
            .iter()
            .find_map(|statement| match statement {
                DeclarationStatement::Namespace(NamespaceDeclaration { name, .. }) => Some(lower_name_path(name)),
                _ => None,
            })
            .unwrap_or_else(default_module_name);

        let imports = root
            .statements
            .iter()
            .filter_map(|statement| match statement {
                DeclarationStatement::Using(UsingStatement { path, .. }) => Some(lower_name_path(path)),
                _ => None,
            })
            .collect();

        let functions = root
            .statements
            .iter()
            .filter_map(|statement| match statement {
                DeclarationStatement::Function(function) => Some(self.lower_function(function)),
                DeclarationStatement::Namespace(namespace) => namespace.body.as_ref().and_then(|body| {
                    body.statements.iter().find_map(|stmt| match stmt {
                        Statement::Function { function, .. } => Some(self.lower_function(function)),
                        _ => None,
                    })
                }),
                _ => None,
            })
            .collect();

        let structs = root
            .statements
            .iter()
            .scan(Vec::<Identifier>::new(), |current_namespace, statement| {
                // `namespace foo;`（body=None）更新当前命名空间上下文。
                if let DeclarationStatement::Namespace(NamespaceDeclaration { name, body: None, .. }) = statement {
                    *current_namespace = name.parts.iter().map(|p| Identifier::new(p.as_str())).collect();
                }
                Some((current_namespace.clone(), statement))
            })
            .filter_map(|(namespace, statement)| match statement {
                DeclarationStatement::Class(class_decl) => Some(self.lower_class(class_decl, &namespace)),
                _ => None,
            })
            .collect();

        let traits = root
            .statements
            .iter()
            .filter_map(|statement| match statement {
                DeclarationStatement::Trait(trait_decl) => Some(self.lower_trait(trait_decl)),
                _ => None,
            })
            .collect();

        let enums = root
            .statements
            .iter()
            .filter_map(|statement| match statement {
                DeclarationStatement::Unite(unite_decl) => Some(self.lower_unite(unite_decl)),
                _ => None,
            })
            .collect();

        let impls = root
            .statements
            .iter()
            .filter_map(|statement| match statement {
                DeclarationStatement::Imply(imply_decl) => Some(self.lower_imply(imply_decl)),
                _ => None,
            })
            .collect();

        let mut hir = HirModule {
            name: module_name,
            doc: HirDocumentation::default(),
            imports,
            submodules: Vec::new(),
            functions,
            structs,
            enums,
            flags: Vec::new(),
            traits,
            impls,
            type_functions: Vec::new(),
            type_families: Vec::new(),
            widgets: Vec::new(),
            singletons: Vec::new(),
            statements: Vec::new(),
        };
        resolve_hir_calls(&mut hir);
        Ok(hir)
    }

    fn lower_function(&self, function: &FunctionDeclaration) -> HirFunction {
        HirFunction {
            name: function.name.name.clone(),
            doc: lower_documentation(&function.annotations),
            annotations: function
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, function.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: function.params.iter().map(|param| lower_param(param, self.source_id, function.span.clone())).collect(),
            return_type: function.return_type.as_ref().map(lower_type_expression).unwrap_or(ValkyrieType::Unit),
            body: lower_block(function.body.as_ref(), self.source_id, function.span.clone()),
            span: with_source(&function.span, self.source_id),
            visibility: lower_visibility(&function.annotations),
            is_abstract: function.body.is_none() || has_modifier(&function.annotations, "abstract"),
            is_final: has_modifier(&function.annotations, "final"),
        }
    }

    fn lower_class(&self, class_decl: &ClassDeclaration, namespace: &[Identifier]) -> HirStruct {
        HirStruct {
            name: class_decl.name.name.clone(),
            namespace: namespace.to_vec(),
            doc: lower_documentation(&class_decl.annotations),
            generics: Vec::new(),
            parents: class_decl.inheritance.iter().map(lower_parent).collect(),
            fields: class_decl.body.fields.iter().map(lower_field).collect(),
            methods: class_decl
                .body
                .methods
                .iter()
                .filter(|method| !is_property_accessor(method))
                .map(|method| self.lower_object_method(method))
                .collect(),
            properties: self.lower_object_properties(&class_decl.body.methods),
            visibility: lower_visibility(&class_decl.annotations),
            is_value_type: class_decl.is_value_type,
            is_abstract: has_modifier(&class_decl.annotations, "abstract"),
            is_sealed: has_modifier(&class_decl.annotations, "sealed"),
            is_final: has_modifier(&class_decl.annotations, "final"),
            is_open: has_modifier(&class_decl.annotations, "open"),
            abstract_methods: Vec::new(),
            abstract_properties: Vec::new(),
            derives: lower_derives(&class_decl.annotations),
        }
    }

    fn lower_trait(&self, trait_decl: &TraitDeclaration) -> HirTrait {
        let methods: Vec<HirFunction> = trait_decl
            .body
            .methods
            .iter()
            .filter(|method| !is_property_accessor(method) && method.body.is_none())
            .map(|method| self.lower_object_method(method))
            .collect();
        let default_methods: Vec<HirFunction> = trait_decl
            .body
            .methods
            .iter()
            .filter(|method| !is_property_accessor(method) && method.body.is_some())
            .map(|method| self.lower_object_method(method))
            .collect();

        HirTrait {
            name: trait_decl.name.name.clone(),
            doc: lower_documentation(&trait_decl.annotations),
            generics: Vec::new(),
            methods,
            associated_types: trait_decl.body.associated_types.iter().map(|item| lower_trait_associated_type(item, self.source_id)).collect(),
            associated_constants: trait_decl
                .body
                .associated_constants
                .iter()
                .map(|item| lower_trait_associated_const(item, self.source_id))
                .collect(),
            super_traits: if trait_decl.is_alias {
                trait_decl.alias_targets.iter().map(lower_named_type).collect()
            }
            else {
                trait_decl.inheritance.iter().map(lower_named_type).collect()
            },
            default_methods,
            visibility: lower_visibility(&trait_decl.annotations),
        }
    }

    fn lower_unite(&self, unite_decl: &UniteDeclaration) -> HirEnum {
        let mut enum_def = HirEnum::new_unity(unite_decl.name.name.clone());
        enum_def.doc = lower_documentation(&unite_decl.annotations);
        enum_def.visibility = lower_visibility(&unite_decl.annotations);
        enum_def.variants = unite_decl.variants.iter().map(lower_unite_variant).collect();
        enum_def
    }

    fn lower_imply(&self, imply_decl: &ImplyDeclaration) -> HirImpl {
        HirImpl {
            generics: lower_imply_generics(imply_decl),
            where_constraints: lower_imply_where_constraints(imply_decl, self.source_id),
            target: lower_type_expression(&imply_decl.target_type),
            trait_path: imply_decl.trait_type.as_ref().map(lower_trait_path),
            methods: imply_decl.methods.iter().map(|method| self.lower_object_method(method)).collect(),
            associated_type_impls: imply_decl
                .associated_type_bindings
                .iter()
                .map(|binding| lower_imply_associated_type_binding(binding, self.source_id))
                .collect(),
            associated_const_impls: imply_decl
                .associated_const_bindings
                .iter()
                .map(|binding| lower_imply_associated_const_binding(binding, self.source_id))
                .collect(),
        }
    }

    fn lower_object_method(&self, method: &ObjectMethodDeclaration) -> HirFunction {
        HirFunction {
            name: method.name.name.clone(),
            doc: lower_documentation(&method.annotations),
            annotations: method
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, method.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: lower_method_params(method, self.source_id),
            return_type: method.return_type.as_ref().map(lower_type_expression).unwrap_or(ValkyrieType::Unit),
            body: lower_block(method.body.as_ref(), self.source_id, method.span.clone()),
            span: with_source(&method.span, self.source_id),
            visibility: lower_visibility(&method.annotations),
            is_abstract: method.body.is_none() || has_modifier(&method.annotations, "abstract"),
            is_final: has_modifier(&method.annotations, "final"),
        }
    }

    fn lower_object_properties(&self, methods: &[ObjectMethodDeclaration]) -> Vec<HirProperty> {
        let mut lowered = Vec::new();

        for method in methods.iter().filter(|method| is_property_accessor(method)) {
            let Some(accessor_kind) = property_accessor_kind(method)
            else {
                continue;
            };
            let accessor = self.lower_property_accessor(method, accessor_kind);
            let ty = lower_property_type(method, accessor_kind);

            if let Some(existing) = lowered.iter_mut().find(|item: &&mut HirProperty| item.name == method.name.name) {
                existing.ty = ty;
                existing.doc = lower_documentation(&method.annotations);
                existing.visibility = lower_visibility(&method.annotations);
                existing.is_abstract = existing.is_abstract || property_is_abstract(method);
                existing.is_final = existing.is_final || property_is_final(method);
                existing.is_static = existing.is_static || property_is_static(method);
                existing.is_virtual = existing.is_virtual || property_is_virtual(method);
                existing.is_override = existing.is_override || property_is_override(method);
                existing.is_lazy = existing.is_lazy || property_is_lazy(method);
                match accessor_kind {
                    PropertyMethodKind::Get => {
                        existing.getter = Some(accessor);
                    }
                    PropertyMethodKind::Set => {
                        existing.setter = Some(accessor);
                        existing.is_readonly = false;
                    }
                }
                continue;
            }

            let mut hir_property = HirProperty {
                name: method.name.name.clone(),
                doc: lower_documentation(&method.annotations),
                ty,
                getter: None,
                setter: None,
                is_readonly: accessor_kind == PropertyMethodKind::Get,
                visibility: lower_visibility(&method.annotations),
                is_abstract: property_is_abstract(method),
                is_final: property_is_final(method),
                is_static: property_is_static(method),
                is_virtual: property_is_virtual(method),
                is_override: property_is_override(method),
                is_lazy: property_is_lazy(method),
                lazy_backing_field: None,
            };

            match accessor_kind {
                PropertyMethodKind::Get => {
                    hir_property.getter = Some(accessor);
                }
                PropertyMethodKind::Set => {
                    hir_property.setter = Some(accessor);
                    hir_property.is_readonly = false;
                }
            }

            lowered.push(hir_property);
        }

        lowered
    }

    fn lower_property_accessor(&self, method: &ObjectMethodDeclaration, accessor_kind: PropertyMethodKind) -> HirFunction {
        let accessor_name = match accessor_kind {
            PropertyMethodKind::Get => method.name.name.clone(),
            PropertyMethodKind::Set => Identifier::new(&format!("set_{}", method.name.as_str())),
        };

        HirFunction {
            name: accessor_name,
            doc: lower_documentation(&method.annotations),
            annotations: method
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, method.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: lower_property_params(method, self.source_id),
            return_type: method.return_type.as_ref().map(lower_type_expression).unwrap_or(ValkyrieType::Unit),
            body: lower_block(method.body.as_ref(), self.source_id, method.span.clone()),
            span: with_source(&method.span, self.source_id),
            visibility: lower_visibility(&method.annotations),
            is_abstract: method.body.is_none() || has_modifier(&method.annotations, "abstract"),
            is_final: has_modifier(&method.annotations, "final"),
        }
    }
}

fn lower_unite_variant(variant: &UniteVariantDeclaration) -> HirVariant {
    HirVariant {
        name: variant.name.name.clone(),
        doc: lower_documentation(&variant.annotations),
        fields: variant.fields.iter().map(lower_field).collect(),
        tuple_types: variant.tuple_types.iter().map(lower_type_expression).collect(),
        result_type: variant.result_type.as_ref().map(lower_type_expression),
    }
}

fn lower_trait_associated_type(item: &TraitAssociatedTypeDeclaration, source_id: SourceID) -> HirAssociatedType {
    HirAssociatedType {
        name: item.name.name.clone(),
        doc: lower_documentation(&item.annotations),
        type_params: Vec::new(),
        bounds: item.bounds.iter().map(lower_type_expression).collect(),
        default: item.default_type.as_ref().map(lower_type_expression),
        span: with_source(&item.span, source_id),
    }
}

fn lower_trait_associated_const(item: &TraitAssociatedConstDeclaration, source_id: SourceID) -> HirAssociatedConst {
    HirAssociatedConst {
        name: item.name.name.clone(),
        doc: lower_documentation(&item.annotations),
        const_type: lower_type_expression(&item.const_type),
        default_value: item.default_value.as_ref().map(|value| lower_term_expression(value, source_id, item.span.clone())),
        span: with_source(&item.span, source_id),
    }
}

fn lower_imply_associated_type_binding(item: &ImplyAssociatedTypeBinding, source_id: SourceID) -> HirAssociatedTypeImpl {
    HirAssociatedTypeImpl {
        name: item.name.name.clone(),
        concrete_type: lower_type_expression(&item.concrete_type),
        type_args: Vec::new(),
        span: with_source(&item.span, source_id),
    }
}

fn lower_imply_associated_const_binding(item: &ImplyAssociatedConstBinding, source_id: SourceID) -> HirAssociatedConstImpl {
    HirAssociatedConstImpl {
        name: item.name.name.clone(),
        const_type: item.const_type.as_ref().map(lower_type_expression),
        value: lower_term_expression(&item.value, source_id, item.span.clone()),
        span: with_source(&item.span, source_id),
    }
}

fn lower_attribute(attribute: &AttributeItem, source_id: SourceID, fallback_span: Range<usize>) -> HirAttribute {
    let arguments = attribute
        .arguments
        .iter()
        .map(|argument| HirArgument {
            key: argument.key.as_deref().map(Identifier::new),
            value: Box::new(lower_attribute_argument_expression(&argument.value, source_id, fallback_span.clone())),
        })
        .collect();
    HirAttribute::with_arguments(lower_name_path(&attribute.name), arguments)
}

fn lower_attribute_argument_expression(expr: &TermExpression, source_id: SourceID, fallback_span: Range<usize>) -> HirExpr {
    match expr {
        TermExpression::Name { path, span } => HirExpr { kind: HirExprKind::Path(lower_name_path(path)), span: with_source(span, source_id) },
        _ => lower_term_expression(expr, source_id, fallback_span),
    }
}

fn lower_documentation(annotations: &valkyrie_parser::Annotations) -> HirDocumentation {
    HirDocumentation::from_lines(annotations.documents.clone())
}

fn lower_visibility(annotations: &valkyrie_parser::Annotations) -> HirVisibility {
    if has_modifier(annotations, "private") {
        HirVisibility::private()
    }
    else if has_modifier(annotations, "protected") {
        HirVisibility::protected()
    }
    else if has_modifier(annotations, "internal") {
        HirVisibility::internal()
    }
    else {
        HirVisibility::public()
    }
}

fn has_modifier(annotations: &valkyrie_parser::Annotations, name: &str) -> bool {
    annotations.modifiers.iter().any(|modifier| modifier.as_str() == name)
}

fn lower_derives(annotations: &valkyrie_parser::Annotations) -> Vec<NamePath> {
    annotations
        .attributes()
        .find(|attribute| attribute.name.parts.last().is_some_and(|name| name == "derive"))
        .map(|attribute| attribute.arguments.iter().filter_map(|argument| extract_name_path(&argument.value)).collect())
        .unwrap_or_default()
}

fn lower_parent(item: &InheritanceItem) -> HirParent {
    match &item.base_type {
        TypeExpression::Path(path) => HirParent::full(
            lower_name_path(&path.name),
            item.alias.as_deref().map(Identifier::new),
            path.arguments.iter().map(lower_type_expression).collect(),
        ),
        other => HirParent::full(
            NamePath::new(vec![Identifier::new(&render_type_expression(other))]),
            item.alias.as_deref().map(Identifier::new),
            Vec::new(),
        ),
    }
}

fn lower_field(field: &ObjectFieldDeclaration) -> HirField {
    HirField {
        name: field.name.name.clone(),
        doc: lower_documentation(&field.annotations),
        ty: lower_type_expression(&field.field_type),
        visibility: lower_visibility(&field.annotations),
        is_readonly: has_modifier(&field.annotations, "readonly"),
    }
}

fn lower_named_type(item: &InheritanceItem) -> ValkyrieType {
    lower_type_expression(&item.base_type)
}

fn lower_trait_path(ty: &TypeExpression) -> NamePath {
    match ty {
        TypeExpression::Path(path) => lower_name_path(&path.name),
        other => NamePath::new(vec![Identifier::new(&render_type_expression(other))]),
    }
}

fn lower_imply_generics(imply_decl: &ImplyDeclaration) -> Vec<GenericType> {
    imply_decl.generic_parameters.iter().map(lower_generic_parameter).collect()
}

fn lower_generic_parameter(parameter: &GenericParameterDeclaration) -> GenericType {
    GenericType {
        name: parameter.name.name.clone(),
        kind: HirKind::Type,
        bounds: parameter.bounds.iter().map(lower_bound_identifier).collect(),
    }
}

fn lower_bound_identifier(bound: &TypeExpression) -> Identifier {
    Identifier::new(&render_type_expression(bound))
}

fn lower_imply_where_constraints(imply_decl: &ImplyDeclaration, source_id: SourceID) -> Vec<HirWhereConstraint> {
    imply_decl
        .where_constraints
        .iter()
        .map(|constraint| HirWhereConstraint {
            target: lower_type_expression(&constraint.target_type),
            bounds: constraint.bounds.iter().map(lower_trait_path).collect(),
            span: with_source(&constraint.span, source_id),
        })
        .collect()
}

fn lower_param(param: &FunctionParameter, source_id: SourceID, fallback_span: Range<usize>) -> HirParam {
    let span = if param.span.is_empty() { fallback_span } else { param.span.clone() };
    HirParam {
        name: HirIdentifier { name: param.name.name.clone(), shadow_index: 0, span: with_source(&span, source_id) },
        ty: param.parameter_type.as_ref().map(lower_type_expression).unwrap_or(ValkyrieType::AutoType),
    }
}

fn lower_method_params(method: &ObjectMethodDeclaration, source_id: SourceID) -> Vec<HirParam> {
    let mut params = Vec::new();
    let has_explicit_self = method.params.first().is_some_and(|param| param.name.as_str() == "self");
    if !has_modifier(&method.annotations, "static") && !has_explicit_self {
        params.push(HirParam {
            name: HirIdentifier { name: Identifier::new("self"), shadow_index: 0, span: with_source(&method.span, source_id) },
            ty: ValkyrieType::r#SelfType,
        });
    }
    params.extend(method.params.iter().map(|param| lower_param(param, source_id, method.span.clone())));
    params
}

fn lower_property_params(method: &ObjectMethodDeclaration, source_id: SourceID) -> Vec<HirParam> {
    let mut params = Vec::new();
    let has_explicit_self = method.params.first().is_some_and(|param| param.name.as_str() == "self");
    if !has_modifier(&method.annotations, "static") && !has_explicit_self {
        params.push(HirParam {
            name: HirIdentifier { name: Identifier::new("self"), shadow_index: 0, span: with_source(&method.span, source_id) },
            ty: ValkyrieType::r#SelfType,
        });
    }
    params.extend(method.params.iter().map(|param| lower_param(param, source_id, method.span.clone())));
    params
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyMethodKind {
    Get,
    Set,
}

fn property_accessor_kind(method: &ObjectMethodDeclaration) -> Option<PropertyMethodKind> {
    if has_modifier(&method.annotations, "get") {
        Some(PropertyMethodKind::Get)
    }
    else if has_modifier(&method.annotations, "set") {
        Some(PropertyMethodKind::Set)
    }
    else {
        None
    }
}

fn is_property_accessor(method: &ObjectMethodDeclaration) -> bool {
    property_accessor_kind(method).is_some()
}

fn lower_property_type(method: &ObjectMethodDeclaration, accessor_kind: PropertyMethodKind) -> ValkyrieType {
    match accessor_kind {
        PropertyMethodKind::Get => method.return_type.as_ref().map(lower_type_expression).unwrap_or(ValkyrieType::Unit),
        PropertyMethodKind::Set => {
            method.params.last().and_then(|param| param.parameter_type.as_ref().map(lower_type_expression)).unwrap_or(ValkyrieType::Unit)
        }
    }
}

fn property_is_abstract(method: &ObjectMethodDeclaration) -> bool {
    method.body.is_none() || has_modifier(&method.annotations, "abstract")
}

fn property_is_final(method: &ObjectMethodDeclaration) -> bool {
    has_modifier(&method.annotations, "final")
}

fn property_is_static(method: &ObjectMethodDeclaration) -> bool {
    has_modifier(&method.annotations, "static")
}

fn property_is_virtual(method: &ObjectMethodDeclaration) -> bool {
    has_modifier(&method.annotations, "virtual")
}

fn property_is_override(method: &ObjectMethodDeclaration) -> bool {
    has_modifier(&method.annotations, "override")
}

fn property_is_lazy(method: &ObjectMethodDeclaration) -> bool {
    has_modifier(&method.annotations, "lazy")
}

fn lower_name_path(path: &AstNamePath) -> NamePath {
    NamePath::new(path.parts.iter().map(|part| Identifier::new(part)).collect())
}

fn default_module_name() -> NamePath {
    NamePath::new(vec![Identifier::new("main")])
}

fn with_source(span: &Range<usize>, source_id: SourceID) -> SourceSpan {
    SourceSpan::new(source_id, span.start as u32, span.end as u32)
}
