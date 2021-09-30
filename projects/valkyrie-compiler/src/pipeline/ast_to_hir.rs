use std::{
    collections::{BTreeMap, BTreeSet},
    ops::Range,
    path::Path,
};

use crate::{
    lir::{LirLowerer, LirModule},
    mir::{MirLowerer, MirModule},
};
use ordered_float::OrderedFloat;
use valkyrie_parser::{
    ast::{MatchPattern, PatternExpression},
    AstParser, AttributeItem, BinaryOperator, ClassDeclaration, DeclarationBody, DeclarationStatement, FunctionDeclaration, FunctionParameter,
    GenericParameterDeclaration, ImplyAssociatedConstBinding, ImplyAssociatedTypeBinding, ImplyDeclaration, InheritanceItem, LetStatement,
    LiteralExpression, NamePath as AstNamePath, NamespaceDeclaration, ObjectBody, ObjectFieldDeclaration, ObjectMethodDeclaration, ParseError,
    Statement, StringLiteral as AstStringLiteral, StringSegment as AstStringSegment, TermExpression, TraitAssociatedConstDeclaration,
    TraitAssociatedTypeDeclaration, TraitDeclaration, TypeExpression, TypePath as AstTypePath, UnaryOperator, UniteDeclaration,
    UniteVariantDeclaration, UsingStatement, ValkyrieRoot,
};
use valkyrie_types::{
    hir::{
        CaptureMode, CaptureStorage, HirArgument, HirAssociatedConst, HirAssociatedConstImpl, HirAssociatedType, HirAssociatedTypeImpl,
        HirAttribute, HirBlock, HirCapture, HirDocumentation, HirEnum, HirExpr, HirExprKind, HirField, HirFunction, HirGeneric, HirIdentifier,
        HirImpl, HirKind, HirLiteral, HirMatchArm, HirModule, HirParam, HirParent, HirPattern, HirProperty, HirStatement, HirStatementKind,
        HirStruct, HirTrait, HirType, HirVariant, HirVisibility, HirWhereConstraint,
    },
    Identifier, NamePath, SourceID, SourceSpan,
};

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
        self.lower_root(&root)
    }

    /// Parses a source file and lowers it into a minimal HIR module.
    pub fn compile_path(&self, path: &Path) -> Result<HirModule, ParseError> {
        let root = AstParser::parse_path(&path.to_path_buf())?;
        self.lower_root(&root)
    }

    /// Lowers parser output into a HIR module.
    pub fn lower_root(&self, root: &ValkyrieRoot) -> Result<HirModule, ParseError> {
        AstToHir::new(self.source_id).lower_root(root)
    }

    /// Lowers parser output into MIR through the current minimal pipeline.
    pub fn lower_root_to_mir(&self, root: &ValkyrieRoot) -> Result<MirModule, ParseError> {
        let hir = self.lower_root(root)?;
        Ok(MirLowerer::lower_module(&hir))
    }

    /// Parses source text and lowers it into MIR.
    pub fn compile_source_to_mir(&self, source: &str) -> Result<MirModule, ParseError> {
        let root = AstParser::parse_root(source)?;
        self.lower_root_to_mir(&root)
    }

    /// Parses source text and lowers it into LIR.
    pub fn compile_source_to_lir(&self, source: &str) -> Result<LirModule, ParseError> {
        let hir = self.compile_source(source)?;
        Ok(LirLowerer::lower_module(&hir))
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
        validate_root(root)?;
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

        Ok(HirModule {
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
        })
    }

    fn lower_function(&self, function: &FunctionDeclaration) -> HirFunction {
        HirFunction {
            name: Identifier::new(&function.name),
            doc: lower_documentation(&function.annotations),
            annotations: function
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, function.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: function.params.iter().map(|param| lower_param(param, self.source_id, function.span.clone())).collect(),
            return_type: function.return_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Unit),
            body: lower_block(function.body.as_ref(), self.source_id, function.span.clone()),
            span: with_source(&function.span, self.source_id),
            visibility: lower_visibility(&function.annotations),
            is_abstract: function.body.is_none() || has_modifier(&function.annotations, "abstract"),
            is_final: has_modifier(&function.annotations, "final"),
        }
    }

    fn lower_class(&self, class_decl: &ClassDeclaration, namespace: &[Identifier]) -> HirStruct {
        HirStruct {
            name: Identifier::new(&class_decl.name),
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
            name: Identifier::new(&trait_decl.name),
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
        let mut enum_def = HirEnum::new_unity(Identifier::new(&unite_decl.name));
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
            name: Identifier::new(&method.name),
            doc: lower_documentation(&method.annotations),
            annotations: method
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, method.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: lower_method_params(method, self.source_id),
            return_type: method.return_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Unit),
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

            if let Some(existing) = lowered.iter_mut().find(|item: &&mut HirProperty| item.name.as_str() == method.name) {
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
                name: Identifier::new(&method.name),
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
            PropertyMethodKind::Get => method.name.clone(),
            PropertyMethodKind::Set => format!("set_{}", method.name),
        };

        HirFunction {
            name: Identifier::new(&accessor_name),
            doc: lower_documentation(&method.annotations),
            annotations: method
                .annotations
                .attributes()
                .map(|attribute| lower_attribute(attribute, self.source_id, method.span.clone()))
                .collect(),
            generics: Vec::new(),
            params: lower_property_params(method, self.source_id),
            return_type: method.return_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Unit),
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
        name: Identifier::new(&variant.name),
        doc: lower_documentation(&variant.annotations),
        fields: variant.fields.iter().map(lower_field).collect(),
        tuple_types: variant.tuple_types.iter().map(lower_type_expression).collect(),
        result_type: variant.result_type.as_ref().map(lower_type_expression),
    }
}

fn lower_trait_associated_type(item: &TraitAssociatedTypeDeclaration, source_id: SourceID) -> HirAssociatedType {
    HirAssociatedType {
        name: Identifier::new(&item.name),
        doc: lower_documentation(&item.annotations),
        type_params: Vec::new(),
        bounds: item.bounds.iter().map(lower_type_expression).collect(),
        default: item.default_type.as_ref().map(lower_type_expression),
        span: with_source(&item.span, source_id),
    }
}

fn lower_trait_associated_const(item: &TraitAssociatedConstDeclaration, source_id: SourceID) -> HirAssociatedConst {
    HirAssociatedConst {
        name: Identifier::new(&item.name),
        doc: lower_documentation(&item.annotations),
        const_type: lower_type_expression(&item.const_type),
        default_value: item.default_value.as_ref().map(|value| lower_term_expression(value, source_id, item.span.clone())),
        span: with_source(&item.span, source_id),
    }
}

