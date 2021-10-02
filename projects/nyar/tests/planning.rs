use nyar::{
    ArtifactPartitionPlan, BackendInputKind, CanonicalTarget, FutamuraProjectionFamily, HostProjectionBoundary, ObjectAlgebraicDimension,
    ObjectAlgebraicProgram, PlanningInput, ProgramFacts, ProjectionPolicy, QualifiedName, ReferenceManagement, RewritePhase, RewriteRule,
    RewriteTheory, RuntimeRequirement, TargetLane,
};
use nyar_types::{CapabilityTag, Identifier};

fn qualified_name(parts: &[&str]) -> QualifiedName {
    QualifiedName::new(parts.iter().map(|part| Identifier::new(part)).collect())
}

#[test]
fn planning_runs_optimizer_before_partitioning() {
    let module_name = qualified_name(&["demo"]);
    let program_facts = ProgramFacts {
        module_name: module_name.clone(),
        entry: None,
        imports: Vec::new(),
        exports: Vec::new(),
        functions: Vec::new(),
        capabilities: vec![CapabilityTag::new("suspend")],
        reference_management: Some(ReferenceManagement::HostGc),
        runtime_requirements: vec![RuntimeRequirement { key: "suspend".to_string(), value: "required".to_string() }],
    };
    let object_algebraic_program = ObjectAlgebraicProgram {
        module_name: module_name.clone(),
        exports: vec![qualified_name(&["demo", "main"])],
        dimensions: vec![ObjectAlgebraicDimension {
            name: Identifier::new("functions"),
            exported_operations: vec![qualified_name(&["demo", "main"])],
            required_capabilities: Vec::new(),
            reference_management_hint: Some(ReferenceManagement::HostGc),
        }],
    };
    let mut rewrite_theory = RewriteTheory::default();
    rewrite_theory.register(RewriteRule {
        name: Identifier::new("pre-projection.suspend-boundary"),
        phase: RewritePhase::PreProjection,
        required_capabilities: vec![CapabilityTag::new("suspend")],
        allowed_projection_families: vec![FutamuraProjectionFamily::Clr],
    });

    let plan = ArtifactPartitionPlan::from_input(PlanningInput {
        module_name,
        target: CanonicalTarget::clr(),
        program_facts,
        object_algebraic_program,
        rewrite_theory,
        projection_policy: ProjectionPolicy {
            family: FutamuraProjectionFamily::Clr,
            host_boundary: HostProjectionBoundary::Clr,
            reference_management: ReferenceManagement::HostGc,
            prefer_small_artifacts: false,
            preserve_effect_boundaries: true,
        },
    });

    assert_eq!(plan.optimization.projection.family, FutamuraProjectionFamily::Clr);
    assert_eq!(plan.optimization.projection.host_boundary, HostProjectionBoundary::Clr);
    assert_eq!(plan.optimization.projection.reference_management, ReferenceManagement::HostGc);
    assert_eq!(plan.partitions[0].lane, TargetLane::Clr);
    assert_eq!(plan.partitions[0].input_kind, Some(BackendInputKind::MsilText));
    assert_eq!(plan.partitions[0].host_boundary, HostProjectionBoundary::Clr);
    assert_eq!(plan.partitions[0].reference_management, ReferenceManagement::HostGc);
    assert_eq!(plan.partitions[0].runtime_requirements.len(), 1);
    assert_eq!(plan.optimization.applied_rules[0].as_str(), "pre-projection.suspend-boundary");
}

#[test]
fn planning_can_promote_operation_level_reference_management_hint() {
    let module_name = qualified_name(&["demo"]);
    let operation = qualified_name(&["demo", "main"]);
    let program_facts = ProgramFacts {
        module_name: module_name.clone(),
        entry: None,
        imports: Vec::new(),
        exports: Vec::new(),
        functions: vec![nyar::FunctionAnalysis {
            symbol: operation.clone(),
            is_external: false,
            can_suspend: false,
            uses_host_interop: false,
            reference_management_hint: Some(ReferenceManagement::HostGc),
        }],
        capabilities: Vec::new(),
        reference_management: None,
        runtime_requirements: Vec::new(),
    };
    let object_algebraic_program = ObjectAlgebraicProgram {
        module_name: module_name.clone(),
        exports: vec![operation.clone()],
        dimensions: vec![ObjectAlgebraicDimension {
            name: Identifier::new("functions"),
            exported_operations: vec![operation],
            required_capabilities: Vec::new(),
            reference_management_hint: Some(ReferenceManagement::HostGc),
        }],
    };

    let plan = ArtifactPartitionPlan::from_input(PlanningInput {
        module_name,
        target: CanonicalTarget::clr(),
        program_facts,
        object_algebraic_program,
        rewrite_theory: RewriteTheory::default(),
        projection_policy: ProjectionPolicy {
            family: FutamuraProjectionFamily::Clr,
            host_boundary: HostProjectionBoundary::Clr,
            reference_management: ReferenceManagement::PerceusRc,
            prefer_small_artifacts: false,
            preserve_effect_boundaries: true,
        },
    });

    assert_eq!(plan.partitions[0].reference_management, ReferenceManagement::HostGc);
}

