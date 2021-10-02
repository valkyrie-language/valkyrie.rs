#![doc = include_str!("readme.md")]

use nyar_analyzer::{ProgramFacts, RuntimeRequirement};
use nyar_optimizer::{
    FutamuraProjectionFamily, HostProjectionBoundary, ObjectAlgebraicProgram, OptimizationRequest, OptimizationResult, OptimizationSession,
    ProjectionPolicy, ReferenceManagement, RewriteTheory,
};
use nyar_types::{CapabilityTag, QualifiedName};
use serde::{Deserialize, Serialize};

use crate::{
    abstractions::{BackendInputKind, BinaryTarget, CanonicalTarget},
    packaging::TargetLane,
};

/// 进入 `nyar` 规划层的中性输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningInput {
    /// 逻辑模块名。
    pub module_name: QualifiedName,
    /// 目标。
    pub target: CanonicalTarget,
    /// 下游已经闭合好的程序事实。
    pub program_facts: ProgramFacts,
    /// 已经完成前端翻译的 `Object Algebraic` 程序。
    pub object_algebraic_program: ObjectAlgebraicProgram,
    /// 当前启用的等价理论。
    pub rewrite_theory: RewriteTheory,
    /// 目标投影策略。
    pub projection_policy: ProjectionPolicy,
}

/// 单个分区计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPartition {
    /// 分区逻辑名。
    pub name: String,
    /// 当前分区对外暴露的操作。
    pub exported_operations: Vec<QualifiedName>,
    /// 目标路线。
    pub lane: TargetLane,
    /// 面向的二进制目标。
    pub binary_target: BinaryTarget,
    /// 预期 backend 输入。
    pub input_kind: Option<BackendInputKind>,
    /// 宿主边界。
    pub host_boundary: HostProjectionBoundary,
    /// 引用对象管理策略。
    pub reference_management: ReferenceManagement,
    /// 本分区要求的能力标签。
    pub capabilities: Vec<CapabilityTag>,
    /// 本分区要求的运行时契约。
    pub runtime_requirements: Vec<RuntimeRequirement>,
}

/// 单个分区已经收口好的后端需求。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionBackendRequirement {
    /// 当前分区所属路线。
    pub lane: TargetLane,
    /// 当前分区产出的 backend 输入种类。
    pub input_kind: BackendInputKind,
    /// 当前分区面向的目标。
    pub target: BinaryTarget,
    /// 当前分区绑定的宿主边界。
    pub host_boundary: HostProjectionBoundary,
    /// 当前分区采用的引用对象管理策略。
    pub reference_management: ReferenceManagement,
}

/// 中性的产物分区计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPartitionPlan {
    /// 逻辑模块名。
    pub module_name: QualifiedName,
    /// 目标。
    pub target: CanonicalTarget,
    /// 规划前执行得到的优化结果。
    pub optimization: OptimizationResult,
    /// 分区列表。
    pub partitions: Vec<ArtifactPartition>,
}

impl ArtifactPartitionPlan {
    /// 基于程序事实生成最小分区计划。
    pub fn from_input(input: PlanningInput) -> Self {
        let mut projection_policy = input.projection_policy;
        projection_policy.reference_management = resolve_reference_management(
            input.program_facts.reference_management,
            projection_policy.reference_management,
            projection_policy.family,
        );
        let optimization = OptimizationSession::default().optimize(OptimizationRequest {
            program: input.object_algebraic_program,
            capabilities: input.program_facts.capabilities.clone(),
            rewrite_theory: input.rewrite_theory,
            projection_policy,
        });
        let lane = lane_for_projection(optimization.projection.family);
        let binary_target: BinaryTarget = input.target.into();
        let input_kind = input_kind_for_projection(optimization.projection.family);
        let partitions = build_partitions(&input.program_facts, &optimization, lane, binary_target, input_kind);
        Self { module_name: input.module_name, target: input.target, optimization, partitions }
    }

    /// 从指定分区提取已经完成规划的后端需求。
    pub fn backend_requirement(&self, partition_index: usize) -> Option<PartitionBackendRequirement> {
        let partition = self.partitions.get(partition_index)?;
        Some(PartitionBackendRequirement {
            lane: partition.lane,
            input_kind: partition.input_kind?,
            target: partition.binary_target.clone(),
            host_boundary: partition.host_boundary,
            reference_management: partition.reference_management,
        })
    }
}

