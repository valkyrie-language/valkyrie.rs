//! Bridges compiler HIR facts into `nyar` planning inputs.

use nyar::{
    ArtifactPartitionPlan, CanonicalTarget, CapabilityTag, EntryContract, ExportContract, FunctionAnalysis, FutamuraProjectionFamily,
    HostProjectionBoundary, Identifier, ImportContract, NamePath, ObjectAlgebraicDimension, ObjectAlgebraicProgram, PlanningInput,
    ProgramFacts, ProjectionPolicy, QualifiedName, ReferenceManagement, RewritePhase, RewriteRule, RewriteTheory, RuntimeRequirement,
    TargetBackendFamily, TargetHostKind,
};
use valkyrie_types::{
    hir::{AccessLevel, HirFunction, HirModule},
    Identifier as ValkyrieIdentifier, NamePath as ValkyrieNamePath,
};

use crate::type_checker::{AssignmentSemantics, CopySemanticsValidator, ParameterSemantics, ReturnSemantics};

/// Builds frontend-neutral `ProgramFacts` from a lowered `HIR` module.
pub fn hir_module_to_program_facts(module: &HirModule) -> ProgramFacts {
    let module_name = lower_qualified_name(&module.name);
    let entry = select_entry_contract(module, &module_name);
    let validator = build_copy_semantics_validator(module);
    let imports = module
        .imports
        .iter()
        .map(|path| {
            let lowered_path = lower_name_path(path);
            let local_name = lower_qualified_name(path);
            ImportContract { path: lowered_path, local_name }
        })
        .collect();
    let exports = module
        .functions
        .iter()
        .filter(|function| function.visibility.access == AccessLevel::Public)
        .map(|function| ExportContract {
            exported_name: lower_identifier(&function.name),
            local_name: function_symbol(module_name.clone(), function),
        })
        .collect();
    let functions: Vec<FunctionAnalysis> = module
        .functions
        .iter()
        .map(|function| FunctionAnalysis {
            symbol: function_symbol(module_name.clone(), function),
            is_external: function.is_abstract,
            can_suspend: has_annotation(function, "await") || has_annotation(function, "block"),
            uses_host_interop: has_host_interop_annotation(function),
            reference_management_hint: infer_function_reference_management(function, &validator),
        })
        .collect();
    let mut capabilities = Vec::new();
    let mut runtime_requirements = Vec::new();
    if functions.iter().any(|function| function.uses_host_interop) {
        capabilities.push(CapabilityTag::new("host-interop"));
        runtime_requirements.push(RuntimeRequirement { key: "host-interop".to_string(), value: "required".to_string() });
    }
    if functions.iter().any(|function| function.can_suspend) {
        capabilities.push(CapabilityTag::new("suspend"));
        runtime_requirements.push(RuntimeRequirement { key: "suspend".to_string(), value: "required".to_string() });
    }
    let reference_management = infer_module_reference_management(module, &functions, &validator);
    if let Some(reference_management) = reference_management {
        runtime_requirements.push(RuntimeRequirement {
            key: "reference-management".to_string(),
            value: match reference_management {
                ReferenceManagement::HostGc => "host-gc".to_string(),
                ReferenceManagement::PerceusRc => "perceus-rc".to_string(),
            },
        });
    }
    ProgramFacts { module_name, entry, imports, exports, functions, capabilities, reference_management, runtime_requirements }
}

/// Compatibility shim for older bridge call sites.
pub fn hir_module_to_analysis_artifact(module: &HirModule) -> ProgramFacts {
    hir_module_to_program_facts(module)
}