fn lower_imply_associated_type_binding(item: &ImplyAssociatedTypeBinding, source_id: SourceID) -> HirAssociatedTypeImpl {
    HirAssociatedTypeImpl {
        name: Identifier::new(&item.name),
        concrete_type: lower_type_expression(&item.concrete_type),
        type_args: Vec::new(),
        span: with_source(&item.span, source_id),
    }
}

fn lower_imply_associated_const_binding(item: &ImplyAssociatedConstBinding, source_id: SourceID) -> HirAssociatedConstImpl {
    HirAssociatedConstImpl {
        name: Identifier::new(&item.name),
        const_type: item.const_type.as_ref().map(lower_type_expression),
        value: lower_term_expression(&item.value, source_id, item.span.clone()),
        span: with_source(&item.span, source_id),
    }
}

/// Tracks values captured from an outer scope.
#[derive(Debug, Default)]
pub struct CaptureAnalyzer {
    bindings: BTreeMap<String, CaptureBinding>,
    captured: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CaptureBinding {
    ty: HirType,
    is_mutable: bool,
}

impl CaptureAnalyzer {
    /// Creates an empty capture analyzer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a visible variable.
    pub fn add_var(&mut self, name: &str, ty: HirType, is_mutable: bool) {
        self.bindings.insert(name.to_string(), CaptureBinding { ty, is_mutable });
    }

    /// Marks a visible variable as captured if it exists in the current bindings.
    pub fn access_var(&mut self, name: &str, _is_write: bool) {
        if self.bindings.contains_key(name) {
            self.captured.insert(name.to_string());
        }
    }

    /// Returns captured variables as HIR captures.
    pub fn into_captures(self) -> Vec<HirCapture> {
        self.captured
            .into_iter()
            .filter_map(|name| {
                let binding = self.bindings.get(&name)?;
                Some(HirCapture {
                    identifier: HirIdentifier {
                        name: Identifier::new(&name),
                        shadow_index: 0,
                        span: SourceSpan::new(SourceID::default(), 0, 0),
                    },
                    ty: binding.ty.clone(),
                    mode: capture_mode(&binding.ty),
                    is_mutable: binding.is_mutable,
                    storage_hint: CaptureStorage::default(),
                })
            })
            .collect()
    }
}

fn lower_attribute(attribute: &AttributeItem, source_id: SourceID, fallback_span: Range<usize>) -> HirAttribute {
    let arguments = attribute
        .arguments
        .iter()
        .map(|argument| HirArgument {
            key: argument.key.as_deref().map(Identifier::new),
            value: Box::new(lower_term_expression(&argument.value, source_id, fallback_span.clone())),
        })
        .collect();
    HirAttribute::with_arguments(lower_name_path(&attribute.name), arguments)
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
    annotations.modifiers.iter().any(|modifier| modifier == name)
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
        name: Identifier::new(&field.name),
        doc: lower_documentation(&field.annotations),
        ty: lower_type_expression(&field.field_type),
        visibility: lower_visibility(&field.annotations),
        is_readonly: has_modifier(&field.annotations, "readonly"),
    }
}

fn lower_named_type(item: &InheritanceItem) -> HirType {
    lower_type_expression(&item.base_type)
}

fn lower_trait_path(ty: &TypeExpression) -> NamePath {
    match ty {
        TypeExpression::Path(path) => lower_name_path(&path.name),
        other => NamePath::new(vec![Identifier::new(&render_type_expression(other))]),
    }
}

fn lower_imply_generics(imply_decl: &ImplyDeclaration) -> Vec<HirGeneric> {
    imply_decl.generic_parameters.iter().map(lower_generic_parameter).collect()
}

fn lower_generic_parameter(parameter: &GenericParameterDeclaration) -> HirGeneric {
    HirGeneric {
        name: Identifier::new(&parameter.name),
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
        name: HirIdentifier { name: Identifier::new(&param.name), shadow_index: 0, span: with_source(&span, source_id) },
        ty: param.parameter_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Infer),
    }
}

fn lower_method_params(method: &ObjectMethodDeclaration, source_id: SourceID) -> Vec<HirParam> {
    let mut params = Vec::new();
    let has_explicit_self = method.params.first().is_some_and(|param| param.name == "self");
    if !has_modifier(&method.annotations, "static") && !has_explicit_self {
        params.push(HirParam {
            name: HirIdentifier { name: Identifier::new("self"), shadow_index: 0, span: with_source(&method.span, source_id) },
            ty: HirType::SelfType,
        });
    }
    params.extend(method.params.iter().map(|param| lower_param(param, source_id, method.span.clone())));
    params
}

