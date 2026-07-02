//! AWSL SSG 报告渲染 + asgard hydrate 图表岛。

use std::path::Path;

use miette::Result;
use serde_json::json;
use voa::ssg::generate_static_page;

use super::{
    assets::{render_report_page, report_base_css},
    codegen::{charts_for_bench_report, charts_for_coverage_report, charts_for_test_report},
    islands::{asgard_boot_head_tags, asgard_start_snippet, emit_chart_islands},
    models::{BenchReport, CoverageReport, TestResultEntry},
    write::atomic_write_all_text,
};

const CHART_STATUS: &str = "LEGION_CHART_STATUS";
const CHART_COVERAGE_BALANCE: &str = "LEGION_CHART_COVERAGE_BALANCE";
const CHART_COVERAGE_FEATURES: &str = "LEGION_CHART_COVERAGE_FEATURES";
const CHART_BENCH_RUNTIME: &str = "LEGION_CHART_BENCH_RUNTIME";
const CHART_BENCH_COMPILE: &str = "LEGION_CHART_BENCH_COMPILE";

/// 渲染测试报告到 `{output_dir}/index.html`。
pub fn render_test_report(output_dir: &Path, project_name: &str, results: &[TestResultEntry]) -> Result<()> {
    let passed = results.iter().filter(|r| r.status == "pass").count();
    let failed = results.iter().filter(|r| r.status == "fail" || r.status == "compile_error").count();
    let skipped = results.iter().filter(|r| r.status == "skip").count();

    let rows: Vec<_> = results
        .iter()
        .map(|r| {
            let status_label = match r.status.as_str() {
                "pass" => "通过",
                "fail" => "失败",
                "skip" => "跳过",
                "compile_error" => "编译错误",
                other => other,
            };
            json!({
                "name": r.name,
                "status_label": status_label,
                "badge_class": format!("status-badge {}", r.status),
                "target": r.target,
                "error": r.error.clone().unwrap_or_default(),
            })
        })
        .collect();

    let charts = charts_for_test_report(results);
    let slots = emit_chart_islands(output_dir, "legion-test", &charts)?;
    let has_chart = !slots.is_empty();

    let page_title = format!("{project_name} - Test Report");
    let body_data = json!({
        "passed": passed,
        "failed": failed,
        "skipped": skipped,
        "has_rows": !results.is_empty(),
        "has_chart": has_chart,
        "rows": rows,
    });

    let html = render_page(&page_title, "pages/test-report.awsl", "TestReport", &body_data, has_chart, "legion-test")?;
    let html = inject_islands(html, &slots, CHART_STATUS);
    atomic_write_all_text(&output_dir.join("index.html"), &html)?;
    Ok(())
}

/// 渲染覆盖率报告到 `{output_dir}/index.html`。
pub fn render_coverage_report(output_dir: &Path, report: &CoverageReport) -> Result<()> {
    let pct = report.percentage;
    let bar_class = if pct >= 70.0 {
        "progress-fill high"
    }
    else if pct >= 40.0 {
        "progress-fill medium"
    }
    else {
        "progress-fill low"
    };
    let pct_label = format!("{pct:.0}%");
    let summary_text = format!("{}/{}（{pct:.0}%）", report.covered, report.total);

    let features: Vec<_> = report
        .features
        .iter()
        .map(|f| {
            let projects_text = if !f.covered {
                "-".to_string()
            }
            else if f.projects.len() > 3 {
                format!("{} ... +{}", f.projects.iter().take(3).cloned().collect::<Vec<_>>().join(", "), f.projects.len() - 3)
            }
            else {
                f.projects.join(", ")
            };
            json!({
                "display": f.display,
                "row_class": if f.covered { "covered" } else { "uncovered" },
                "status_class": if f.covered { "status-covered" } else { "status-uncovered" },
                "status": if f.covered { "✓" } else { "✗" },
                "projects_text": projects_text,
            })
        })
        .collect();

    let charts = charts_for_coverage_report(report);
    let slots = emit_chart_islands(output_dir, "legion-coverage", &charts)?;
    let has_charts = !slots.is_empty();

    let body_data = json!({
        "summary_text": summary_text,
        "bar_class": bar_class,
        "bar_style": format!("width: {pct:.0}%"),
        "pct_label": pct_label,
        "has_charts": has_charts,
        "features": features,
    });

    let html = render_page("Coverage Report", "pages/coverage-report.awsl", "CoverageReport", &body_data, has_charts, "legion-coverage")?;
    let html = inject_islands(html, &slots, CHART_COVERAGE_BALANCE);
    let html = inject_islands(html, &slots, CHART_COVERAGE_FEATURES);
    atomic_write_all_text(&output_dir.join("index.html"), &html)?;
    Ok(())
}

