//! 执行清单、`legion test` 与内嵌 OVM 调用（不经 argv、不定位外部 CLI）。

use std::path::PathBuf;

use napi::bindgen_prelude::*;
use napi_derive::napi;

/// execution manifest 摘要（JSON 友好）。
#[napi(object)]
pub struct LegionExecutionManifestView {
    /// schema 版本。
    pub schema_version: u32,
    /// canonical target 字符串。
    pub target: String,
    /// 产物种类：`binary` / `library`。
    pub artifact_kind: String,
    /// 库导出索引（逻辑名 + 物理产物）；非库产物为空。
    pub library_exports: Vec<LegionLibraryExport>,
    /// 语义闭包哈希。
    pub semantic_closure_hash: Option<String>,
    /// 规范表面哈希。
    pub canonical_surface_hash: Option<String>,
    /// host provider 绑定哈希。
    pub host_provider_binding_hash: Option<String>,
    /// 首个 run contract 物理入口。
    pub primary_physical_entry: Option<String>,
    /// 首个 run contract 逻辑入口。
    pub primary_logical_entry: Option<String>,
    /// 产出该构建的编译器构建身份；旧构建缺少该段时为 `null`。
    pub compiler_build_identity: Option<String>,
}

/// 库导出索引条目（源码名 + 目标公开名 + 物理产物 + ABI 合同标签 + 载体签名）。
#[napi(object)]
pub struct LegionLibraryExport {
    /// 逻辑导出名（目标命名合同投影后的公开名）。
    pub logical_name: String,
    /// 语言侧冻结的源码声明名；旧 manifest 缺少该字段时为空串。
    pub source_name: String,
    /// 物理产物（相对输出目录的文件名）。
    pub physical_artifact: String,
    /// 目标 ABI 合同标签（`node-api` / `cpython` / `c-abi` / `wasm` 等）；旧 manifest 缺少该字段时为空串。
    pub abi: String,
    /// 形参宿主载体标签（`i64` / `i64[]` / `utf8`，按形参顺序）；未冻结时为空列表。
    pub parameters: Vec<String>,
    /// 返回宿主载体标签（`i64` / `bool` / `utf8` / `i64[]` / `unit`）；未冻结时为空串。
    pub returns: String,
}

/// 产物新鲜度检查结果。
#[napi(object)]
pub struct LegionArtifactFreshness {
    /// 是否可信复用；只有 `level == "fully-verified"` 才为 `true`。
    pub fresh: bool,
    /// 失败原因。
    pub reason: Option<String>,
    /// 权威校验级别：`fully-verified` / `artifact-self-consistent` / `stale`。
    pub level: String,
}

/// `legion test` 结构化结果。
#[napi(object)]
pub struct LegionTestOutcome {
    /// 是否全部通过。
    pub ok: bool,
    /// 失败说明。
    pub error: Option<String>,
    /// 通过数。
    pub passed: u32,
    /// 失败数。
    pub failed: u32,
    /// 跳过数。
    pub skipped: u32,
}

/// 读取 `run-contracts.txt` 摘要。
#[napi]
pub fn legion_read_execution_manifest(output_dir: String) -> Option<LegionExecutionManifestView> {
    let manifest = package_manager_legion::execution::read_manifest(&PathBuf::from(output_dir))?;
    Some(LegionExecutionManifestView {
        schema_version: manifest.schema_version,
        target: manifest.target,
        artifact_kind: manifest.artifact_kind,
        library_exports: manifest
            .library_exports
            .into_iter()
            .map(|entry| LegionLibraryExport {
                logical_name: entry.logical_name,
                source_name: entry.source_name,
                physical_artifact: entry.physical_artifact,
                abi: entry.abi,
                parameters: entry.parameters,
                returns: entry.returns,
            })
            .collect(),
        semantic_closure_hash: manifest.semantic_closure_hash,
        canonical_surface_hash: manifest.canonical_surface_hash,
        host_provider_binding_hash: manifest.host_provider_binding_hash,
        primary_physical_entry: manifest.primary_physical_entry,
        primary_logical_entry: manifest.primary_logical_entry,
        compiler_build_identity: manifest.compiler_build_identity,
    })
}

/// 本 addon 内嵌的编译器构建身份。
#[napi]
pub fn legion_compiler_build_identity() -> String {
    package_manager_legion::execution::compiler_build_identity()
}

/// 本 addon 的完整 toolchain fingerprint，供消费者交叉核对。
#[napi]
pub fn legion_toolchain_fingerprint() -> String {
    package_manager_legion::execution::toolchain_fingerprint()
}

/// 判断输出目录中的 execution manifest 是否可信复用。
///
/// 只有 `fully-verified` 才放行；缺字段、摘要不符、产物消失一律 `fresh:false`。
/// `project_dir` 提供时重算当前 workspace 输入摘要，达到 `fully-verified`；
/// 不提供时最高只能达到 `artifact-self-consistent`。
#[napi]
pub fn legion_check_artifact_fresh(output_dir: String, expected_target: String, project_dir: Option<String>) -> LegionArtifactFreshness {
    let freshness = package_manager_legion::execution::check_artifact_fresh(
        &PathBuf::from(output_dir),
        &expected_target,
        project_dir.as_deref().map(std::path::Path::new),
    );
    LegionArtifactFreshness { fresh: freshness.fresh, reason: freshness.reason, level: freshness.level }
}

/// 运行 `legion test`（结构化；不经 argv）。
#[napi]
pub fn legion_test(project_dir: String, target: Option<String>) -> LegionTestOutcome {
    let outcome = package_manager_asgard::product::test_project(PathBuf::from(project_dir), target);
    LegionTestOutcome { ok: outcome.ok, error: outcome.error, passed: outcome.passed, failed: outcome.failed, skipped: outcome.skipped }
}

/// Legion 内嵌 OVM：列出 `.odin` 导出。
#[napi]
pub fn legion_list_ovm_exports(module: String) -> Result<Vec<String>> {
    package_manager_legion::ovm::list_exports(std::path::Path::new(&module)).map_err(Error::from_reason)
}

/// Legion 内嵌 OVM：调用导出并返回 JSON（`args_json` 为 JSON 数组）。
#[napi]
pub fn legion_invoke_ovm_export(module: String, entry: String, args_json: String) -> Result<String> {
    package_manager_legion::ovm::invoke_export(std::path::Path::new(&module), &entry, &args_json).map_err(Error::from_reason)
}
