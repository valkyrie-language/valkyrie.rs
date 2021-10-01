#![doc = include_str!("readme.md")]

use nyar_analyzer::{ProgramFacts, RuntimeRequirement};
use nyar_optimizer::{
    FutamuraProjectionFamily, ObjectAlgebraicProgram, OptimizationRequest, OptimizationResult, OptimizationSession, ProjectionPolicy,
    RewriteTheory,
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
    /// 目标路线。
    pub lane: TargetLane,
    /// 面向的二进制目标。
    pub binary_target: BinaryTarget,
    /// 预期 backend 输入。
    pub input_kind: Option<BackendInputKind>,
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
        let optimization = OptimizationSession::default().optimize(OptimizationRequest {
            program: input.object_algebraic_program,
            capabilities: input.program_facts.capabilities.clone(),
            rewrite_theory: input.rewrite_theory,
            projection_policy: input.projection_policy,
        });
        let lane = lane_for_projection(optimization.projection.family);
        let partition = ArtifactPartition {
            name: optimization.program.module_name.to_string(),
            lane,
            binary_target: input.target.into(),
            input_kind: input_kind_for_projection(optimization.projection.family),
            capabilities: optimization.projection.preserved_capabilities.clone(),
            runtime_requirements: input.program_facts.runtime_requirements,
        };
        Self { module_name: input.module_name, target: input.target, optimization, partitions: vec![partition] }
    }

    /// 从指定分区提取已经完成规划的后端需求。
    pub fn backend_requirement(&self, partition_index: usize) -> Option<PartitionBackendRequirement> {
        let partition = self.partitions.get(partition_index)?;
        Some(PartitionBackendRequirement { lane: partition.lane, input_kind: partition.input_kind?, target: partition.binary_target.clone() })
    }
}

fn lane_for_projection(family: FutamuraProjectionFamily) -> TargetLane {
    match family {
        FutamuraProjectionFamily::Clr => TargetLane::Clr,
        FutamuraProjectionFamily::Jvm => TargetLane::Jvm,
        FutamuraProjectionFamily::Wasm => TargetLane::Wasm,
        FutamuraProjectionFamily::Native => TargetLane::Native,
        FutamuraProjectionFamily::Vm => TargetLane::Vm,
    }
}

fn input_kind_for_projection(family: FutamuraProjectionFamily) -> Option<BackendInputKind> {
    match family {
        FutamuraProjectionFamily::Clr => Some(BackendInputKind::MsilText),
        FutamuraProjectionFamily::Jvm => Some(BackendInputKind::JvmClassFile),
        FutamuraProjectionFamily::Wasm => Some(BackendInputKind::WasmModule),
        FutamuraProjectionFamily::Native => Some(BackendInputKind::CoffObject),
        FutamuraProjectionFamily::Vm => None,
    }
}
