use nyar::{
    ArtifactPartitionPlan, BackendInputKind, CanonicalTarget, FutamuraProjectionFamily, ObjectAlgebraicDimension, ObjectAlgebraicProgram,
    PlanningInput, ProgramFacts, ProjectionPolicy, QualifiedName, RewritePhase, RewriteRule, RewriteTheory, RuntimeRequirement, TargetLane,
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
        runtime_requirements: vec![RuntimeRequirement { key: "suspend".to_string(), value: "required".to_string() }],
    };
    let object_algebraic_program = ObjectAlgebraicProgram {
        module_name: module_name.clone(),
        exports: vec![qualified_name(&["demo", "main"])],
        dimensions: vec![ObjectAlgebraicDimension {
            name: Identifier::new("functions"),
            exported_operations: vec![qualified_name(&["demo", "main"])],
            required_capabilities: Vec::new(),
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
            prefer_small_artifacts: false,
            preserve_effect_boundaries: true,
        },
    });

    assert_eq!(plan.optimization.projection.family, FutamuraProjectionFamily::Clr);
    assert_eq!(plan.partitions[0].lane, TargetLane::Clr);
    assert_eq!(plan.partitions[0].input_kind, Some(BackendInputKind::MsilText));
    assert_eq!(plan.partitions[0].runtime_requirements.len(), 1);
    assert_eq!(plan.optimization.applied_rules[0].as_str(), "pre-projection.suspend-boundary");
}