fn resolve_reference_management(
    preferred: Option<ReferenceManagement>,
    fallback: ReferenceManagement,
    family: FutamuraProjectionFamily,
) -> ReferenceManagement {
    match (family, preferred) {
        (FutamuraProjectionFamily::Native, _) => ReferenceManagement::PerceusRc,
        (_, Some(value)) => value,
        _ => fallback,
    }
}

fn resolve_partition_reference_management(
    program_facts: &ProgramFacts,
    operations: &[QualifiedName],
    dimension_hint: Option<ReferenceManagement>,
    fallback: ReferenceManagement,
    family: FutamuraProjectionFamily,
) -> ReferenceManagement {
    if family == FutamuraProjectionFamily::Native {
        return ReferenceManagement::PerceusRc;
    }

    program_facts.reference_management_for_operations(operations).or(dimension_hint).or(program_facts.reference_management).unwrap_or(fallback)
}

fn build_partitions(
    program_facts: &ProgramFacts,
    optimization: &OptimizationResult,
    lane: TargetLane,
    binary_target: BinaryTarget,
    input_kind: Option<BackendInputKind>,
) -> Vec<ArtifactPartition> {
    if optimization.program.dimensions.is_empty() {
        let reference_management = resolve_partition_reference_management(
            program_facts,
            &optimization.program.exports,
            None,
            optimization.projection.reference_management,
            optimization.projection.family,
        );
        return vec![ArtifactPartition {
            name: optimization.program.module_name.to_string(),
            exported_operations: optimization.program.exports.clone(),
            lane,
            binary_target,
            input_kind,
            host_boundary: optimization.projection.host_boundary,
            reference_management,
            capabilities: optimization.projection.preserved_capabilities.clone(),
            runtime_requirements: resolve_partition_runtime_requirements(
                &program_facts.runtime_requirements,
                &optimization.projection.preserved_capabilities,
            ),
        }];
    }

    optimization
        .program
        .dimensions
        .iter()
        .map(|dimension| {
            let capabilities =
                resolve_partition_capabilities(&optimization.projection.preserved_capabilities, &dimension.required_capabilities);
            let reference_management = resolve_partition_reference_management(
                program_facts,
                &dimension.exported_operations,
                dimension.reference_management_hint,
                optimization.projection.reference_management,
                optimization.projection.family,
            );
            ArtifactPartition {
                name: format!("{}::{}", optimization.program.module_name, dimension.name),
                exported_operations: dimension.exported_operations.clone(),
                lane,
                binary_target: binary_target.clone(),
                input_kind,
                host_boundary: optimization.projection.host_boundary,
                reference_management,
                capabilities: capabilities.clone(),
                runtime_requirements: resolve_partition_runtime_requirements(&program_facts.runtime_requirements, &capabilities),
            }
        })
        .collect()
}

fn resolve_partition_capabilities(
    preserved_capabilities: &[CapabilityTag],
    dimension_required_capabilities: &[CapabilityTag],
) -> Vec<CapabilityTag> {
    if dimension_required_capabilities.is_empty() {
        preserved_capabilities.to_vec()
    }
    else {
        dimension_required_capabilities.to_vec()
    }
}

fn resolve_partition_runtime_requirements(
    runtime_requirements: &[RuntimeRequirement],
    capabilities: &[CapabilityTag],
) -> Vec<RuntimeRequirement> {
    runtime_requirements
        .iter()
        .filter(|requirement| {
            requirement.key == "reference-management" || capabilities.iter().any(|capability| capability.as_str() == requirement.key)
        })
        .cloned()
        .collect()
}

fn lane_for_projection(family: FutamuraProjectionFamily) -> TargetLane {
    match family {
        FutamuraProjectionFamily::Clr => TargetLane::Clr,
        FutamuraProjectionFamily::Jvm => TargetLane::Jvm,
        FutamuraProjectionFamily::Wasm => TargetLane::Wasm,
        FutamuraProjectionFamily::Native => TargetLane::Native,
        FutamuraProjectionFamily::NyarVm => TargetLane::Vm,
    }
}

fn input_kind_for_projection(family: FutamuraProjectionFamily) -> Option<BackendInputKind> {
    match family {
        FutamuraProjectionFamily::Clr => Some(BackendInputKind::MsilText),
        FutamuraProjectionFamily::Jvm => Some(BackendInputKind::JvmClassFile),
        FutamuraProjectionFamily::Wasm => Some(BackendInputKind::WasmModule),
        FutamuraProjectionFamily::Native => Some(BackendInputKind::CoffObject),
        FutamuraProjectionFamily::NyarVm => None,
    }
}
