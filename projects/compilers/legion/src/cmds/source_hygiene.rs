//! Source hygiene diagnostics shared by `legion build` and `legion lint`.
//!
//! - `VSRC001`: source file too long (warning above soft limit, error above hard limit)
//! - `VSRC002`: encoding corruption (invalid UTF-8, U+FFFD, mojibake, PS escape residue)
//!
//! Node/Wasm GC builds soft-isolate frozen CLR/JVM legacy body_source files for
//! `VSRC001` only (size). Encoding corruption (`VSRC002`) always fails closed.

use std::{
    fs,
    path::{Path, PathBuf},
};

use miette::{IntoDiagnostic, Result, miette};
use nyar_language::CanonicalTarget;

/// Soft ceiling for `.v` / `.vx` source size (lines). Exceeding emits `VSRC001` at warning.
pub const SOURCE_LINE_WARN: usize = 1024;
/// Hard ceiling for `.v` / `.vx` source size (lines). Exceeding emits `VSRC001` at error.
pub const SOURCE_LINE_ERROR: usize = 4096;

/// Aggregated hygiene findings for one scan pass.
#[derive(Debug, Default, Clone)]
pub struct HygieneReport {
    /// `VSRC001` warnings (over soft line limit).
    pub warnings: usize,
    /// Hard failures (`VSRC001` error and any `VSRC002`).
    pub errors: usize,
}

impl HygieneReport {
    /// Merge another report into this one.
    pub fn merge(&mut self, other: HygieneReport) {
        self.warnings += other.warnings;
        self.errors += other.errors;
    }

    /// True when any hard failure was recorded.
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }
}

/// Scan policy for a build target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HygienePolicy {
    /// When true, oversized frozen CLR/JVM legacy files emit `VSRC001` warning instead of error.
    pub soften_frozen_lane_size: bool,
}

impl HygienePolicy {
    /// Strict policy (lint / CLR·JVM builds).
    pub fn strict() -> Self {
        Self { soften_frozen_lane_size: false }
    }

    /// Policy derived from the active build target.
    pub fn for_target(target: &CanonicalTarget) -> Self {
        let profile = target.to_profile(None);
        let soften = matches!(profile.host_kind, nyar_language::TargetHostKind::JavaScript | nyar_language::TargetHostKind::Wasi)
            || matches!(profile.backend_family, nyar_language::nyar::TargetBackendFamily::Wasm);
        Self { soften_frozen_lane_size: soften }
    }
}

/// Scan planned build sources; print diagnostics to stderr.
///
/// Callers should abort the build when [`HygieneReport::has_errors`] is true.
pub fn scan_build_sources(source_files: &[PathBuf]) -> Result<HygieneReport> {
    scan_build_sources_with_policy(source_files, HygienePolicy::strict())
}

/// Scan planned build sources with a target-aware policy.
pub fn scan_build_sources_with_policy(source_files: &[PathBuf], policy: HygienePolicy) -> Result<HygieneReport> {
    let mut report = HygieneReport::default();
    for path in source_files {
        let Some(ext) = path.extension().and_then(|e| e.to_str())
        else {
            continue;
        };
        if !(ext.eq_ignore_ascii_case("v") || ext.eq_ignore_ascii_case("vx")) {
            continue;
        }
        report.merge(scan_source_path_with_policy(path, policy)?);
    }
    Ok(report)
}

/// Read one source path and emit hygiene diagnostics.
pub fn scan_source_path(path: &Path) -> Result<HygieneReport> {
    scan_source_path_with_policy(path, HygienePolicy::strict())
}

/// Read one source path with an explicit policy.
pub fn scan_source_path_with_policy(path: &Path, policy: HygienePolicy) -> Result<HygieneReport> {
    let bytes = fs::read(path).into_diagnostic().map_err(|e| miette!("读取源码失败 {}：{e}", path.display()))?;
    match std::str::from_utf8(&bytes) {
        Ok(source) => Ok(scan_source_text_with_policy(path, source, policy)),
        Err(error) => {
            eprintln!(
                "VSRC002 error: {}: invalid UTF-8 at byte {} (encoding corruption; re-save as UTF-8)",
                path.display(),
                error.valid_up_to()
            );
            Ok(HygieneReport { warnings: 0, errors: 1 })
        }
    }
}

/// Emit hygiene diagnostics for already-decoded source text.
pub fn scan_source_text(path: &Path, source: &str) -> HygieneReport {
    scan_source_text_with_policy(path, source, HygienePolicy::strict())
}