#[test]
fn planning_splits_partitions_by_dimension() {
    let module_name = qualified_name(&["demo"]);
    let host_operation = qualified_name(&["demo", "host_call"]);
    let suspend_operation = qualified_name(&["demo", "await_next"]);
    let base_operation = qualified_name(&["demo", "main"]);
    let program_facts = ProgramFacts {
        module_name: module_name.clone(),
        entry: None,
        imports: Vec::new(),
        exports: Vec::new(),
        functions: vec![
            nyar::FunctionAnalysis {
                symbol: base_operation.clone(),
                is_external: false,
                can_suspend: false,
                uses_host_interop: false,
                reference_management_hint: None,
            },
            nyar::FunctionAnalysis {
                symbol: host_operation.clone(),
                is_external: false,
                can_suspend: false,
                uses_host_interop: true,
                reference_management_hint: Some(ReferenceManagement::HostGc),
            },
            nyar::FunctionAnalysis {
                symbol: suspend_operation.clone(),
                is_external: false,
                can_suspend: true,
                uses_host_interop: false,
                reference_management_hint: Some(ReferenceManagement::HostGc),
            },
        ],
        capabilities: vec![CapabilityTag::new("host-interop"), CapabilityTag::new("suspend")],
        reference_management: None,
        runtime_requirements: vec![
            RuntimeRequirement { key: "host-interop".to_string(), value: "required".to_string() },
            RuntimeRequirement { key: "suspend".to_string(), value: "required".to_string() },
        ],
    };
    let object_algebraic_program = ObjectAlgebraicProgram {
        module_name: module_name.clone(),
        exports: vec![base_operation.clone(), host_operation.clone(), suspend_operation.clone()],
        dimensions: vec![
            ObjectAlgebraicDimension {
                name: Identifier::new("functions"),
                exported_operations: vec![base_operation],
                required_capabilities: Vec::new(),
                reference_management_hint: None,
            },
            ObjectAlgebraicDimension {
                name: Identifier::new("host-interop"),
                exported_operations: vec![host_operation],
                required_capabilities: vec![CapabilityTag::new("host-interop")],
                reference_management_hint: Some(ReferenceManagement::HostGc),
            },
            ObjectAlgebraicDimension {
                name: Identifier::new("suspend"),
                exported_operations: vec![suspend_operation],
                required_capabilities: vec![CapabilityTag::new("suspend")],
                reference_management_hint: Some(ReferenceManagement::HostGc),
            },
        ],
    };

    let plan = ArtifactPartitionPlan::from_input(PlanningInput {
        module_name,
        target: CanonicalTarget::clr(),
        program_facts,
        object_algebraic_program,
        rewrite_theory: RewriteTheory::default(),
        projection_policy: ProjectionPolicy {
            family: FutamuraProjectionFamily::Clr,
            host_boundary: HostProjectionBoundary::Clr,
            reference_management: ReferenceManagement::PerceusRc,
            prefer_small_artifacts: false,
            preserve_effect_boundaries: true,
        },
    });

    assert_eq!(plan.partitions.len(), 3);
    assert_eq!(plan.partitions[0].name, "demo::functions");
    assert_eq!(plan.partitions[1].name, "demo::host-interop");
    assert_eq!(plan.partitions[2].name, "demo::suspend");
    assert_eq!(plan.partitions[0].capabilities.len(), 2);
    assert_eq!(plan.partitions[1].capabilities, vec![CapabilityTag::new("host-interop")]);
    assert_eq!(plan.partitions[2].capabilities, vec![CapabilityTag::new("suspend")]);
    assert_eq!(
        plan.partitions[1].runtime_requirements,
        vec![RuntimeRequirement { key: "host-interop".to_string(), value: "required".to_string() }]
    );
    assert_eq!(plan.partitions[2].runtime_requirements, vec![RuntimeRequirement { key: "suspend".to_string(), value: "required".to_string() }]);
    assert_eq!(plan.partitions[1].reference_management, ReferenceManagement::HostGc);
    assert_eq!(plan.partitions[2].reference_management, ReferenceManagement::HostGc);
}