/// Builds a minimal `Object Algebraic` program boundary from `HIR`.
pub fn hir_module_to_object_algebraic_program(module: &HirModule) -> ObjectAlgebraicProgram {
    let module_name = lower_qualified_name(&module.name);
    let validator = build_copy_semantics_validator(module);
    let functions = module.functions.iter().map(|function| function_symbol(module_name.clone(), function)).collect::<Vec<_>>();
    let mut dimensions = vec![ObjectAlgebraicDimension {
        name: Identifier::new("functions"),
        exported_operations: functions.clone(),
        required_capabilities: Vec::new(),
        reference_management_hint: infer_dimension_reference_management(module.functions.iter(), &validator),
    }];

    let host_operations = module
        .functions
        .iter()
        .filter(|function| has_host_interop_annotation(function))
        .map(|function| function_symbol(module_name.clone(), function))
        .collect::<Vec<_>>();
    if !host_operations.is_empty() {
        dimensions.push(ObjectAlgebraicDimension {
            name: Identifier::new("host-interop"),
            exported_operations: host_operations,
            required_capabilities: vec![CapabilityTag::new("host-interop")],
            reference_management_hint: infer_dimension_reference_management(
                module.functions.iter().filter(|function| has_host_interop_annotation(function)),
                &validator,
            ),
        });
    }

    let suspend_operations = module
        .functions
        .iter()
        .filter(|function| has_annotation(function, "await") || has_annotation(function, "block"))
        .map(|function| function_symbol(module_name.clone(), function))
        .collect::<Vec<_>>();
    if !suspend_operations.is_empty() {
        dimensions.push(ObjectAlgebraicDimension {
            name: Identifier::new("suspend"),
            exported_operations: suspend_operations,
            required_capabilities: vec![CapabilityTag::new("suspend")],
            reference_management_hint: infer_dimension_reference_management(
                module.functions.iter().filter(|function| has_annotation(function, "await") || has_annotation(function, "block")),
                &validator,
            ),
        });
    }

    ObjectAlgebraicProgram { module_name, exports: functions, dimensions }
}

/// Builds a minimal `nyar` artifact partition plan from `HIR`.
pub fn hir_module_to_artifact_plan(module: &HirModule, target: CanonicalTarget) -> ArtifactPartitionPlan {
    let program_facts = hir_module_to_program_facts(module);
    ArtifactPartitionPlan::from_input(PlanningInput {
        module_name: lower_qualified_name(&module.name),
        target,
        program_facts: program_facts.clone(),
        object_algebraic_program: hir_module_to_object_algebraic_program(module),
        rewrite_theory: default_rewrite_theory(&program_facts),
        projection_policy: default_projection_policy(target),
    })
}

fn select_entry_contract(module: &HirModule, module_name: &QualifiedName) -> Option<EntryContract> {
    let entry_function = module
        .functions
        .iter()
        .find(|function| has_annotation(function, "main") && function.name.as_str() == "main")
        .or_else(|| module.functions.iter().find(|function| has_annotation(function, "main")))
        .or_else(|| module.functions.iter().find(|function| function.name.as_str() == "main"))?;
    Some(EntryContract { symbol: function_symbol(module_name.clone(), entry_function), requires_wrapper: true })
}

fn function_symbol(module_name: QualifiedName, function: &HirFunction) -> QualifiedName {
    let mut parts = module_name.parts().to_vec();
    parts.push(lower_identifier(&function.name));
    QualifiedName::new(parts)
}

fn lower_identifier(identifier: &ValkyrieIdentifier) -> Identifier {
    Identifier::new(identifier.as_str())
}

fn lower_name_path(path: &ValkyrieNamePath) -> NamePath {
    NamePath::new(path.parts().iter().map(lower_identifier).collect())
}

fn lower_qualified_name(path: &ValkyrieNamePath) -> QualifiedName {
    QualifiedName::new(path.parts().iter().map(lower_identifier).collect())
}

fn has_host_interop_annotation(function: &HirFunction) -> bool {
    has_annotation(function, "clr") || has_annotation(function, "wasm_import")
}

fn has_annotation(function: &HirFunction, expected: &str) -> bool {
    function.annotations.iter().any(|attribute| attribute.name.parts().last().is_some_and(|part| part.as_str() == expected))
}

fn build_copy_semantics_validator(module: &HirModule) -> CopySemanticsValidator {
    let mut validator = CopySemanticsValidator::new();
    for item in &module.structs {
        validator.register_value_type(item);
    }
    validator
}

fn infer_module_reference_management(
    module: &HirModule,
    functions: &[FunctionAnalysis],
    validator: &CopySemanticsValidator,
) -> Option<ReferenceManagement> {
    module_uses_reference_semantics(module, functions, validator).then_some(ReferenceManagement::HostGc)
}

