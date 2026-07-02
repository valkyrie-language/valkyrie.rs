//! Source hygiene diagnostics shared by `legion build` and `legion lint`.
//!
//! - `VSRC001`: source file too long (warning above soft limit, error above hard limit)
//! - `VSRC002`: encoding corruption (invalid UTF-8, U+FFFD, mojibake, PS escape residue)

use std::{
    fs,
    path::{Path, PathBuf},
};

use miette::{IntoDiagnostic, Result, miette};

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

/// Scan planned build sources; print diagnostics to stderr.
///
/// Callers should abort the build when [`HygieneReport::has_errors`] is true.
pub fn scan_build_sources(source_files: &[PathBuf]) -> Result<HygieneReport> {
    let mut report = HygieneReport::default();
    for path in source_files {
        let Some(ext) = path.extension().and_then(|e| e.to_str())
        else {
            continue;
        };
        if !(ext.eq_ignore_ascii_case("v") || ext.eq_ignore_ascii_case("vx")) {
            continue;
        }
        report.merge(scan_source_path(path)?);
    }
    Ok(report)
}

/// Read one source path and emit hygiene diagnostics.
pub fn scan_source_path(path: &Path) -> Result<HygieneReport> {
    let bytes = fs::read(path).into_diagnostic().map_err(|e| miette!("读取源码失败 {}：{e}", path.display()))?;
    match std::str::from_utf8(&bytes) {
        Ok(source) => Ok(scan_source_text(path, source)),
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
    let mut warnings = 0usize;
    let mut errors = 0usize;
    let line_count = source.lines().count();

    // VSRC001: same diagnostic; severity escalates with size.
    if line_count > SOURCE_LINE_ERROR {
        errors += 1;
        eprintln!(
            "VSRC001 error: {} has {line_count} lines (hard limit {SOURCE_LINE_ERROR}; warn at {SOURCE_LINE_WARN}); split the module",
            path.display()
        );
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
    fn replacement_char_is_encoding_error() {
        let report = scan_source_text(Path::new("bad.v"), "函\u{FFFD}数");
        assert_eq!(report.errors, 1);
    }
}
