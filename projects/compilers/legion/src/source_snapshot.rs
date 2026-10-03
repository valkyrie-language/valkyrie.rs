//! 将 Resolver 的源码组交给 Compiler；依赖身份不因组数或缓存状态改变。

use std::fs;

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::CompilerSourceGroup;

use crate::planner::PlannedSemanticSourceGroup;

pub(crate) fn compile_source_snapshot(
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
    fn single_group_keeps_resolver_identity() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "[main] micro main() -> i32 { return 23 }", &[]);
        let groups = compile_source_snapshot(&[group],).expect("Resolver 形成完整源码组");
        let output = nyar_language::ValkyrieCompiler::default().compile_source_groups(&groups).expect("源码进入唯一 Compiler 入口");
        assert_eq!(output.compiled_program().canonical().linked.module_name, "application");
        assert_eq!(output.compiled_program().canonical().linked.entries.len(), 1);
        assert_eq!(output.compiled_program().canonical().mir.functions.len(), 1);
    }

    #[test]
    fn dependency_groups_link_from_current_source() {
        let directory = tempdir().expect("创建源码目录");
        let dependency = source_group(directory.path(), "library", "micro answer() -> i32 { return 1 }", &[]);
        let application = source_group(directory.path(), "application", "micro main() -> i32 { return answer() }", &["library"]);
        let groups = compile_source_snapshot(&[dependency, application]).expect("Resolver 形成完整源码组");
        let output = nyar_language::ValkyrieCompiler::default().compile_source_groups(&groups).expect("当前依赖源码完成 Compiler 链接");
        assert_eq!(output.compiled_program().canonical().linked.module_name, "application");
        assert_eq!(output.compiled_program().canonical().mir.functions.len(), 2);
        assert_eq!(output.compiled_program().representation().invoke_lowerings.len(), 1);
    }

    #[test]
    fn changed_source_is_not_replaced_by_a_previous_success() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &[]);
        let compiler = nyar_language::ValkyrieCompiler::default();
        let groups = compile_source_snapshot(std::slice::from_ref(&group)).expect("Resolver 形成完整源码组");
        compiler.compile_source_groups(&groups).expect("首次编译成功");
        fs::write(&group.source_files[0], "micro main(").expect("替换为无效源码");
        let groups = compile_source_snapshot(&[group]).expect("Resolver 读取当前源码");
        compiler.compile_source_groups(&groups).expect_err("必须拒绝当前无效源码");
    }

    #[test]
    fn unresolved_dependency_does_not_flatten_the_source_closure() {
        let directory = tempdir().expect("创建源码目录");
        let group = source_group(directory.path(), "application", "micro main() -> i32 { return 23 }", &["missing"]);
        let groups = compile_source_snapshot(&[group]).expect("Resolver 形成完整源码组");
        let error = nyar_language::ValkyrieCompiler::default().compile_source_groups(&groups).expect_err("必须拒绝未解析依赖");
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
}
