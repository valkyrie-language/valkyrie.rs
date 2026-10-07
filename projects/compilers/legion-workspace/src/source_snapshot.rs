//! 将 Resolver 的源码组交给 Compiler；依赖身份不因组数或缓存状态改变。

use std::fs;

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::CompilerSourceGroup;

use crate::planner::PlannedSemanticSourceGroup;

pub fn compile_source_snapshot(groups: &[PlannedSemanticSourceGroup]) -> Result<Vec<CompilerSourceGroup>> {
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

    fn compile_snapshot(groups: &[CompilerSourceGroup], output_dir: &Path) -> Result<emitter::DriverCompileReport> {
        let build_context = nyar_language::CompilerBuildContext::new(
            "wasm32",
            nyar_language::CanonicalTarget::parse("node").expect("正式 Node 目标"),
            nyar_language::nyar::ClrSuspendStrategy::default(),
            emitter::nyar_backend_wasi::WasmPackageKind::Binary,
        );
        nyar_language::compile_source_groups_to_artifacts(
            &nyar_language::ValkyrieCompiler::default(),
            groups,
            &build_context,
            output_dir,
            "application",
            false,
            false,
        )
    }

    #[test]
    fn single_group_keeps_resolver_identity() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "[main] micro main() -> i32 { return 23 }", &[]);
        let groups = compile_source_snapshot(&[group]).expect("Resolver 形成完整源码组");
        assert_eq!(groups[0].dependency_key, "application");
        assert_eq!(groups[0].name, "application");
        let output = directory.path().join("output");
        fs::create_dir_all(&output).expect("创建产物目录");
        let report = compile_snapshot(&groups, &output).expect("当前源码进入正式产物入口");
        assert!(!report.artifacts.artifacts.is_empty(), "编译成功必须具有产物");
        assert!(!report.run_contracts.is_empty(), "二进制产物必须具有执行合同");
    }

    #[test]
    fn dependency_groups_link_from_current_source() {
        let directory = tempdir().expect("创建源码目录");
        let dependency = source_group(directory.path(), "library", "micro answer() -> i32 { return 1 }", &[]);
        let application = source_group(directory.path(), "application", "[main] micro main() -> i32 { return answer() }", &["library"]);
        let groups = compile_source_snapshot(&[dependency, application]).expect("Resolver 形成完整源码组");
        assert_eq!(groups[1].direct_dependencies, ["library"]);
        assert_eq!(groups[0].dependency_key, "library");
        let output = directory.path().join("output");
        fs::create_dir_all(&output).expect("创建产物目录");
        let report = compile_snapshot(&groups, &output).expect("当前依赖源码必须完成正式编译");
        assert!(!report.artifacts.artifacts.is_empty());
    }

    #[test]
    fn changed_source_is_not_replaced_by_a_previous_success() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "[main] micro main() -> i32 { return 23 }", &[]);
        let output = directory.path().join("output");
        fs::create_dir_all(&output).expect("创建产物目录");
        let groups = compile_source_snapshot(std::slice::from_ref(&group)).expect("Resolver 形成完整源码组");
        compile_snapshot(&groups, &output).expect("首次源码编译成功");
        fs::write(&group.source_files[0], "micro main(").expect("替换为无效源码");
        let groups = compile_source_snapshot(&[group]).expect("Resolver 读取当前源码");
        compile_snapshot(&groups, &output).expect_err("不得复用已存在的旧产物掩盖当前源码失败");
    }

    #[test]
    fn unresolved_dependency_does_not_flatten_the_source_closure() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "[main] micro main() -> i32 { return 23 }", &["missing"]);
        let groups = compile_source_snapshot(&[group]).expect("Resolver 形成完整源码组");
        let output = directory.path().join("output");
        fs::create_dir_all(&output).expect("创建产物目录");
        let error = compile_snapshot(&groups, &output).expect_err("未解析依赖必须在 Compiler 边界失败");
        assert!(error.to_string().contains("semantic dependency export `missing`"), "{error}");
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

    #[test]
    fn resolver_keeps_templates_and_compiler_expands_by_arch() {
        let directory = tempdir().expect("创建源码目录");
        let template = r#"<% match arch %>
<% case "wasm32" %>
[main] micro main() -> i32 { return 23 }
<% else %>
[main] micro main() -> i32 { return 0 }
<% end %>"#;
        let group = source_group(directory.path(), "application", template, &[]);
        let groups = compile_source_snapshot(&[group]).expect("Resolver 只拼接源码，不展开模板");
        assert!(groups[0].source.contains("<% match "));
        let output = directory.path().join("output");
        fs::create_dir_all(&output).expect("创建产物目录");
        let report = compile_snapshot(&groups, &output).expect("结构化 TGrammar 应由 Compiler 按 arch 展开");
        assert!(!report.artifacts.artifacts.is_empty(), "编译成功必须具有产物");
    }
}
