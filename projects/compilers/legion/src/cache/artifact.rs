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

/// Toolchain id embedded in artifact keys (invalidates cache on legion upgrades).
pub fn toolchain_fingerprint() -> String {
    format!("legion={}", env!("CARGO_PKG_VERSION"))
}

/// Compute artifact cache hash from sources + target + build flags + toolchain.
pub fn compute_artifact_hash(
    source_files: &[PathBuf],
    canonical_triple: &str,
    msil: bool,
    wat: bool,
    runtime_async: bool,
) -> Result<String, String> {
    let sources = files_hash(source_files).map_err(|e| e.to_string())?;
    let flags = format!("{}{}{}", if msil { '1' } else { '0' }, if wat { '1' } else { '0' }, if runtime_async { '1' } else { '0' });
    let toolchain = toolchain_fingerprint();
    Ok(combined_hash(&[&sources, canonical_triple, &flags, &toolchain]))
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

/// Write cached files back and reconstruct a driver report.
pub fn materialize_build_bundle(output_dir: &Path, payload: &CachedBuildBundle) -> Result<DriverCompileReport, String> {
    fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;
    for (rel, bytes) in &payload.files {
        let path = output_dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::write(&path, bytes).map_err(|e| e.to_string())?;
    }
    Ok(DriverCompileReport {
        artifacts: payload.artifacts.clone(),
        entry_symbol: payload.entry_symbol.clone(),
        run_contracts: payload.run_contracts.iter().map(DriverRunContract::from).collect(),
    })
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

/// Load a build bundle from the IR cache.
pub fn load_cached_build(cache: &CompilationCache, module_name: &str, canonical_triple: &str, ir_hash: &str) -> Option<CachedBuildBundle> {
    let entry = cache.try_get_ir(module_name, canonical_triple, ir_hash)?;
    if entry.ir_kind != ARTIFACT_KIND {
        return None;
    }
    serde_json::from_slice(&entry.ir_data).ok()
}

/// Try restore from cache into `output_dir`.
pub fn try_restore_cached_build(
    cache: &CompilationCache,
    module_name: &str,
    canonical_triple: &str,
    ir_hash: &str,
    output_dir: &Path,
) -> Option<DriverCompileReport> {
    let payload = load_cached_build(cache, module_name, canonical_triple, ir_hash)?;
    materialize_build_bundle(output_dir, &payload).ok()
}

#[cfg(test)]
mod tests {
    use super::{CompilationCache, *};
    use tempfile::tempdir;

    #[test]
    fn artifact_bundle_round_trip() {
        let dir = tempdir().unwrap();
        let cache = CompilationCache::open(dir.path().join(".cache"));
        let out_dir = dir.path().join("out");
        fs::create_dir_all(&out_dir).unwrap();
        fs::write(out_dir.join("main.exe"), b"exe-bytes").unwrap();

        let report = DriverCompileReport {
            artifacts: ArtifactSet::default(),
            entry_symbol: Some("main".into()),
            run_contracts: vec![DriverRunContract {
                logical_entry: "main".into(),
                physical_entry: "main.exe".into(),
                invocation: "dotnet".into(),
                validate: "true".into(),
            }],
        };
        let bundle = collect_build_bundle(&out_dir, &report).unwrap();
        store_cached_build(&cache, "demo", "clr-microsoft-unknown-managed", "hash1", &bundle).unwrap();

        let restored_dir = dir.path().join("restored");
        let report2 = try_restore_cached_build(&cache, "demo", "clr-microsoft-unknown-managed", "hash1", &restored_dir).expect("hit");
        assert_eq!(report2.entry_symbol.as_deref(), Some("main"));
        assert_eq!(fs::read(restored_dir.join("main.exe")).unwrap(), b"exe-bytes");
        assert_eq!(report2.run_contracts.len(), 1);
    }
}
