//! 将 Resolver 的源码组交给 Compiler；依赖身份不因组数或缓存状态改变。

use std::fs;

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::CompilerSourceGroup;

use crate::planner::PlannedSemanticSourceGroup;

pub fn compile_source_snapshot(
    groups: &[PlannedSemanticSourceGroup],
) -> Result<Vec<CompilerSourceGroup>> {
    if groups.is_empty() {
        return Err(miette!("semantic source group plan is empty"));
    }
    let mut compiler_groups = Vec::with_capacity(groups.len());
    for group in groups {
        let mut source = String::new();
        for path in &group.source_files {
            let content = fs::read_to_string(path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("读取源码失败 {}", path.display())))?;
            source.push_str(content.strip_prefix('\u{FEFF}').unwrap_or(&content));
            source.push('\n');
        }
        compiler_groups.push(CompilerSourceGroup {
            dependency_key: group.dependency_key.clone(),
            name: group.name.clone(),
            source,
            direct_dependencies: group.direct_dependencies.clone(),
        });
    }
    Ok(compiler_groups)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tempfile::tempdir;

    fn source_group(root: &Path, name: &str, source: &str, dependencies: &[&str]) -> PlannedSemanticSourceGroup {
        let source_file = root.join(format!("{name}.v"));
        fs::write(&source_file, source).expect("写入当前源码");
        PlannedSemanticSourceGroup {
            dependency_key: name.to_owned(),
            name: name.to_owned(),
            manifest_dir: root.to_path_buf(),
            source_files: vec![source_file],
            direct_dependencies: dependencies.iter().map(|dependency| (*dependency).to_owned()).collect(),
        }
    }

    #[test]
    fn missing_source_file_fails_instead_of_compiling_a_partial_group() {
        let directory = tempdir().expect("创建源码目录");
        let mut group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &[]);
        group.source_files.push(directory.path().join("missing.v"));
        let error = compile_source_snapshot(&[group]).expect_err("必须拒绝不完整源码组");
        assert!(error.to_string().contains("读取源码失败"), "{error}");
    }

    #[test]
    fn empty_snapshot_is_not_a_successful_program() {
        let error = compile_source_snapshot(&[]).expect_err("必须拒绝空快照");
        assert!(error.to_string().contains("semantic source group plan is empty"), "{error}");
    }
}