fn lower_property_params(method: &ObjectMethodDeclaration, source_id: SourceID) -> Vec<HirParam> {
    let mut params = Vec::new();
    let has_explicit_self = method.params.first().is_some_and(|param| param.name == "self");
    if !has_modifier(&method.annotations, "static") && !has_explicit_self {
        params.push(HirParam {
            name: HirIdentifier { name: Identifier::new("self"), shadow_index: 0, span: with_source(&method.span, source_id) },
            ty: HirType::SelfType,
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

fn lower_property_type(method: &ObjectMethodDeclaration, accessor_kind: PropertyMethodKind) -> HirType {
    match accessor_kind {
        PropertyMethodKind::Get => method.return_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Unit),
        PropertyMethodKind::Set => {
            method.params.last().and_then(|param| param.parameter_type.as_ref().map(lower_type_expression)).unwrap_or(HirType::Unit)
        }
    }
}

fn validate_root(root: &ValkyrieRoot) -> Result<(), ParseError> {
    for statement in &root.statements {
        validate_declaration_statement(statement)?;
    }
    Ok(())
}

fn validate_declaration_statement(statement: &DeclarationStatement) -> Result<(), ParseError> {
    match statement {
        DeclarationStatement::Namespace(namespace) => {
            if let Some(body) = &namespace.body {
                validate_declaration_body(body)?;
            }
        }
        DeclarationStatement::Using(_) => {}
        DeclarationStatement::Function(function) => validate_function_declaration(function)?,
        DeclarationStatement::Class(class_decl) => validate_class_declaration(class_decl)?,
        DeclarationStatement::Trait(trait_decl) => validate_trait_declaration(trait_decl)?,
        DeclarationStatement::Imply(imply_decl) => validate_imply_declaration(imply_decl)?,
        DeclarationStatement::Unite(unite_decl) => validate_unite_declaration(unite_decl)?,
        DeclarationStatement::Attribute(_) => {}
        DeclarationStatement::TypeAlias(_) => {}
    }
    Ok(())
}

fn validate_function_declaration(function: &FunctionDeclaration) -> Result<(), ParseError> {
    for param in &function.params {
        validate_function_parameter(param)?;
    }
    if let Some(return_type) = &function.return_type {
        validate_type_expression(return_type)?;
    }
    if let Some(body) = &function.body {
        validate_declaration_body(body)?;
    }
    Ok(())
}

fn validate_class_declaration(class_decl: &ClassDeclaration) -> Result<(), ParseError> {
    for parent in &class_decl.inheritance {
        validate_type_expression(&parent.base_type)?;
    }
    validate_object_body(&class_decl.body)
}

fn validate_trait_declaration(trait_decl: &TraitDeclaration) -> Result<(), ParseError> {
    for parent in &trait_decl.inheritance {
        validate_type_expression(&parent.base_type)?;
    }
    for target in &trait_decl.alias_targets {
        validate_type_expression(&target.base_type)?;
    }
    validate_object_body(&trait_decl.body)
}

fn validate_imply_declaration(imply_decl: &ImplyDeclaration) -> Result<(), ParseError> {
    for parameter in &imply_decl.generic_parameters {
        validate_generic_parameter(parameter)?;
    }
    validate_type_expression(&imply_decl.target_type)?;
    if let Some(trait_type) = &imply_decl.trait_type {
        validate_type_expression(trait_type)?;
    }
    for constraint in &imply_decl.where_constraints {
        validate_type_expression(&constraint.target_type)?;
        for bound in &constraint.bounds {
            validate_type_expression(bound)?;
        }
    }
    for method in &imply_decl.methods {
        validate_object_method(method)?;
    }
    for binding in &imply_decl.associated_type_bindings {
        for parameter in &binding.generic_parameters {
            validate_generic_parameter(parameter)?;
        }
        validate_type_expression(&binding.concrete_type)?;
    }
    for binding in &imply_decl.associated_const_bindings {
        if let Some(const_type) = &binding.const_type {
            validate_type_expression(const_type)?;
        }
        validate_term_expression(&binding.value)?;
    }
    Ok(())
}

fn validate_unite_declaration(unite_decl: &UniteDeclaration) -> Result<(), ParseError> {
    for variant in &unite_decl.variants {
        for field in &variant.fields {
            validate_object_field(field)?;
        }
        if let Some(result_type) = &variant.result_type {
            validate_type_expression(result_type)?;
        }
    }
    Ok(())
}

fn validate_object_body(body: &ObjectBody) -> Result<(), ParseError> {
    for field in &body.fields {
        validate_object_field(field)?;
    }
    for method in &body.methods {
        validate_object_method(method)?;
    }
    for item in &body.associated_types {
        validate_trait_associated_type(item)?;
    }
    for item in &body.associated_constants {
        validate_trait_associated_const(item)?;
    }
    Ok(())
}

fn validate_object_field(field: &ObjectFieldDeclaration) -> Result<(), ParseError> {
    validate_type_expression(&field.field_type)?;
    if let Some(default_value) = &field.default_value {
        validate_term_expression(default_value)?;
    }
    Ok(())
}

fn validate_object_method(method: &ObjectMethodDeclaration) -> Result<(), ParseError> {
    for param in &method.params {
        validate_function_parameter(param)?;
    }
    if let Some(return_type) = &method.return_type {
        validate_type_expression(return_type)?;
    }
    if let Some(body) = &method.body {
        validate_declaration_body(body)?;
    }
    Ok(())
}

fn validate_trait_associated_type(item: &TraitAssociatedTypeDeclaration) -> Result<(), ParseError> {
    for bound in &item.bounds {
        validate_type_expression(bound)?;
    }
    if let Some(default_type) = &item.default_type {
        validate_type_expression(default_type)?;
    }
    Ok(())
}

fn validate_trait_associated_const(item: &TraitAssociatedConstDeclaration) -> Result<(), ParseError> {
    validate_type_expression(&item.const_type)?;
    if let Some(default_value) = &item.default_value {
        validate_term_expression(default_value)?;
    }
    Ok(())
}

fn validate_generic_parameter(parameter: &GenericParameterDeclaration) -> Result<(), ParseError> {
    for bound in &parameter.bounds {
        validate_type_expression(bound)?;
    }
    if let Some(default_type) = &parameter.default_type {
        validate_type_expression(default_type)?;
    }
    Ok(())
}

fn validate_function_parameter(param: &FunctionParameter) -> Result<(), ParseError> {
    if let Some(parameter_type) = &param.parameter_type {
        validate_type_expression(parameter_type)?;
    }
    Ok(())
}

fn validate_declaration_body(body: &DeclarationBody) -> Result<(), ParseError> {
    for statement in &body.statements {
        validate_statement(statement)?;
    }
    if let Some(tail_expression) = &body.tail_expression {
        validate_term_expression(tail_expression)?;
    }
    Ok(())
}

fn validate_statement(statement: &Statement) -> Result<(), ParseError> {
    match statement {
        Statement::Let { statement, .. } => {
            if let Some(ty) = &statement.ty {
                validate_type_expression(ty)?;
            }
            if let Some(initializer) = &statement.initializer {
                validate_term_expression(initializer)?;
            }
        }
        Statement::Expr { expression, .. } => validate_term_expression(expression)?,
        Statement::Function { function, .. } => validate_function_declaration(function)?,
    }
    Ok(())
}

fn validate_term_expression(expression: &TermExpression) -> Result<(), ParseError> {
    match expression {
        TermExpression::Name { .. } | TermExpression::Literal { .. } | TermExpression::Continue { .. } => {}
        TermExpression::Unary { expr, .. } => validate_term_expression(expr)?,
        TermExpression::Binary { lhs, rhs, .. } => {
            validate_term_expression(lhs)?;
            validate_term_expression(rhs)?;
        }
        TermExpression::Call { callee, args, .. } => {
            validate_term_expression(callee)?;
            for arg in args {
                validate_term_expression(arg)?;
            }
        }
        TermExpression::MemberAccess { object, .. } => validate_term_expression(object)?,
        TermExpression::Subscript { object, index, .. } => {
            validate_term_expression(object)?;
            validate_term_expression(index)?;
        }
        TermExpression::Tuple { items, .. } | TermExpression::Array { items, .. } => {
            for item in items {
                validate_term_expression(item)?;
            }
        }
        TermExpression::As { expr, ty, .. } => {
            validate_term_expression(expr)?;
            validate_type_expression(ty)?;
        }
        TermExpression::Turbofish { expr, arguments, .. } => {
            validate_term_expression(expr)?;
            for argument in arguments {
                validate_type_expression(argument)?;
            }
        }
        TermExpression::Assign { target, value, .. } => {
            validate_term_expression(target)?;
            validate_term_expression(value)?;
        }
        TermExpression::Return { value, .. } | TermExpression::Break { value, .. } => {
            if let Some(value) = value {
                validate_term_expression(value)?;
            }
        }
        TermExpression::If { condition, then_body, else_body, .. } => {
            validate_term_expression(condition)?;
            validate_declaration_body(then_body)?;
            if let Some(else_body) = else_body {
                validate_declaration_body(else_body)?;
            }
        }
        TermExpression::Loop { iterator, condition, body, .. } => {
            if let Some(iterator) = iterator {
                validate_term_expression(iterator)?;
            }
            if let Some(condition) = condition {
                validate_term_expression(condition)?;
            }
            validate_declaration_body(body)?;
        }
        TermExpression::Match { scrutinee, arms, .. } => {
            validate_term_expression(scrutinee)?;
            for arm in arms {
                if let MatchPattern::Constructor { guard: Some(guard_expr), .. } = &arm.pattern {
                    validate_term_expression(guard_expr)?;
                }
                validate_declaration_body(&arm.body)?;
            }
        }
        TermExpression::Construct { path: _, fields, .. } => {
            for (_, value) in fields {
                validate_term_expression(value)?;
            }
        }
        TermExpression::Lambda { params: _, return_type: _, body, .. } => {
            validate_declaration_body(body)?;
        }
        TermExpression::Block { body, .. } => {
            validate_declaration_body(body)?;
        }
    }
    Ok(())
}

fn validate_type_expression(ty: &TypeExpression) -> Result<(), ParseError> {
    match ty {
        TypeExpression::Path(path) => {
            if let Some(name) = path.name.parts.last() {
                if is_legacy_text_type_name(name) {
                    return Err(ParseError::Invalid(format!("legacy text type `{name}` has been removed; use `utf8` or `utf16` explicitly")));
                }
            }
            for argument in &path.arguments {
                validate_type_expression(argument)?;
            }
        }
        TypeExpression::Array { item, .. } => validate_type_expression(item)?,
        TypeExpression::Tuple { items, .. } => {
            for item in items {
                validate_type_expression(item)?;
            }
        }
        TypeExpression::Unit { .. }
        | TypeExpression::SelfType { .. }
        | TypeExpression::Associated { .. }
        | TypeExpression::Nullable { .. }
        | TypeExpression::Function { .. } => {}
    }
    Ok(())
}

fn is_legacy_text_type_name(name: &str) -> bool {
    matches!(name, "string" | "str" | "String")
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

fn lower_type_expression(ty: &TypeExpression) -> HirType {
    match ty {
        TypeExpression::Path(path) => lower_type_path(path),
        TypeExpression::Array { item, .. } => HirType::Array(Box::new(lower_type_expression(item))),
        TypeExpression::Tuple { items, .. } => HirType::Tuple(items.iter().map(lower_type_expression).collect()),
        TypeExpression::Unit { .. } => HirType::Unit,
        TypeExpression::SelfType { .. } => HirType::SelfType,
        // 关联类型绑定在类型参数位置仅作为约束信息，降级为目标类型。
        TypeExpression::Associated { ty, .. } => lower_type_expression(ty),
        // 可空类型 `T?` 降级为内部类型，可空语义由类型检查器处理。
        TypeExpression::Nullable { item, .. } => lower_type_expression(item),
        // 函数类型降级为 HirType::Function。
        TypeExpression::Function { params, return_type, .. } => HirType::Function {
            params: params.iter().map(lower_type_expression).collect(),
            return_type: Box::new(lower_type_expression(return_type)),
        },
    }
}

fn lower_type_path(path: &AstTypePath) -> HirType {
    let last = path.name.parts.last().cloned().unwrap_or_default();
    // 仅识别语言中真实存在的内建类型，禁止偷偷接受历史别名。
    let base = match last.as_str() {
        "i32" => HirType::Integer32,
        "i64" => HirType::Integer64,
        "f32" => HirType::Float32,
        "f64" => HirType::Float64,
        "bool" => HirType::Boolean,
        "utf8" => HirType::Utf8,
        "utf16" => HirType::Utf16,
        "unit" => HirType::Unit,
        "void" => HirType::Void,
        _ => HirType::Named(Identifier::new(&last)),
    };
    if path.arguments.is_empty() {
        base
    }
    else {
        HirType::Apply(Box::new(base), path.arguments.iter().map(lower_type_expression).collect())
    }
}

fn render_type_expression(ty: &TypeExpression) -> String {
    match ty {
        TypeExpression::Path(path) => {
            let base = path.name.parts.join("::");
            if path.arguments.is_empty() {
                base
            }
            else {
                let args = path.arguments.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
                format!("{base}<{args}>")
            }
        }
        TypeExpression::Array { item, .. } => format!("[{}]", render_type_expression(item)),
        TypeExpression::Tuple { items, .. } => {
            let inner = items.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
            format!("({inner})")
        }
        TypeExpression::Unit { .. } => "()".to_string(),
        TypeExpression::SelfType { .. } => "Self".to_string(),
        TypeExpression::Associated { name, ty, .. } => format!("{name}={}", render_type_expression(ty)),
        TypeExpression::Nullable { item, .. } => format!("{}?", render_type_expression(item)),
        TypeExpression::Function { params, return_type, .. } => {
            let params_str = params.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
            format!("micro({params_str}) -> {}", render_type_expression(return_type))
        }
    }
}

fn lower_block(body: Option<&DeclarationBody>, source_id: SourceID, fallback_span: Range<usize>) -> HirBlock {
    let Some(body) = body
    else {
        return HirBlock { statements: Vec::new(), expr: None, span: with_source(&fallback_span, source_id) };
    };

    let statements = body.statements.iter().map(|statement| lower_statement(statement, source_id, fallback_span.clone())).collect();
    let expr = body.tail_expression.as_ref().map(|expr| Box::new(lower_term_expression(expr, source_id, fallback_span.clone())));
    HirBlock { statements, expr, span: with_source(&body.span, source_id) }
}

fn lower_name_path(path: &AstNamePath) -> NamePath {
    NamePath::new(path.parts.iter().map(|part| Identifier::new(part)).collect())
}

fn default_module_name() -> NamePath {
    NamePath::new(vec![Identifier::new("main")])
}

fn lower_statement(statement: &Statement, source_id: SourceID, fallback_span: Range<usize>) -> HirStatement {
    let span_range = if statement.span().is_empty() { fallback_span } else { statement.span().clone() };
    let span = with_source(&span_range, source_id);
    let kind = match statement {
        Statement::Let { statement, .. } => lower_let_statement(statement, source_id, span_range),
        Statement::Expr { expression, .. } => HirStatementKind::Expr(Box::new(lower_term_expression(expression, source_id, span_range))),
        Statement::Function { function, .. } => {
            // Function declarations in block bodies are lowered as top-level functions, not statements.
            // This path should ideally not be reached for well-formed code.
            HirStatementKind::Expr(Box::new(HirExpr {
                kind: HirExprKind::Path(NamePath::new(vec![Identifier::new(&function.name)])),
                span: span.clone(),
            }))
        }
    };
    HirStatement { kind, span }
}

fn lower_let_statement(statement: &LetStatement, source_id: SourceID, fallback_span: Range<usize>) -> HirStatementKind {
    HirStatementKind::Let {
        is_mutable: statement.is_mutable,
        pattern: lower_pattern_expression(&statement.pattern, source_id),
        initializer: statement.initializer.as_ref().map(|expr| Box::new(lower_term_expression(expr, source_id, fallback_span.clone()))),
        ty: statement.ty.as_ref().map(lower_type_expression),
    }
}

fn lower_pattern_expression(pattern: &PatternExpression, source_id: SourceID) -> HirPattern {
    match pattern {
        PatternExpression::Wildcard { .. } => HirPattern::Wildcard,
        PatternExpression::Variable { name, span } => {
            HirPattern::Variable(HirIdentifier { name: Identifier::new(name), shadow_index: 0, span: with_source(span, source_id) })
        }
        PatternExpression::Tuple { items, .. } => {
            HirPattern::Tuple(items.iter().map(|item| lower_pattern_expression(item, source_id)).collect())
        }
    }
}

fn lower_term_expression(expression: &TermExpression, source_id: SourceID, fallback_span: Range<usize>) -> HirExpr {
    lower_term_expression_with_context(expression, source_id, fallback_span, false)
}

fn lower_term_expression_with_context(
    expression: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    preserve_member_access: bool,
) -> HirExpr {
    let span_range = if expression.span().is_empty() { fallback_span } else { expression.span().clone() };
    let span = with_source(&span_range, source_id);
    let kind = match expression {
        TermExpression::Name { path, .. } => lower_name_expression(path, span.clone()),
        TermExpression::Literal { literal, .. } => lower_literal_expression(literal, source_id, span_range.clone()),
        TermExpression::Unary { op, expr, .. } => lower_method_call_kind(
            unary_operator_method_name(op),
            vec![lower_term_expression_with_context(expr, source_id, span_range.clone(), false)],
            span.clone(),
        ),
        TermExpression::Binary { op, lhs, rhs, .. } => lower_binary_expression(op, lhs, rhs, source_id, span_range.clone(), span.clone()),
        TermExpression::Call { callee, args, .. } => lower_call_expression(callee, args, source_id, span_range.clone(), span.clone()),
        TermExpression::MemberAccess { object, member, .. } => {
            let object = lower_term_expression_with_context(object, source_id, span_range.clone(), false);
            if preserve_member_access {
                lower_method_call_kind(member, vec![object], span.clone())
            }
            else {
                // 字段访问：降级为 FieldAccess 而非 get_ 方法调用。
                HirExprKind::FieldAccess { object: Box::new(object), field: Identifier::new(member) }
            }
        }
        TermExpression::Subscript { object, index, .. } => HirExprKind::Subscript {
            object: Box::new(lower_term_expression_with_context(object, source_id, span_range.clone(), false)),
            index: Box::new(lower_term_expression_with_context(index, source_id, span_range.clone(), false)),
        },
        TermExpression::Tuple { items, .. } => HirExprKind::Call {
            callee: Box::new(HirExpr { kind: HirExprKind::Path(NamePath::new(vec![Identifier::new("tuple")])), span: span.clone() }),
            args: items.iter().map(|item| lower_term_expression_with_context(item, source_id, span_range.clone(), false)).collect(),
        },
        TermExpression::Array { items, .. } => HirExprKind::Call {
            callee: Box::new(HirExpr { kind: HirExprKind::Path(NamePath::new(vec![Identifier::new("array")])), span: span.clone() }),
            args: items.iter().map(|item| lower_term_expression_with_context(item, source_id, span_range.clone(), false)).collect(),
        },
        TermExpression::Turbofish { expr, arguments, .. } => HirExprKind::GenericApply {
            callee: Box::new(lower_term_expression_with_context(expr, source_id, span_range.clone(), preserve_member_access)),
            arguments: arguments.iter().map(lower_type_expression).collect(),
        },
        TermExpression::Assign { target, value, .. } => lower_assignment_expression(target, value, source_id, span_range.clone(), span.clone()),
        TermExpression::As { expr, .. } => lower_term_expression_with_context(expr, source_id, span_range.clone(), false).kind,
        TermExpression::Loop { pattern, iterator, condition, body, .. } => HirExprKind::Loop {
            label: None,
            pattern: pattern.as_ref().map(|pat| lower_pattern_expression(pat, source_id)),
            iterator: iterator.as_ref().map(|expr| Box::new(lower_term_expression_with_context(expr, source_id, span_range.clone(), false))),
            condition: condition.as_ref().map(|expr| Box::new(lower_term_expression_with_context(expr, source_id, span_range.clone(), false))),
            body: Box::new(lower_block(Some(body), source_id, span_range.clone())),
        },
        TermExpression::Return { value, .. } => HirExprKind::Return(
            value.as_ref().map(|expr| Box::new(lower_term_expression_with_context(expr, source_id, span_range.clone(), false))),
        ),
        TermExpression::Break { value, .. } => HirExprKind::Break {
            label: None,
            expr: value.as_ref().map(|expr| Box::new(lower_term_expression_with_context(expr, source_id, span_range.clone(), false))),
        },
        TermExpression::Continue { .. } => HirExprKind::Continue { label: None },
        TermExpression::If { condition, then_body, else_body, .. } => HirExprKind::If {
            condition: Box::new(lower_term_expression_with_context(condition, source_id, span_range.clone(), false)),
            then_branch: Box::new(lower_block(Some(then_body), source_id, span_range.clone())),
            else_branch: else_body.as_ref().map(|body| Box::new(lower_block(Some(body), source_id, span_range.clone()))),
        },
        TermExpression::Match { scrutinee, arms, .. } => {
            let scrutinee = Box::new(lower_term_expression_with_context(scrutinee, source_id, span_range.clone(), false));
            let arms = arms
                .iter()
                .map(|arm| {
                    let pattern = match &arm.pattern {
                        MatchPattern::Constructor { name, bindings, .. } => {
                            let fields: Vec<HirPattern> = bindings
                                .iter()
                                .map(|binding| {
                                    HirPattern::Variable(HirIdentifier { name: Identifier::new(binding), shadow_index: 0, span: span.clone() })
                                })
                                .collect();
                            let constructor_name =
                                name.parts.last().map(|s| Identifier::new(s.as_str())).unwrap_or_else(|| Identifier::new("_"));
                            HirPattern::Constructor { name: constructor_name, fields }
                        }
                        MatchPattern::Default { .. } => HirPattern::Else,
                    };
                    let guard = match &arm.pattern {
                        MatchPattern::Constructor { guard: Some(guard_expr), .. } => {
                            Some(Box::new(lower_term_expression_with_context(guard_expr, source_id, span_range.clone(), false)))
                        }
                        _ => None,
                    };
                    let body_block = lower_block(Some(&arm.body), source_id, span_range.clone());
                    let body = Box::new(HirExpr { kind: HirExprKind::Block(Box::new(body_block)), span: span.clone() });
                    HirMatchArm { pattern, guard, body }
                })
                .collect();
            HirExprKind::Match { scrutinee, arms }
        }
        TermExpression::Construct { path, fields, .. } => {
            // 结构体构造表达式：保留字段名，降级为 Construct + FieldInit。
            let name = path.parts.last().map(|s| Identifier::new(s.as_str())).unwrap_or_else(|| Identifier::new("_"));
            let args = fields
                .iter()
                .map(|(field_name, value)| HirExpr {
                    kind: HirExprKind::FieldInit {
                        name: Identifier::new(field_name),
                        value: Box::new(lower_term_expression_with_context(value, source_id, span_range.clone(), false)),
                    },
                    span: span.clone(),
                })
                .collect();
            HirExprKind::Construct { name, args }
        }
        TermExpression::Lambda { params, return_type, body, .. } => {
            // Lambda 表达式：降级为 HIR Lambda，参数使用 lower_param，函数体使用 lower_block。
            let hir_params = params.iter().map(|param| lower_param(param, source_id, span_range.clone())).collect();
            let hir_return_type = return_type.as_ref().map(lower_type_expression).unwrap_or(HirType::Infer);
            let hir_body = lower_block(Some(body), source_id, span_range.clone());
            HirExprKind::Lambda { generics: Vec::new(), params: hir_params, return_type: hir_return_type, body: Box::new(hir_body) }
        }
        TermExpression::Block { body, .. } => {
            // 块表达式（如 `unsafe { ... }`）：降级为 HIR Block。
            let hir_block = lower_block(Some(body), source_id, span_range.clone());
            HirExprKind::Block(Box::new(hir_block))
        }
    };
    HirExpr { kind, span }
}

fn lower_assignment_expression(
    target: &TermExpression,
    value: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    span: SourceSpan,
) -> HirExprKind {
    let value = lower_term_expression_with_context(value, source_id, fallback_span.clone(), false);
    match target {
        TermExpression::MemberAccess { object, member, .. } => {
            // 字段赋值：降级为 StoreField 而非 set_ 方法调用。
            let object = lower_term_expression_with_context(object, source_id, fallback_span, false);
            HirExprKind::StoreField { object: Box::new(object), field: Identifier::new(member), value: Box::new(value) }
        }
        TermExpression::Subscript { object, index, .. } => {
            // 数组元素赋值 `arr[i] = value` 降级为 StoreSubscript。
            let object = lower_term_expression_with_context(object, source_id, fallback_span.clone(), false);
            let index = lower_term_expression_with_context(index, source_id, fallback_span, false);
            HirExprKind::StoreSubscript { object: Box::new(object), index: Box::new(index), value: Box::new(value) }
        }
        TermExpression::Name { path, .. } if path.parts.len() == 1 => {
            HirExprKind::Assign { target: Identifier::new(&path.parts[0]), value: Box::new(value) }
        }
        _ => HirExprKind::Call {
            callee: Box::new(HirExpr {
                kind: HirExprKind::Path(NamePath::new(vec![Identifier::new("unsupported_assignment")])),
                span: span.clone(),
            }),
            args: vec![value],
        },
    }
}

fn lower_call_expression(
    callee: &TermExpression,
    args: &[TermExpression],
    source_id: SourceID,
    fallback_span: Range<usize>,
    span: SourceSpan,
) -> HirExprKind {
    // 识别内建数组创建函数 `__newarr_<type>(length)`，降级为 ArrayNew。
    // 这是编写 PE 二进制写入器的必要前置能力。
    // 必须在点分路径检查之前执行，否则 `__newarr_u8` 会被误判为普通路径调用。
    if let TermExpression::Name { path, .. } = callee {
        if path.parts.len() == 1 {
            let name = path.parts[0].as_str();
            if let Some(element_type) = parse_newarr_builtin(name) {
                if args.len() == 1 {
                    return HirExprKind::ArrayNew {
                        element_type,
                        length: Box::new(lower_term_expression_with_context(&args[0], source_id, fallback_span.clone(), false)),
                    };
                }
            }
        }
    }

    if let TermExpression::MemberAccess { object, member, .. } = callee {
        if is_self_rooted_member_chain(object) {
            let mut lowered_args = vec![lower_term_expression_with_context(object, source_id, fallback_span.clone(), false)];
            lowered_args.extend(args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)));
            return lower_method_call_kind(member, lowered_args, span);
        }
    }

    if let TermExpression::Turbofish { expr, arguments, .. } = callee {
        if let TermExpression::MemberAccess { object, member, .. } = expr.as_ref() {
            if is_self_rooted_member_chain(object) {
                let mut lowered_args = vec![lower_term_expression_with_context(object, source_id, fallback_span.clone(), false)];
                lowered_args.extend(args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)));
                return HirExprKind::Call {
                    callee: Box::new(HirExpr {
                        kind: HirExprKind::GenericApply {
                            callee: Box::new(HirExpr {
                                kind: HirExprKind::Path(NamePath::new(vec![Identifier::new(member)])),
                                span: span.clone(),
                            }),
                            arguments: arguments.iter().map(lower_type_expression).collect(),
                        },
                        span: span.clone(),
                    }),
                    args: lowered_args,
                };
            }
        }
    }

    // 先尝试将 callee 解释为点分路径（如 `std.io.print_line`）。
    // 如果整个链都是 Name/MemberAccess，视为路径调用而非方法调用，
    // 避免将模块路径 `std.io` 误降级为字段访问。
    if let Some(parts) = extract_dotted_path(callee) {
        return HirExprKind::Call {
            callee: Box::new(HirExpr {
                kind: HirExprKind::Path(NamePath::new(parts.iter().map(|p| Identifier::new(p.as_str())).collect())),
                span: span.clone(),
            }),
            args: args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)).collect(),
        };
    }

    if let TermExpression::MemberAccess { object, member, .. } = callee {
        let mut lowered_args = vec![lower_term_expression_with_context(object, source_id, fallback_span.clone(), false)];
        lowered_args.extend(args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)));
        return lower_method_call_kind(member, lowered_args, span);
    }

    if let TermExpression::Turbofish { expr, arguments, .. } = callee {
        if let TermExpression::MemberAccess { object, member, .. } = expr.as_ref() {
            let mut lowered_args = vec![lower_term_expression_with_context(object, source_id, fallback_span.clone(), false)];
            lowered_args.extend(args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)));
            return HirExprKind::Call {
                callee: Box::new(HirExpr {
                    kind: HirExprKind::GenericApply {
                        callee: Box::new(HirExpr { kind: HirExprKind::Path(NamePath::new(vec![Identifier::new(member)])), span: span.clone() }),
                        arguments: arguments.iter().map(lower_type_expression).collect(),
                    },
                    span: span.clone(),
                }),
                args: lowered_args,
            };
        }
    }

    // 识别内建数组创建函数 `__newarr_<type>(length)`，降级为 ArrayNew。
    // 这是编写 PE 二进制写入器的必要前置能力。
    // （已移至函数开头，在点分路径检查之前执行。）

    HirExprKind::Call {
        callee: Box::new(lower_term_expression_with_context(callee, source_id, fallback_span.clone(), true)),
        args: args.iter().map(|arg| lower_term_expression_with_context(arg, source_id, fallback_span.clone(), false)).collect(),
    }
}

