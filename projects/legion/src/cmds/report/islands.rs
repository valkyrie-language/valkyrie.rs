//! Thin wrapper: legion report charts delegate to `voa::report_build`.

use std::path::Path;

use miette::Result;
use voa::report_build::emit_report_chart_islands;

use super::{codegen::ChartIslandSpec, project::legion_report_project_dir};

pub use voa::report_build::{asgard_boot_head_tags, asgard_start_snippet};

/// Emit chart islands under `output_dir` and return `(route, mount_html)` pairs.
pub fn emit_chart_islands(output_dir: &Path, module_stem: &str, charts: &[ChartIslandSpec]) -> Result<Vec<(String, String)>> {
    let project_dir = legion_report_project_dir()?;
    emit_report_chart_islands(&project_dir, output_dir, module_stem, charts)
}