/// Emit hygiene diagnostics with an explicit policy.
pub fn scan_source_text_with_policy(path: &Path, source: &str, policy: HygienePolicy) -> HygieneReport {
    let mut warnings = 0usize;
    let mut errors = 0usize;
    let line_count = source.lines().count();
    let soften_size = policy.soften_frozen_lane_size && is_frozen_lane_legacy_source(path);

    // VSRC001: same diagnostic; severity escalates with size (unless softened).
    if line_count > SOURCE_LINE_ERROR {
        if soften_size {
            warnings += 1;
            eprintln!(
                "VSRC001 warning: {} has {line_count} lines (hard limit {SOURCE_LINE_ERROR}; soft-isolated for Node/Wasm — frozen CLR/JVM legacy; split later)",
                path.display()
            );
        }
        else {
            errors += 1;
            eprintln!(
                "VSRC001 error: {} has {line_count} lines (hard limit {SOURCE_LINE_ERROR}; warn at {SOURCE_LINE_WARN}); split the module",
                path.display()
            );
        }
    }
    else if line_count > SOURCE_LINE_WARN {
        warnings += 1;
        eprintln!("VSRC001 warning: {} has {line_count} lines (soft limit {SOURCE_LINE_WARN}; hard limit {SOURCE_LINE_ERROR})", path.display());
    }

    for (idx, line) in source.lines().enumerate() {
        if looks_like_encoding_corruption(line) {
            errors += 1;
            eprintln!(
                "VSRC002 error: {}:{line_no}: source encoding corruption (mojibake / replacement char / PS escape residue)",
                path.display(),
                line_no = idx + 1
            );
        }
    }

    HygieneReport { warnings, errors }
}

/// Fail closed when hygiene errors were found during build.
pub fn require_clean(report: &HygieneReport) -> Result<()> {
    if report.has_errors() {
        return Err(miette!(
            "source hygiene failed: {} error(s), {} warning(s) (VSRC001 size / VSRC002 encoding)",
            report.errors,
            report.warnings
        ));
    }
    if report.warnings > 0 {
        eprintln!("source hygiene: {} warning(s)", report.warnings);
    }
    Ok(())
}

/// Frozen-lane legacy sources that must not block the Node/Wasm seed path on size alone.
fn is_frozen_lane_legacy_source(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
    if name.starts_with("clr_") || name.starts_with("jvm_") || name.contains("body_lowering") || name.contains("body_source") {
        return true;
    }
    let rendered = path.to_string_lossy().replace('\\', "/").to_ascii_lowercase();
    rendered.contains("/clr_") || rendered.contains("/jvm_") || rendered.contains("body_lowering")
}

fn looks_like_encoding_corruption(line: &str) -> bool {
    // Primary signal: U+FFFD already baked in (truncated multi-byte CJK, console/PS mangling).
    if line.contains('\u{FFFD}') {
        return true;
    }
    // Classic GBK/UTF-8 mis-decode placeholders.
    if line.contains("锟斤拷") || line.contains("Ã©") || line.contains("â€™") {
        return true;
    }
    // Secondary: PowerShell escape residue dumped as literal text.
    line.contains("`r`n") || line.contains("`n`r")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_limit_same_code_escalates_severity() {
        let soft = "x\n".repeat(SOURCE_LINE_WARN + 1);
        let hard = "x\n".repeat(SOURCE_LINE_ERROR + 1);
        let soft_report = scan_source_text(Path::new("soft.v"), &soft);
        let hard_report = scan_source_text(Path::new("hard.v"), &hard);
        assert_eq!(soft_report.warnings, 1);
        assert_eq!(soft_report.errors, 0);
        assert_eq!(hard_report.warnings, 0);
        assert_eq!(hard_report.errors, 1);
    }

    #[test]
    fn node_policy_softens_frozen_clr_size() {
        let hard = "x\n".repeat(SOURCE_LINE_ERROR + 1);
        let policy = HygienePolicy { soften_frozen_lane_size: true };
        let report = scan_source_text_with_policy(Path::new("clr_body_lowering.v"), &hard, policy);
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 1);
    }

    #[test]
    fn replacement_char_is_encoding_error() {
        let report = scan_source_text(Path::new("bad.v"), "函\u{FFFD}数");
        assert_eq!(report.errors, 1);
    }
}