fn module_uses_reference_semantics(module: &HirModule, functions: &[FunctionAnalysis], validator: &CopySemanticsValidator) -> bool {
    module.structs.iter().any(|item| {
        !item.is_value_type
            || item.fields.iter().any(|field| matches!(validator.validate_assignment(&field.ty), AssignmentSemantics::Reference))
            || item.methods.iter().any(|method| infer_function_reference_management(method, validator).is_some())
    }) || functions.iter().any(|function| function.reference_management_hint.is_some())
}

fn infer_dimension_reference_management<'a>(
    functions: impl IntoIterator<Item = &'a HirFunction>,
    validator: &CopySemanticsValidator,
) -> Option<ReferenceManagement> {
    functions
        .into_iter()
        .any(|function| infer_function_reference_management(function, validator).is_some())
        .then_some(ReferenceManagement::HostGc)
}

fn infer_function_reference_management(function: &HirFunction, validator: &CopySemanticsValidator) -> Option<ReferenceManagement> {
    function_uses_reference_semantics(function, validator).then_some(ReferenceManagement::HostGc)
}

fn function_uses_reference_semantics(function: &HirFunction, validator: &CopySemanticsValidator) -> bool {
    function.params.iter().any(|param| matches!(validator.validate_parameter_passing(&param.ty), ParameterSemantics::Reference))
        || matches!(validator.validate_return(&function.return_type), ReturnSemantics::Reference)
}

fn default_rewrite_theory(program_facts: &ProgramFacts) -> RewriteTheory {
    let mut theory = RewriteTheory::default();
    theory.register(RewriteRule {
        name: Identifier::new("normalize.exports"),
        phase: RewritePhase::Normalize,
        required_capabilities: Vec::new(),
        allowed_projection_families: Vec::new(),
    });
    if program_facts.requires_capability("host-interop") {
        theory.register(RewriteRule {
            name: Identifier::new("saturate.host-interop"),
            phase: RewritePhase::Saturate,
            required_capabilities: vec![CapabilityTag::new("host-interop")],
            allowed_projection_families: vec![
                FutamuraProjectionFamily::Clr,
                FutamuraProjectionFamily::Jvm,
                FutamuraProjectionFamily::Wasm,
                FutamuraProjectionFamily::Native,
            ],
        });
    }
    if program_facts.requires_capability("suspend") {
        theory.register(RewriteRule {
            name: Identifier::new("pre-projection.suspend-boundary"),
            phase: RewritePhase::PreProjection,
            required_capabilities: vec![CapabilityTag::new("suspend")],
            allowed_projection_families: vec![FutamuraProjectionFamily::Clr, FutamuraProjectionFamily::Wasm, FutamuraProjectionFamily::NyarVm],
        });
    }
    theory
}

fn default_projection_policy(target: CanonicalTarget) -> ProjectionPolicy {
    let target_profile = target.to_profile(None);
    let backend_family = target_profile.backend_family;
    ProjectionPolicy {
        family: match backend_family {
            TargetBackendFamily::Clr => FutamuraProjectionFamily::Clr,
            TargetBackendFamily::Jvm => FutamuraProjectionFamily::Jvm,
            TargetBackendFamily::Wasm => FutamuraProjectionFamily::Wasm,
            TargetBackendFamily::Native => FutamuraProjectionFamily::Native,
            _ => FutamuraProjectionFamily::NyarVm,
        },
        host_boundary: match target_profile.host_kind {
            TargetHostKind::DotNet => HostProjectionBoundary::Clr,
            TargetHostKind::Jvm => HostProjectionBoundary::Jvm,
            TargetHostKind::Wasi => HostProjectionBoundary::WasiComponent,
            TargetHostKind::Browser | TargetHostKind::JavaScript => HostProjectionBoundary::WasmJsGlue,
            TargetHostKind::Native => HostProjectionBoundary::Native,
            TargetHostKind::NyarVm => HostProjectionBoundary::Vm,
            TargetHostKind::Gpu | TargetHostKind::Unknown => target_profile.host_boundary,
        },
        reference_management: match backend_family {
            TargetBackendFamily::Native | TargetBackendFamily::Gpu => ReferenceManagement::PerceusRc,
            _ => ReferenceManagement::HostGc,
        },
        prefer_small_artifacts: matches!(backend_family, TargetBackendFamily::Wasm | TargetBackendFamily::Native),
        preserve_effect_boundaries: !matches!(backend_family, TargetBackendFamily::Native),
    }
}
