//! WASM 编译：复用 `compile_v_bundle`（仅 browser）。

use std::path::Path;

use miette::Result;
use nyar_language::CanonicalTarget;

use crate::{
    compile::{HostArtifactKind, compile_v_bundle},
    host_backend::HostBackend,
};

/// WASM 编译报告。
#[derive(Debug, Clone)]
pub struct WasmCompileReport {
    /// 主 wasm 文件名（不含路径）。
    pub wasm_filename: String,
    /// glue 文件名。
    pub glue_filename: String,
}

/// 将合并后的 V 源码编译为 WASM + glue。
pub fn compile_wasm_bundle(
    combined_v_source: &str,
    output_dir: &Path,
    module_name: &str,
    target: &CanonicalTarget,
) -> Result<WasmCompileReport> {
    let report = compile_v_bundle(combined_v_source, output_dir, module_name, target, HostBackend::BrowserDom)?;
    if report.kind != HostArtifactKind::Wasm {
        return Err(miette::miette!("预期 WASM 制品"));
    }
    report.wasm.ok_or_else(|| miette::miette!("缺少 WASM 报告"))
}

/// 将 WASM 产物复制/提升到 dist 根目录（供 manifest 引用）。
pub fn copy_wasm_artifacts_to_dist(output_dir: &Path, report: &WasmCompileReport) -> Result<()> {
    use miette::{IntoDiagnostic, WrapErr};
    use std::fs;

    let wasm_name = &report.wasm_filename;
    let glue_name = &report.glue_filename;
    let mut copied_wasm = false;

    for entry in walkdir_files(output_dir)? {
        let path = entry?;
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let is_wasm = file_name == wasm_name || file_name.ends_with(".wasm");
        if is_wasm {
            let dest = output_dir.join(if file_name != wasm_name { wasm_name } else { file_name });
            if path != dest {
                fs::copy(&path, &dest).into_diagnostic().wrap_err_with(|| format!("复制 {file_name} 失败"))?;
            }
            copied_wasm = true;
        }
    }

    if !copied_wasm {
        for entry in walkdir_files(output_dir)? {
            let path = entry?;
            if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                let dest = output_dir.join(wasm_name);
                if path != dest {
                    fs::copy(&path, &dest).into_diagnostic().wrap_err("复制 wasm 失败")?;
                }
                copied_wasm = true;
                break;
            }
        }
    }

    if copied_wasm {
        write_browser_wasm_glue(output_dir, glue_name)?;
    }
    Ok(())
}

/// 写入浏览器 `boot.js` 可 `import()` 的 WASM glue（`instantiate` / `run`）。
pub fn write_browser_wasm_glue(output_dir: &Path, glue_filename: &str) -> Result<()> {
    use miette::IntoDiagnostic;
    use std::fs;

    let content = r#"export async function instantiate(wasmUrl, imports) {
  const response = await fetch(wasmUrl);
  if (!response.ok) {
    throw new Error('wasm fetch failed: ' + response.status);
  }
  const bytes = await response.arrayBuffer();
  const result = await WebAssembly.instantiate(bytes, imports || {});
  return result.instance;
}

export async function run(wasmUrl, imports) {
  return instantiate(wasmUrl, imports);
}
"#;
    fs::write(output_dir.join(glue_filename), content).into_diagnostic()?;
    Ok(())
}

fn walkdir_files(dir: &Path) -> Result<impl Iterator<Item = Result<std::path::PathBuf>>> {
    use miette::IntoDiagnostic;
    use std::fs;

    let mut stack = vec![dir.to_path_buf()];
    let mut files = Vec::new();
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).into_diagnostic()? {
            let entry = entry.into_diagnostic()?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            }
            else {
                files.push(Ok(path));
            }
        }
    }
    Ok(files.into_iter())
}
