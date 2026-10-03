//! 将 Resolver 的源码组交给 Compiler；依赖身份不因组数或缓存状态改变。

use std::fs;

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::{CompilerSourceGroup, FrontendBuildOutput, ValkyrieCompiler};

use crate::planner::PlannedSemanticSourceGroup;

pub(crate) fn compile_source_snapshot(
    groups: &[PlannedSemanticSourceGroup],
    arch: &str,
    preprocess: impl Fn(&str, &str) -> String,
) -> Result<FrontendBuildOutput> {
    let mut compiler_groups = Vec::with_capacity(groups.len());
    for group in groups {
        let mut source = String::new();
        for path in &group.source_files {
            let content = fs::read_to_string(path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("读取源码失败 {}", path.display())))?;
            let staged = preprocess(content.strip_prefix('\u{FEFF}').unwrap_or(&content), arch);
            source.push_str(&staged);
            source.push('\n');
        }
        compiler_groups.push(CompilerSourceGroup {
            dependency_key: group.dependency_key.clone(),
            name: group.name.clone(),
            source,
            direct_dependencies: group.direct_dependencies.clone(),
        });
    }
    ValkyrieCompiler::default()
        .compile_source_groups(&compiler_groups)
        .map_err(|error| miette!("Compiler semantic snapshot failed: {error}"))
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
    fn single_group_keeps_resolver_identity() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "[main] micro main() -> i32 { return 23 }", &[]);
        let output = compile_source_snapshot(&[group], "wasm32", |source, _| source.to_owned()).expect("源码进入唯一 Compiler 入口");
        assert_eq!(output.compiled_program().canonical().linked.module_name, "application");
        assert_eq!(output.compiled_program().canonical().linked.entries.len(), 1);
        assert_eq!(output.compiled_program().canonical().mir.functions.len(), 1);
    }

    #[test]
    fn dependency_groups_link_from_current_source() {
        let directory = tempdir().expect("创建源码目录");
        let dependency = source_group(directory.path(), "library", "micro answer() -> i32 { return 1 }", &[]);
        let application = source_group(directory.path(), "application", "micro main() -> i32 { return answer() }", &["library"]);
        let output = compile_source_snapshot(&[dependency, application], "wasm32", |source, _| source.to_owned())
            .expect("当前依赖源码完成 Compiler 链接");
        assert_eq!(output.compiled_program().canonical().linked.module_name, "application");
        assert_eq!(output.compiled_program().canonical().mir.functions.len(), 2);
        assert_eq!(output.compiled_program().representation().invoke_lowerings.len(), 1);
    }

    #[test]
    fn changed_source_is_not_replaced_by_a_previous_success() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &[]);
        compile_source_snapshot(std::slice::from_ref(&group), "wasm32", |source, _| source.to_owned()).expect("首次编译成功");
        fs::write(&group.source_files[0], "micro main(").expect("替换为无效源码");
        compile_source_snapshot(&[group], "wasm32", |source, _| source.to_owned()).expect_err("必须拒绝当前无效源码");
    }

    #[test]
    fn unresolved_dependency_does_not_flatten_the_source_closure() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &["missing"]);
        let error = compile_source_snapshot(&[group], "wasm32", |source, _| source.to_owned()).expect_err("必须拒绝未解析依赖");
        assert!(error.to_string().contains("semantic dependency export `missing`"), "{error}");
    }

    #[test]
    fn missing_source_file_fails_instead_of_compiling_a_partial_group() {
        let directory = tempdir().expect("创建源码目录");
        let mut group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &[]);
        group.source_files.push(directory.path().join("missing.v"));
        let error = compile_source_snapshot(&[group], "wasm32", |source, _| source.to_owned()).expect_err("必须拒绝不完整源码组");
        assert!(error.to_string().contains("读取源码失败"), "{error}");
    }

    #[test]
    fn empty_snapshot_is_not_a_successful_program() {
        let error = compile_source_snapshot(&[], "wasm32", |source, _| source.to_owned()).expect_err("必须拒绝空快照");
        assert!(error.to_string().contains("semantic source group plan is empty"), "{error}");
    }
}
