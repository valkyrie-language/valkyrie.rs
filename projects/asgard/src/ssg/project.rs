//! 从 Valyrie VOA 工程目录加载 AWSL/CSS 并静态渲染。

use std::{
    fs,
    path::{Path, PathBuf},
};

use miette::{IntoDiagnostic, Result, WrapErr};
use serde_json::Value;

use crate::{codegen::StaticRenderResult, ssg::render_awsl_static};

/// 定位 `valkyrie.v` 根目录（相对 `valkyrie.rs` 工作区）。
pub fn valkyrie_v_roots() -> Result<Vec<PathBuf>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidates = [manifest.join("../../valkyrie.v"), manifest.join("../../../valkyrie.v"), manifest.join("../../../../valkyrie.v")];
    let roots: Vec<_> = candidates.into_iter().filter(|p| p.is_dir()).collect();
    if roots.is_empty() {
        return Err(miette::miette!("cannot locate valkyrie.v workspace root"));
    }
    Ok(roots)
}

/// 读取工程 `source/{relative}` 文本。
pub fn read_project_source(project_dir: &Path, relative: &str) -> Result<String> {
    let path = project_dir.join("source").join(relative);
    fs::read_to_string(&path).into_diagnostic().wrap_err_with(|| format!("read {}", path.display()))
}

/// 从工程静态渲染 AWSL 页面片段。
pub fn render_project_awsl_static(project_dir: &Path, source_relative: &str, component_name: &str, data: &Value) -> Result<StaticRenderResult> {
    let source = read_project_source(project_dir, source_relative)?;
    Ok(render_awsl_static(&source, component_name, source_relative, data))
}