/// 解析内建数组创建函数名，返回对应的元素类型。
///
/// 支持 `__newarr_u8`、`__newarr_i32` 等命名约定，
/// 将其映射为对应的 `HirType`。
fn parse_newarr_builtin(name: &str) -> Option<HirType> {
    let suffix = name.strip_prefix("__newarr_")?;
    Some(match suffix {
        "u8" | "byte" => HirType::Named(Identifier::new("u8")),
        "u16" | "ushort" => HirType::Named(Identifier::new("u16")),
        "u32" | "uint" => HirType::Named(Identifier::new("u32")),
        "u64" | "ulong" => HirType::Named(Identifier::new("u64")),
        "i8" | "sbyte" => HirType::Named(Identifier::new("i8")),
        "i16" | "short" => HirType::Named(Identifier::new("i16")),
        "i32" | "int" => HirType::Integer32,
        "i64" | "long" => HirType::Integer64,
        "f32" | "float" => HirType::Float32,
        "f64" | "double" => HirType::Float64,
        "bool" | "boolean" => HirType::Boolean,
        "char" => HirType::Named(Identifier::new("char")),
        "utf8" | "string" => HirType::Utf8,
        _ => return None,
    })
}

fn lower_method_call_kind(member: &str, args: Vec<HirExpr>, span: SourceSpan) -> HirExprKind {
    HirExprKind::Call {
        callee: Box::new(HirExpr { kind: HirExprKind::Path(NamePath::new(vec![Identifier::new(member)])), span: span.clone() }),
        args,
    }
}

