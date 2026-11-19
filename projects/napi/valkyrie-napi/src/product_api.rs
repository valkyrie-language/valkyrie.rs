//! 程序化 N-API 产品面（`legionPlan` / `legionBuild`），不经 argv CLI。
//!
//! 参数搬运与结果映射之外没有逻辑；规划与编译的唯一实现是
//! `package_manager_legion::product`。

use napi_derive::napi;

/// N-API 暴露的单个语义源码组摘要。
#[napi(object)]
pub struct LegionSemanticGroupItem {
    /// 语义源码组对应的依赖键。
    pub dependency_key: String,
    /// 该组包含的源码文件数。
    pub file_count: u32,
}

/// `legionPlan` 入参。
#[napi(object)]
pub struct LegionPlanOptions {
    /// 项目目录。
    pub project_dir: String,
    /// 编译目标平台字符串，缺省为 `clr`。
    pub target: Option<String>,
}

/// `legionPlan` 的结构化返回。
#[napi(object)]
pub struct LegionPlanOutcome {
    /// 是否成功。
    pub ok: bool,
    /// 失败时的错误描述。
    pub error: Option<String>,
    /// 项目名。
    pub project: Option<String>,
    /// 编译目标平台字符串。
    pub target: Option<String>,
    /// workspace 根目录路径。
    pub workspace_root: Option<String>,
    /// 产物输出目录路径。
    pub output_dir: Option<String>,
    /// 参与构建的源码文件数。
    pub source_count: u32,
    /// 语义源码组数量。
    pub semantic_group_count: u32,
    /// 直接依赖数量。
    pub dependency_count: u32,
    /// 各语义源码组摘要。
    pub groups: Vec<LegionSemanticGroupItem>,
}

/// `legionBuild` 入参。
#[napi(object)]
pub struct LegionBuildOptions {
    /// 项目目录。
    pub project_dir: String,
    /// 编译目标平台字符串，缺省为 `clr`。
    pub target: Option<String>,
    /// 产物输出目录，缺省由规划决定。
    pub output_dir: Option<String>,
}

/// `legionBuild` 的结构化返回。
#[napi(object)]
pub struct LegionBuildOutcome {
    /// 是否成功。
    pub ok: bool,
    /// 失败时的错误描述。
    pub error: Option<String>,
    /// 项目名。
    pub project: Option<String>,
    /// 编译目标平台字符串。
    pub target: Option<String>,
    /// 产物输出目录路径。
    pub output_dir: Option<String>,
    /// 落盘产物路径列表。
    pub artifacts: Vec<String>,
    /// 产物入口符号名（可能为空）。
    pub entry_symbol: Option<String>,
}

/// 解析 workspace 并输出 build plan（不进入 Compiler）。
#[napi]
pub fn legion_plan(options: LegionPlanOptions) -> LegionPlanOutcome {
    let outcome = package_manager_legion::product::plan_project(&package_manager_legion::product::PlanOptions {
        project_dir: options.project_dir.into(),
        target: options.target,
    });
    LegionPlanOutcome {
        ok: outcome.ok,
        error: outcome.error,
        project: outcome.project,
        target: outcome.target,
        workspace_root: outcome.workspace_root,
        output_dir: outcome.output_dir,
        source_count: outcome.source_count,
        semantic_group_count: outcome.semantic_group_count,
        dependency_count: outcome.dependency_count,
        groups: outcome
            .groups
            .into_iter()
            .map(|group| LegionSemanticGroupItem { dependency_key: group.dependency_key, file_count: group.file_count })
            .collect(),
    }
}

/// 编译项目并落盘产物（结构化结果）。
#[napi]
pub fn legion_build(options: LegionBuildOptions) -> LegionBuildOutcome {
    let outcome = package_manager_legion::product::build_project(&package_manager_legion::product::BuildOptions {
        project_dir: options.project_dir.into(),
        target: options.target,
        output_dir: options.output_dir.map(Into::into),
    });
    LegionBuildOutcome {
        ok: outcome.ok,
        error: outcome.error,
        project: outcome.project,
        target: outcome.target,
        output_dir: outcome.output_dir,
        artifacts: outcome.artifacts,
        entry_symbol: outcome.entry_symbol,
    }
}