/// 渲染基准报告到 `{output_dir}/index.html`。
pub fn render_bench_report(output_dir: &Path, report: &BenchReport) -> Result<()> {
    if report.rows.is_empty() {
        return Ok(());
    }

    let mut rows = report.rows.clone();
    rows.sort_by(|a, b| a.project.cmp(&b.project).then(a.test.cmp(&b.test)).then(a.target.cmp(&b.target)));

    let projects = {
        let mut names: Vec<_> = rows.iter().map(|r| r.project.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        names.len()
    };

    let summary_text = format!("{} 项基准测试 · {projects} 个项目 · 每项运行 {} 次", rows.len(), report.runs);

    let row_data: Vec<_> = rows
        .iter()
        .map(|r| {
            json!({
                "project": r.project,
                "test": r.test,
                "target": r.target,
                "compile_ms": format!("{:.1}", r.compile_ms),
                "runtime_ms": format!("{:.1}", r.runtime_ms),
            })
        })
        .collect();

    let charts = charts_for_bench_report(&rows);
    let slots = emit_chart_islands(output_dir, "legion-bench", &charts)?;
    let has_charts = !slots.is_empty();

    let body_data = json!({
        "summary_text": summary_text,
        "has_charts": has_charts,
        "rows": row_data,
    });

    let html = render_page("Benchmark Report", "pages/bench-report.awsl", "BenchReport", &body_data, has_charts, "legion-bench")?;
    let html = inject_islands(html, &slots, CHART_BENCH_RUNTIME);
    let html = inject_islands(html, &slots, CHART_BENCH_COMPILE);
    atomic_write_all_text(&output_dir.join("index.html"), &html)?;
    Ok(())
}

fn inject_islands(mut html: String, slots: &[(String, String)], token: &str) -> String {
    let slot_marker = format!("<p>{token}</p>");
    let replacement = slots.iter().find(|(route, _)| token_matches_route(token, route)).map(|(_, html)| html.as_str()).unwrap_or("");
    html = html.replace(&slot_marker, replacement);
    html
}

fn token_matches_route(token: &str, route: &str) -> bool {
    match token {
        CHART_STATUS => route == "chart-status",
        CHART_COVERAGE_BALANCE => route == "chart-coverage-balance",
        CHART_COVERAGE_FEATURES => route == "chart-coverage-features",
        CHART_BENCH_RUNTIME => route == "chart-bench-runtime",
        CHART_BENCH_COMPILE => route == "chart-bench-compile",
        _ => false,
    }
}

fn render_page(
    page_title: &str,
    body_awsl_relative: &str,
    body_component: &str,
    body_data: &serde_json::Value,
    has_charts: bool,
    module_stem: &str,
) -> Result<String> {
    let body = render_report_page(body_awsl_relative, body_component, body_data)?;
    let layout_data = json!({ "page_title": page_title });
    let layout = render_report_page("pages/layout.awsl", "layout", &layout_data)?;
    let fragment_html = layout.html.replace("<p>LEGION_REPORT_SLOT</p>", &body.html);

    let page_result = voa::codegen::StaticRenderResult {
        html: fragment_html,
        css: {
            let mut css = layout.css.clone();
            css.push('\n');
            css.push_str(&body.css);
            css
        },
        scope: layout.scope,
    };

    let base_css = report_base_css()?;
    let mut html = generate_static_page(page_title, &[page_result], &base_css);
    if has_charts {
        html = attach_asgard_boot(html, module_stem);
    }
    Ok(html)
}

/// 注入 asgard auto glue markup（stylesheet + `boot.js` src），禁止内联业务脚本。
fn attach_asgard_boot(html: String, module_stem: &str) -> String {
    let css_name = format!("{}.css", module_stem.replace('.', "-"));
    let head_tags = asgard_boot_head_tags(&css_name);
    let with_head = if html.contains("</head>") { html.replacen("</head>", &format!("{head_tags}</head>"), 1) } else { html };
    let boot_tag = asgard_start_snippet();
    if with_head.contains("</body>") {
        with_head.replacen("</body>", &format!("{boot_tag}</body>"), 1)
    }
    else {
        let mut out = with_head;
        out.push_str(boot_tag);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    use crate::cmds::report::models::{BenchResultRow, CoverageFeatureEntry};

    #[test]
    fn render_test_report_uses_asgard_islands_not_handwritten_js() {
        let dir = tempdir().unwrap();
        let results = vec![
            TestResultEntry { name: "add".into(), status: "pass".into(), error: None, target: "nyar".into() },
            TestResultEntry { name: "sub".into(), status: "fail".into(), error: Some("boom".into()), target: "nyar".into() },
        ];
        render_test_report(dir.path(), "demo", &results).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("demo - Test Report"));
        assert!(html.contains("data-island=\"hydrated\""));
        assert!(html.contains("data-component=\"chart-status\""));
        assert!(html.contains(r#"src="boot.js""#));
        assert!(!html.contains("__voa.start"));
        assert!(!html.contains("globalThis.__voa"));
        assert!(dir.path().join("c/chart-status.js").exists());
        assert!(dir.path().join("boot.js").exists());
        assert!(dir.path().join("manifest.json").exists());
        let boot = std::fs::read_to_string(dir.path().join("boot.js")).unwrap();
        assert!(boot.contains("__voa"));
        assert!(boot.contains("start('manifest.json')"));
        let glue = std::fs::read_to_string(dir.path().join("c/chart-status.js")).unwrap();
        assert!(glue.contains("callExport"));
        assert!(!glue.contains("innerHTML"));
        assert!(!boot.contains("plotter-interactive"));
        assert!(!html.contains("plotter-interactive"));
        assert!(!html.contains("asgard-runtime.js"));
        assert!(!html.contains(CHART_STATUS));
        // UI comes from asgard.plotter (styles packaged), not Rust-synthesized charts.
        let css_path = dir.path().join("legion-test.css");
        if css_path.exists() {
            let css = std::fs::read_to_string(css_path).unwrap();
            assert!(css.contains("asgard-icol") || css.contains("asgard-icol-bar"));
        }
    }

    #[test]
    fn render_coverage_report_emits_islands() {
        let dir = tempdir().unwrap();
        let report = CoverageReport {
            covered: 1,
            total: 2,
            percentage: 50.0,
            features: vec![
                CoverageFeatureEntry { feature: "micro".into(), display: "micro 函数".into(), covered: true, projects: vec!["app".into()] },
                CoverageFeatureEntry { feature: "match".into(), display: "模式匹配".into(), covered: false, projects: vec![] },
            ],
        };
        render_coverage_report(dir.path(), &report).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("Coverage Report"));
        assert!(html.contains("data-component=\"chart-coverage-balance\""));
        assert!(html.contains("data-component=\"chart-coverage-features\""));
        assert!(html.contains("boot.js"));
        assert!(!html.contains("plotter-interactive"));
    }

    #[test]
    fn render_bench_report_emits_islands() {
        let dir = tempdir().unwrap();
        let report = BenchReport {
            runs: 3,
            rows: vec![BenchResultRow { project: "app".into(), test: "fib".into(), target: "nyar".into(), compile_ms: 12.5, runtime_ms: 1.2 }],
        };
        render_bench_report(dir.path(), &report).unwrap();
        let html = std::fs::read_to_string(dir.path().join("index.html")).unwrap();
        assert!(html.contains("Benchmark Report"));
        assert!(html.contains("data-component=\"chart-bench-runtime\""));
        assert!(html.contains("data-component=\"chart-bench-compile\""));
        assert!(dir.path().join("boot.js").exists());
        assert!(!html.contains("plotter-interactive"));
    }
}