fn lower_binary_expression(
    op: &BinaryOperator,
    lhs: &TermExpression,
    rhs: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    span: SourceSpan,
) -> HirExprKind {
    match op {
        BinaryOperator::And => lower_short_circuit_and(lhs, rhs, source_id, fallback_span, span),
        BinaryOperator::Or => lower_short_circuit_or(lhs, rhs, source_id, fallback_span, span),
        // `lhs |> rhs` 降级为 `rhs(lhs)` 函数调用。
        BinaryOperator::Pipe => lower_pipe_expression(lhs, rhs, source_id, fallback_span, span),
        _ => lower_method_call_kind(
            binary_operator_method_name(op),
            vec![
                lower_term_expression_with_context(lhs, source_id, fallback_span.clone(), false),
                lower_term_expression_with_context(rhs, source_id, fallback_span, false),
            ],
            span,
        ),
    }
}

/// 将 `lhs |> rhs` 降级为函数调用。
///
/// 当 `rhs` 是调用表达式 `f(args)` 时，变为 `f(lhs, args)`；
/// 当 `rhs` 是函数引用时，变为 `rhs(lhs)`。
fn lower_pipe_expression(
    lhs: &TermExpression,
    rhs: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    _span: SourceSpan,
) -> HirExprKind {
    let arg = lower_term_expression_with_context(lhs, source_id, fallback_span.clone(), false);

    // 如果 rhs 是调用表达式 `f(args)`，将 lhs 插入参数列表头部，变为 `f(lhs, args)`。
    if let TermExpression::Call { callee, args, .. } = rhs {
        let callee = lower_term_expression_with_context(callee, source_id, fallback_span.clone(), false);
        let mut all_args = vec![arg];
        all_args.extend(args.iter().map(|a| lower_term_expression_with_context(a, source_id, fallback_span.clone(), false)));
        return HirExprKind::Call { callee: Box::new(callee), args: all_args };
    }

    // 否则，rhs 是函数引用，直接调用 `rhs(lhs)`。
    let callee = lower_term_expression_with_context(rhs, source_id, fallback_span, false);
    HirExprKind::Call { callee: Box::new(callee), args: vec![arg] }
}

