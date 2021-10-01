#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

use nyar_types::{CapabilityTag, Identifier, QualifiedName};

/// 单个 `Object Algebraic` 语义维度的清单。
///
/// 这里记录的是“这个程序对外暴露了哪些语义视图”，
/// 而不是重新发明一个闭合节点池。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectAlgebraicDimension {
    /// 维度名称。
    pub name: Identifier,
    /// 该维度对外提供的语义操作或解释入口。
    pub exported_operations: Vec<QualifiedName>,
    /// 激活该维度所需的能力标签。
    pub required_capabilities: Vec<CapabilityTag>,
}

/// `Object Algebraic` 程序边界。
///
/// 当前阶段只保留组合边界和维度清单，
/// 明确拒绝把 `Object Algebraic` 简化为统一节点枚举。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectAlgebraicProgram {
    /// 模块名。
    pub module_name: QualifiedName,
    /// 稳定导出符号。
    pub exports: Vec<QualifiedName>,
    /// 已注册的语义维度。
    pub dimensions: Vec<ObjectAlgebraicDimension>,
}

impl ObjectAlgebraicProgram {
    /// 注册一个语义维度。
    pub fn register_dimension(&mut self, dimension: ObjectAlgebraicDimension) {
        self.dimensions.push(dimension);
    }
}

/// `Object Algebraic` builder 的最小公共接口。
pub trait ObjectAlgebraicBuilder {
    /// 返回逻辑模块名。
    fn module_name(&self) -> &QualifiedName;

    /// 返回稳定导出符号。
    fn exports(&self) -> &[QualifiedName];

    /// 返回已声明的语义维度。
    fn dimensions(&self) -> &[ObjectAlgebraicDimension];
}

impl ObjectAlgebraicBuilder for ObjectAlgebraicProgram {
    fn module_name(&self) -> &QualifiedName {
        &self.module_name
    }

    fn exports(&self) -> &[QualifiedName] {
        &self.exports
    }

    fn dimensions(&self) -> &[ObjectAlgebraicDimension] {
        &self.dimensions
    }
}

/// `Object Algebraic` 解释器边界。
pub trait ObjectAlgebraicInterpreter {
    /// 返回解释器名。
    fn interpreter_name(&self) -> &Identifier;

    /// 返回解释器支持的能力。
    fn supported_capabilities(&self) -> &[CapabilityTag];
}

/// `E-Graph` 重写所在阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewritePhase {
    /// 规范化阶段。
    Normalize,
    /// 等价饱和阶段。
    Saturate,
    /// 为目标投影做收口的阶段。
    PreProjection,
}

/// 单条等价重写规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteRule {
    /// 规则名。
    pub name: Identifier,
    /// 所属阶段。
    pub phase: RewritePhase,
    /// 触发规则所需能力。
    pub required_capabilities: Vec<CapabilityTag>,
    /// 允许投影到的目标家族。
    pub allowed_projection_families: Vec<FutamuraProjectionFamily>,
}

/// 一组重写规则。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RewriteTheory {
    /// 已注册规则。
    pub rules: Vec<RewriteRule>,
}

impl RewriteTheory {
    /// 注册一条重写规则。
    pub fn register(&mut self, rule: RewriteRule) {
        self.rules.push(rule);
    }

    /// 统计某阶段的规则数量。
    pub fn count_in_phase(&self, phase: RewritePhase) -> usize {
        self.rules.iter().filter(|rule| rule.phase == phase).count()
    }
}

/// `E-Graph` 会话快照。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EGraphSnapshot {
    /// 等价类数量。
    pub equivalence_class_count: usize,
    /// 维度操作数量。
    pub operation_count: usize,
    /// 是否达到当前理论下的饱和。
    pub saturated: bool,
}

/// `Futamura projection` 目标家族。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FutamuraProjectionFamily {
    /// `futa_clr`
    Clr,
    /// `futa_jvm`
    Jvm,
    /// `futa_wasm`
    Wasm,
    /// `futa_native`
    Native,
    /// `futa_vm`
    Vm,
}

impl FutamuraProjectionFamily {
    /// 返回稳定家族名。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clr => "futa_clr",
            Self::Jvm => "futa_jvm",
            Self::Wasm => "futa_wasm",
            Self::Native => "futa_native",
            Self::Vm => "futa_vm",
        }
    }
}

/// `Futamura projection` 选择策略。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionPolicy {
    /// 目标投影家族。
    pub family: FutamuraProjectionFamily,
    /// 是否优先缩小产物体积。
    pub prefer_small_artifacts: bool,
    /// 是否保留显式 effect 边界。
    pub preserve_effect_boundaries: bool,
}

/// 单次目标投影计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionPlan {
    /// 选中的目标家族。
    pub family: FutamuraProjectionFamily,
    /// 投影后仍需要保留的能力。
    pub preserved_capabilities: Vec<CapabilityTag>,
}

/// 一次优化请求。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationRequest {
    /// 输入的 `Object Algebraic` 程序。
    pub program: ObjectAlgebraicProgram,
    /// 当前可用能力。
    pub capabilities: Vec<CapabilityTag>,
    /// 本轮使用的等价理论。
    pub rewrite_theory: RewriteTheory,
    /// 目标投影策略。
    pub projection_policy: ProjectionPolicy,
}

/// 优化结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptimizationResult {
    /// 选择后的 `Object Algebraic` 程序。
    pub program: ObjectAlgebraicProgram,
    /// 本轮 `E-Graph` 快照。
    pub egraph: EGraphSnapshot,
    /// 目标投影计划。
    pub projection: ProjectionPlan,
    /// 被采用的规则名。
    pub applied_rules: Vec<Identifier>,
}

/// `nyar` 优化会话。
#[derive(Debug, Default)]
pub struct OptimizationSession;

impl OptimizationSession {
    /// 执行一次最小优化骨架。
    ///
    /// 当前阶段只固定：
    /// - `Object Algebraic` 的组合边界
    /// - `E-Graph` 的规则筛选入口
    /// - `Futamura projection` 的目标选择边界
    pub fn optimize(&self, request: OptimizationRequest) -> OptimizationResult {
        let applied_rules = request
            .rewrite_theory
            .rules
            .iter()
            .filter(|rule| {
                rule.allowed_projection_families.is_empty() || rule.allowed_projection_families.contains(&request.projection_policy.family)
            })
            .filter(|rule| {
                rule.required_capabilities.iter().all(|capability| request.capabilities.iter().any(|provided| provided == capability))
            })
            .map(|rule| rule.name.clone())
            .collect::<Vec<_>>();

        let operation_count = request.program.dimensions.iter().map(|dimension| dimension.exported_operations.len()).sum();
        let preserved_capabilities = request
            .capabilities
            .iter()
            .filter(|capability| request.projection_policy.preserve_effect_boundaries || capability.as_str() != "suspend")
            .cloned()
            .collect();

        OptimizationResult {
            program: request.program,
            egraph: EGraphSnapshot { equivalence_class_count: applied_rules.len().max(1), operation_count, saturated: true },
            projection: ProjectionPlan { family: request.projection_policy.family, preserved_capabilities },
            applied_rules,
        }
    }
}
