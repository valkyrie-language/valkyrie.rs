#![doc = include_str!("readme.md")]

pub mod clr;

use serde::{Deserialize, Serialize};

use crate::{
    abstractions::{BackendInputKind, BinaryTarget},
    packaging::ArtifactSet,
};

/// 编译选项。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompilationOptions {
    /// 输出目标。
    pub target: BinaryTarget,
    /// 逻辑产物名。
    pub artifact_name: String,
    /// 是否生成调试信息。
    pub emit_debug_symbols: bool,
    /// 是否启用优化。
    pub optimize: bool,
}

/// 后端描述。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendDescriptor {
    /// 后端名。
    pub name: String,
    /// 期待的 lane 输入种类。
    pub input_kind: BackendInputKind,
    /// 支持的目标。
    pub supported_targets: Vec<BinaryTarget>,
}

/// 目标代码生成后端。
///
/// 这里不统一所有目标的物理表示，只统一“每个后端都必须诚实声明自己吃什么”。
pub trait TargetCodeGenBackend {
    /// 后端真实消费的输入类型。
    type Input;

    /// 返回后端描述。
    fn descriptor(&self) -> &BackendDescriptor;

    /// 验证输入是否满足本后端路线约束。
    fn validate(&self, input: &Self::Input) -> miette::Result<()>;

    /// 只对通过验证的输入执行目标相关编码与产物生成。
    fn compile(&self, input: Self::Input, options: &CompilationOptions) -> miette::Result<ArtifactSet>;
}
