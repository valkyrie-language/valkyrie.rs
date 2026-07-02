//! 报告混合岛构建：hydrated 图表走 WASM + asgard glue（模板来自 VOA 工程 AWSL）。

use std::{fs, path::Path};

use miette::{IntoDiagnostic, Result, WrapErr};
use nyar_analyzer::report::{HydratedChartSpec, series_to_init_literal};
use nyar_language::CanonicalTarget;
use std_data::text::awsl::widget_name_from_stem;

use crate::{
    awsl::{LoweringOptions, compile_awsl_source},
    codegen::{IslandPackageOptions, asgard_boot_script_tag, asgard_boot_stylesheet_tag, build_awsl_wasm_source, package_browser_islands},
    ssg::project::{read_project_source, valkyrie_v_roots},
    wasm::{compile_wasm_bundle, copy_wasm_artifacts_to_dist},
};

/// 构建 hydrated 图表岛，返回 `(route, mount_html)`。
pub fn emit_report_chart_islands(
    project_dir: &Path,
    output_dir: &Path,
    module_stem: &str,
    charts: &[HydratedChartSpec],
) -> Result<Vec<(String, String)>> {
    if charts.is_empty() {
        return Ok(Vec::new());
    }

    let roots = valkyrie_v_roots()?;
    let icp_awsl = load_interactive_col_plot(&roots)?;

    fs::create_dir_all(output_dir).into_diagnostic()?;

    let mut components = Vec::new();
    let mut slots = Vec::new();

    for chart in charts {
        let wrapper = read_project_source(project_dir, &format!("charts/{}.awsl", chart.route))?;
        let title_lit = utf8_literal(&chart.title);
        let series_lit = series_to_init_literal(&chart.series);
        let filled = fill_chart_placeholders(&wrapper, &title_lit, &series_lit);
        let awsl = expand_interactive_col_plot(&filled, &icp_awsl, &chart.route)?;
        let component = compile_awsl_source(&awsl, &chart.route, &format!("{}.awsl", chart.route), &LoweringOptions::default())
            .map_err(|error| miette::miette!("compile chart AWSL {}: {}", chart.route, error.message))?;
        let mount = format!(
            r#"<div data-island="hydrated" data-component="{route}" data-module="{module}"></div>"#,
            route = chart.route,
            module = module_stem,
        );
        slots.push((chart.route.clone(), mount));
        components.push(component);
    }

    if components.is_empty() {
        return Ok(Vec::new());
    }

    let wasm_stem = module_stem.replace('.', "-");
    let awsl_v = build_awsl_wasm_source(&components);
    let wasm_built = match compile_wasm_bundle(&awsl_v, output_dir, module_stem, &CanonicalTarget::wasm()) {
        Ok(report) => {
            if let Err(error) = copy_wasm_artifacts_to_dist(output_dir, &report) {
                eprintln!("asgard report: copy wasm artifacts failed: {error}");
                false
            }
            else {
                true
            }
        }
        Err(error) => {
            eprintln!("asgard report: wasm compile deferred ({error}); glue/boot still written");
            false
        }
    };

    let opts = IslandPackageOptions {
        module_stem: module_stem.to_string(),
        wasm_stem,
        wasm_built,
        relative_urls: true,
        css_mode: "merge".into(),
        mode: "report".into(),
        tailwind_css: None,
    };
    package_browser_islands(&components, output_dir, &opts).wrap_err("package chart islands")?;

    Ok(slots)
}

/// Head tags for asgard boot CSS (markup only).
pub fn asgard_boot_head_tags(css_name: &str) -> String {
    asgard_boot_stylesheet_tag(css_name)
}

/// Boot script tag before `</body>`（仅 `src=`，start 在 auto boot.js 内）。
pub fn asgard_start_snippet() -> &'static str {
    asgard_boot_script_tag(true)
}

