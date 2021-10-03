mod support;

use std::path::{Path, PathBuf};

use support::oop_fixture::{can_run_oop_fixtures, collect_oop_fixture_cases, verify_oop_fixture};

#[test]
fn runs_oop_fixtures_on_clr() {
    let fixtures_root = oop_fixture_root();
    let fixtures = collect_oop_fixture_cases(&fixtures_root);
    assert!(!fixtures.is_empty(), "no oop fixtures found under '{}'", fixtures_root.display());

    if !can_run_oop_fixtures(&fixtures) {
        return;
    }

    for fixture in &fixtures {
        verify_oop_fixture(fixture);
    }

    if support::runtime_fixture::regenerate_enabled() {
        eprintln!("oop fixtures regenerated under {}", fixtures_root.display());
    }
}

/// 仅运行 singleton fixture 的集成测试，用于 P0 CLR 闭环验证。
///
/// 与 `runs_oop_fixtures_on_clr` 的区别在于只挑选 `oop/singleton` 目录下的 fixture，
/// 避免其他尚未对齐的 OOP fixture（如 `access_control` 的 match pattern）阻塞
/// singleton 闭环验证。
#[test]
fn runs_singleton_oop_fixture_on_clr() {
    let singleton_dir = oop_fixture_root().join("singleton");
    let fixtures = collect_oop_fixture_cases(&singleton_dir);
    assert!(!fixtures.is_empty(), "no singleton fixtures found under '{}'", singleton_dir.display());

    if !can_run_oop_fixtures(&fixtures) {
        eprintln!("skip singleton oop fixture: dotnet unavailable");
        return;
    }

    for fixture in &fixtures {
        verify_oop_fixture(fixture);
    }

    if support::runtime_fixture::regenerate_enabled() {
        eprintln!("singleton oop fixtures regenerated under {}", singleton_dir.display());
    }
}

fn oop_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("oop")
}
