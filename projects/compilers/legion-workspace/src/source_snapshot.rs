//! 将 Resolver 的源码组交给 Compiler；依赖身份不因组数或缓存状态改变。

use std::fs;

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::{CompilerSourceFile, CompilerSourceGroup};

use nyar_language::plan_semantic_closure;

use crate::planner::PlannedSemanticSourceGroup;

/// 单个语义源码组允许纳入的最大字节数。
const MAX_GROUP_SOURCE_BYTES: usize = 16 * 1024 * 1024;
/// 全部语义组拼接后的总字节数上限。
const MAX_TOTAL_SOURCE_BYTES: usize = 64 * 1024 * 1024;

pub fn compile_source_snapshot(groups: &[PlannedSemanticSourceGroup]) -> Result<Vec<CompilerSourceGroup>> {
    if groups.is_empty() {
        return Err(miette!("semantic source group plan is empty"));
    }

    let mut staged_sources = Vec::with_capacity(groups.len());
    let mut total_bytes = 0usize;
    for group in groups {
        let manifest_dir = fs::canonicalize(&group.manifest_dir).into_diagnostic()?;
        let mut sources = Vec::with_capacity(group.source_files.len());
        let mut group_bytes = 0usize;
        for path in &group.source_files {
            let resolved_path = fs::canonicalize(path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("读取源码失败：源码组 `{}`", group.dependency_key)))?;
            let logical_path = resolved_path
                .strip_prefix(&manifest_dir)
                .map_err(|_| miette!("源码文件不属于源码组 `{}` 的 manifest 目录", group.dependency_key))?
                .to_string_lossy()
                .replace('\\', "/");
            let content = fs::read_to_string(path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("读取源码失败 {}::{logical_path}", group.dependency_key)))?;
            let normalized = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
            group_bytes += normalized.len();
            if group_bytes > MAX_GROUP_SOURCE_BYTES {
                return Err(miette!(
                    "semantic source group `{}` exceeds {} byte limit (would be {} bytes)",
                    group.dependency_key,
                    MAX_GROUP_SOURCE_BYTES,
                    group_bytes
                ));
            }
            total_bytes += normalized.len();
            if total_bytes > MAX_TOTAL_SOURCE_BYTES {
                return Err(miette!("semantic source snapshot exceeds {} byte total limit", MAX_TOTAL_SOURCE_BYTES));
            }
            sources.push(CompilerSourceFile { path: logical_path, source: normalized.to_owned() });
        }
        staged_sources.push((group, sources));
    }

    let plan_input = staged_sources
        .iter()
        .map(|(group, sources)| {
            let files = sources
                .iter()
                .map(|file| (file.path.clone(), file.source.clone()))
                .collect::<Vec<_>>();
            (group.dependency_key.as_str(), group.is_entry, files)
        })
        .collect::<Vec<_>>();
    let plan_input_refs = plan_input
        .iter()
        .map(|(dependency_key, is_entry, files)| (*dependency_key, *is_entry, files.as_slice()))
        .collect::<Vec<_>>();
    let semantic_closure = plan_semantic_closure(&plan_input_refs);
    let closure_digest = semantic_closure.digest.clone();

    let mut compiler_groups = Vec::with_capacity(groups.len());
    for ((group, sources), group_closure) in staged_sources.into_iter().zip(semantic_closure.groups) {
        compiler_groups.push(CompilerSourceGroup {
            dependency_key: group.dependency_key.clone(),
            name: group.name.clone(),
            source: String::new(),
            sources,
            direct_dependencies: group.direct_dependencies.clone(),
            tolerate_inventory_parse_failures: group_closure.tolerate_inventory_parse_failures,
            body_lowering_source_paths: Some(group_closure.body_lowering_source_paths),
            closure_digest: Some(closure_digest.clone()),
        });
    }
    Ok(compiler_groups)
}