fn load_interactive_col_plot(roots: &[std::path::PathBuf]) -> Result<String> {
    let rel = "source/components/interactive-col-plot.awsl";
    let candidates = ["projects/asgard.plotter", "projects/.asgard/projects/asgard.plotter", "projects/asgard._/projects/asgard.plotter"];
    for root in roots {
        for project in candidates {
            let path = root.join(project).join(rel);
            if path.is_file() {
                return fs::read_to_string(&path).into_diagnostic().wrap_err_with(|| format!("read {}", path.display()));
            }
        }
    }
    Err(miette::miette!("missing asgard.plotter InteractiveColPlot.awsl"))
}

fn fill_chart_placeholders(wrapper: &str, title_lit: &str, series_lit: &str) -> String {
    wrapper.replace("__TITLE__", title_lit).replace("__SERIES__", series_lit)
}

fn utf8_literal(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn expand_interactive_col_plot(wrapper: &str, icp_awsl: &str, route: &str) -> Result<String> {
    let (title_line, series_line) = extract_title_series_lets(wrapper)?;
    let widget_name = widget_name_from_stem(route);
    let mut out = icp_awsl.replacen("<widget interactive_col_plot>", &format!("<widget {widget_name}>"), 1);
    if !out.contains(&format!("<widget {widget_name}>")) {
        return Err(miette::miette!("InteractiveColPlot template missing widget tag"));
    }
    out = replace_let_line(&out, "title", &title_line)?;
    out = replace_let_line(&out, "series", &series_line)?;
    Ok(out)
}

fn extract_title_series_lets(wrapper: &str) -> Result<(String, String)> {
    let mut title = None;
    let mut series = None;
    for line in wrapper.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("let title") {
            title = Some(trimmed.to_string());
        }
        else if trimmed.starts_with("let series") {
            series = Some(trimmed.to_string());
        }
    }
    match (title, series) {
        (Some(t), Some(s)) => Ok((t, s)),
        _ => Err(miette::miette!("chart wrapper missing let title / let series")),
    }
}

fn replace_let_line(source: &str, name: &str, replacement: &str) -> Result<String> {
    let prefix = format!("let {name}");
    let mut found = false;
    let mut out = String::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&prefix) {
            out.push_str("    ");
            out.push_str(replacement);
            out.push('\n');
            found = true;
        }
        else {
            out.push_str(line);
            out.push('\n');
        }
    }
    if !found {
        return Err(miette::miette!("InteractiveColPlot missing `{prefix}` binding"));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyar_analyzer::report::ColSeriesItem;

    use crate::ssg::project::{read_project_source, valkyrie_v_roots};

    fn legion_report_fixture() -> std::path::PathBuf {
        let roots = valkyrie_v_roots().unwrap();
        let nested = roots[0].join("projects/legion._/projects/legion.report");
        if nested.is_dir() && nested.join("legion.von").is_file() {
            return nested;
        }
        roots[0].join("projects/legion._/projects/legion.report")
    }

    #[test]
    fn expand_inlines_asgard_icol_from_report_project() {
        let project = legion_report_fixture();
        let roots = valkyrie_v_roots().unwrap();
        let Ok(icp) = load_interactive_col_plot(&roots)
        else {
            return;
        };
        let series = vec![ColSeriesItem {
            key: "pass".into(),
            label: "pass".into(),
            value_text: "1.0 tests".into(),
            fill: "#22c55e".into(),
            height_pct: 100.0,
        }];
        let wrapper = fill_chart_placeholders(
            &read_project_source(&project, "charts/chart-status.awsl").unwrap(),
            "\"Test Status\"",
            &series_to_init_literal(&series),
        );
        let awsl = expand_interactive_col_plot(&wrapper, &icp, "chart-status").unwrap();
        assert!(awsl.contains("asgard-icol"));
        assert!(awsl.contains("micro toggle"));
        assert!(!awsl.contains("test_status_series"));
    }
}