fn lower_short_circuit_and(
    lhs: &TermExpression,
    rhs: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    span: SourceSpan,
) -> HirExprKind {
    let condition = lower_term_expression_with_context(lhs, source_id, fallback_span.clone(), false);
    let rhs_expr = lower_term_expression_with_context(rhs, source_id, fallback_span, false);
    HirExprKind::If {
        condition: Box::new(condition),
        then_branch: Box::new(HirBlock { statements: Vec::new(), expr: Some(Box::new(rhs_expr)), span: span.clone() }),
        else_branch: Some(Box::new(HirBlock {
            statements: Vec::new(),
            expr: Some(Box::new(HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(false)), span: span.clone() })),
            span,
        })),
    }
}

fn lower_short_circuit_or(
    lhs: &TermExpression,
    rhs: &TermExpression,
    source_id: SourceID,
    fallback_span: Range<usize>,
    span: SourceSpan,
) -> HirExprKind {
    let condition = lower_term_expression_with_context(lhs, source_id, fallback_span.clone(), false);
    let rhs_expr = lower_term_expression_with_context(rhs, source_id, fallback_span, false);
    HirExprKind::If {
        condition: Box::new(condition),
        then_branch: Box::new(HirBlock {
            statements: Vec::new(),
            expr: Some(Box::new(HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: span.clone() })),
            span: span.clone(),
        }),
        else_branch: Some(Box::new(HirBlock { statements: Vec::new(), expr: Some(Box::new(rhs_expr)), span })),
    }
}

