//! Artifact-set bundle serialization for `legion build` IR cache.

use std::{
    fs,
    path::{Path, PathBuf},
};

use emitter::{DriverCompileReport, DriverRunContract};
use nyar_language::ArtifactSet;
use nyar_workspace::{combined_hash, files_hash};
use serde::{Deserialize, Serialize};

use super::{CompilationCache, IrCacheEntry};

const ARTIFACT_KIND: &str = "artifact-set";

/// Alias used by build tests and call sites.
pub type BuildBundle = CachedBuildBundle;

/// Cached build outputs: report metadata + relative files under the output directory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedBuildBundle {
    /// Artifact descriptors from the driver report.
    pub artifacts: ArtifactSet,
    /// Optional entry symbol.
    pub entry_symbol: Option<String>,
    /// Run contracts.
    pub run_contracts: Vec<CachedRunContract>,
    /// Relative path → file bytes.
    pub files: Vec<(String, Vec<u8>)>,
}

/// Serializable run contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CachedRunContract {
    /// Logical entry name.
    pub logical_entry: String,
    /// Physical entry file.
    pub physical_entry: String,
    /// Invocation command.
    pub invocation: String,
    /// Validation command.
    pub validate: String,
}

impl From<&DriverRunContract> for CachedRunContract {
    fn from(value: &DriverRunContract) -> Self {
        Self {
            logical_entry: value.logical_entry.clone(),
            physical_entry: value.physical_entry.clone(),
            invocation: value.invocation.clone(),
            validate: value.validate.clone(),
        }
    }
}

impl From<&CachedRunContract> for DriverRunContract {
    fn from(value: &CachedRunContract) -> Self {
        Self {
            logical_entry: value.logical_entry.clone(),
            physical_entry: value.physical_entry.clone(),
            invocation: value.invocation.clone(),
            validate: value.validate.clone(),
        }
    }
}

/// Toolchain id embedded in artifact keys (invalidates cache on legion upgrades
/// and on identity / MIR / layout / bytecode contract version bumps).
pub fn toolchain_fingerprint() -> String {
    format!(
        "legion={};{};bytecode={}",
        env!("CARGO_PKG_VERSION"),
        nyar_types::contract_version_fingerprint(),
        vcc_data::binary::nyar_ir::BYTECODE_FORMAT_VERSION,
    )
}

/// Compute artifact cache hash from sources + target + build flags + toolchain.
pub fn compute_artifact_hash(
    source_files: &[PathBuf],
    manifest_files: &[PathBuf],
    canonical_triple: &str,
    msil: bool,
    wat: bool,
    runtime_async: bool,
) -> Result<String, String> {
    let sources = files_hash(source_files).map_err(|e| e.to_string())?;
    let manifests = files_hash(manifest_files).map_err(|e| e.to_string())?;
    let flags = format!("{}{}{}", if msil { '1' } else { '0' }, if wat { '1' } else { '0' }, if runtime_async { '1' } else { '0' });
    let toolchain = toolchain_fingerprint();
    Ok(combined_hash(&[&sources, &manifests, canonical_triple, &flags, &toolchain]))
}

/// Collect output directory files into a cacheable bundle.
pub fn collect_build_bundle(output_dir: &Path, report: &DriverCompileReport) -> Result<CachedBuildBundle, String> {
    let mut files = Vec::new();
    collect_files_dir(output_dir, output_dir, &mut files)?;
    Ok(CachedBuildBundle {
        artifacts: report.artifacts.clone(),
        entry_symbol: report.entry_symbol.clone(),
        run_contracts: report.run_contracts.iter().map(CachedRunContract::from).collect(),
        files,
    })
}

fn collect_files_dir(root: &Path, current: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_files_dir(root, &path, out)?;
        }
        else if path.is_file() {
            let rel = path.strip_prefix(root).map_err(|e| e.to_string())?.to_string_lossy().replace('\\', "/");
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            out.push((rel, bytes));
        }
    }
    Ok(())
}

/// Persist a build bundle under the IR cache key.
pub fn store_cached_build(
    cache: &CompilationCache,
    module_name: &str,
    canonical_triple: &str,
    ir_hash: &str,
    payload: &CachedBuildBundle,
) -> Result<(), String> {
    let ir_data = serde_json::to_vec(payload).map_err(|e| e.to_string())?;
    cache.put_ir(
        module_name,
        canonical_triple,
        ir_hash,
        &IrCacheEntry { ir_kind: ARTIFACT_KIND.into(), ir_data, ir_hash: ir_hash.to_string(), canonical_triple: canonical_triple.to_string() },
    )
}

/// 读取缓存元数据；读取结果不能直接成为构建成功载荷。
pub fn load_cached_build(cache: &CompilationCache, module_name: &str, canonical_triple: &str, ir_hash: &str) -> Result<Option<CachedBuildBundle>, String> {
    let Some(entry) = cache.try_get_ir(module_name, canonical_triple, ir_hash) else {
        return Ok(None);
    };
    if entry.ir_kind != ARTIFACT_KIND {
        return Err(format!("cached artifact has unexpected kind `{}`", entry.ir_kind));
    }
    serde_json::from_slice(&entry.ir_data).map(Some).map_err(|error| format!("cached artifact is invalid: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{CompilationCache, *};
    use tempfile::tempdir;

    #[test]
    fn artifact_hash_changes_when_manifest_changes() {
        let dir = tempdir().unwrap();
        let source = dir.path().join("main.v");
        let manifest = dir.path().join("legion.von");
        fs::write(&source, "micro main() -> i64 { return 0 }\n").unwrap();
        fs::write(&manifest, "target: clr\n").unwrap();
        let first = compute_artifact_hash(&[source.clone()], &[manifest.clone()], "clr", false, false, false).unwrap();
        fs::write(&manifest, "target: wasm\n").unwrap();
        let second = compute_artifact_hash(&[source], &[manifest], "clr", false, false, false).unwrap();
        assert_ne!(first, second);
    }

}