fn lower_name_expression(path: &AstNamePath, span: SourceSpan) -> HirExprKind {
    let path = lower_name_path(path);
    if path.0.len() == 1 {
        HirExprKind::Variable(HirIdentifier { name: path.0[0].clone(), shadow_index: 0, span })
    }
    else {
        HirExprKind::Path(path)
    }
}

/// 解析整数字面量文本为 `i64`。
///
/// 支持以下进制前缀（与词法分析器 `lex_number` 保持一致）：
/// - `0x` / `0X`：十六进制
/// - `0b` / `0B`：二进制
/// - `0o` / `0O`：八进制
/// - 无前缀：十进制
///
/// 解析失败时返回 `Err`，由调用方决定 fallback 行为。
fn parse_integer_literal(text: &str) -> Result<i64, std::num::ParseIntError> {
    if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        return i64::from_str_radix(hex, 16);
    }
    if let Some(bin) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        return i64::from_str_radix(bin, 2);
    }
    if let Some(oct) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        return i64::from_str_radix(oct, 8);
    }
    text.parse::<i64>()
}

fn lower_literal_expression(literal: &LiteralExpression, source_id: SourceID, fallback_span: Range<usize>) -> HirExprKind {
    match literal {
        LiteralExpression::Integer(value) => parse_integer_literal(value)
            .map(HirLiteral::Integer64)
            .map(HirExprKind::Literal)
            .unwrap_or_else(|_| HirExprKind::Literal(HirLiteral::Integer64(0))),
        LiteralExpression::Float(value) => value
            .parse::<f64>()
            .map(|v| HirExprKind::Literal(HirLiteral::Float64(OrderedFloat(v))))
            .unwrap_or_else(|_| HirExprKind::Literal(HirLiteral::Float64(OrderedFloat(0.0)))),
        LiteralExpression::String(value) => HirExprKind::Literal(HirLiteral::String(lower_string_literal(value, source_id, fallback_span))),
        LiteralExpression::Bool(value) => HirExprKind::Literal(HirLiteral::Bool(*value)),
        LiteralExpression::Unit => HirExprKind::Literal(HirLiteral::Unit),
    }
}

fn lower_string_literal(literal: &AstStringLiteral, source_id: SourceID, fallback_span: Range<usize>) -> valkyrie_types::hir::HirStringLiteral {
    valkyrie_types::hir::HirStringLiteral {
        prefix: literal.prefix.as_deref().map(Identifier::new),
        quote_count: literal.quote_count,
        segments: literal
            .segments
            .iter()
            .map(|segment| match segment {
                AstStringSegment::Text(text) => valkyrie_types::hir::HirStringSegment::Text(text.clone()),
                AstStringSegment::Interpolation { expression, is_fluent } => valkyrie_types::hir::HirStringSegment::Interpolation {
                    expr: lower_term_expression_with_context(expression, source_id, fallback_span.clone(), false),
                    is_fluent: *is_fluent,
                },
            })
            .collect(),
    }
}

fn binary_operator_method_name(op: &BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::And => unreachable!("&& 走短路控制流，不进入 operator method lowering"),
        BinaryOperator::Or => unreachable!("|| 走短路控制流，不进入 operator method lowering"),
        BinaryOperator::Add => "infix +",
        BinaryOperator::Sub => "infix -",
        BinaryOperator::Mul => "infix *",
        BinaryOperator::Div => "infix /",
        BinaryOperator::Rem => "infix %",
        BinaryOperator::Eq => "infix ==",
        BinaryOperator::Ne => "infix !=",
        BinaryOperator::Lt => "infix <",
        BinaryOperator::Le => "infix <=",
        BinaryOperator::Gt => "infix >",
        BinaryOperator::Ge => "infix >=",
        BinaryOperator::Shl => "infix <<",
        BinaryOperator::Shr => "infix >>",
        BinaryOperator::BitAnd => "infix &",
        BinaryOperator::BitOr => "infix |",
        BinaryOperator::BitXor => "infix ^",
        // `|>` 管道操作符在 HIR 阶段转为函数调用，不进入 operator method lowering。
        BinaryOperator::Pipe => unreachable!("|> 管道操作符走函数调用 lowering，不进入 operator method lowering"),
    }
}

fn unary_operator_method_name(op: &UnaryOperator) -> &'static str {
    match op {
        UnaryOperator::Neg => "prefix -",
        UnaryOperator::Not => "prefix !",
    }
}

fn extract_name_path(expression: &TermExpression) -> Option<NamePath> {
    match expression {
        TermExpression::Name { path, .. } => Some(lower_name_path(path)),
        TermExpression::Literal { literal: LiteralExpression::String(text), .. } => {
            let raw = plain_string_literal_text(text)?;
            Some(NamePath::new(raw.split("::").filter(|part| !part.is_empty()).map(Identifier::new).collect()))
        }
        _ => None,
    }
}

/// 尝试从 Name/MemberAccess 链中提取点分路径。
///
/// 例如 `std.io.print_line` 会被提取为 `["std", "io", "print_line"]`。
/// 如果链中包含非 Name/MemberAccess 节点（如 Call、Subscript 等），返回 None。
fn extract_dotted_path(expr: &TermExpression) -> Option<Vec<String>> {
    match expr {
        TermExpression::Name { path, .. } => {
            if path.parts.is_empty() {
                None
            }
            else {
                Some(path.parts.clone())
            }
        }
        TermExpression::MemberAccess { object, member, .. } => {
            let mut parts = extract_dotted_path(object)?;
            parts.push(member.clone());
            Some(parts)
        }
        _ => None,
    }
}

fn is_self_rooted_member_chain(expr: &TermExpression) -> bool {
    match expr {
        TermExpression::Name { path, .. } => path.parts.len() == 1 && path.parts[0] == "self",
        TermExpression::MemberAccess { object, .. } => is_self_rooted_member_chain(object),
        _ => false,
    }
}

fn plain_string_literal_text(literal: &AstStringLiteral) -> Option<&str> {
    if literal.segments.len() != 1 {
        return None;
    }

    match &literal.segments[0] {
        AstStringSegment::Text(text) => Some(text.as_str()),
        AstStringSegment::Interpolation { .. } => None,
    }
}

fn capture_mode(ty: &HirType) -> CaptureMode {
    match ty {
        HirType::Integer32 | HirType::Integer64 | HirType::Float32 | HirType::Float64 | HirType::Boolean | HirType::Unit | HirType::Void => {
            CaptureMode::ByValue
        }
        _ => CaptureMode::ByReference,
    }
}

fn with_source(span: &Range<usize>, source_id: SourceID) -> SourceSpan {
    SourceSpan::new(source_id, span.start as u32, span.end as u32)
}
