use std::{fs, path::PathBuf};

use nyar::{assert_or_regenerate_text_sidecar, collect_fixture_cases_with_extensions, regenerate_enabled};

use super::dump::{
    lex::dump_lex_snapshot,
    parse::{dump_parse_snapshot, dump_parse_vx_snapshot},
};

/// `legion` 持有的 Valkyrie 文本快照 fixture 根目录。
pub fn legion_text_fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../legion/tests/fixtures/text/valkyrie")
}

fn is_vx_fixture(fixture_path: &PathBuf) -> bool {
    fixture_path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("vx"))
}

#[test]
fn text_fixture_lex_and_parse_regression() {
    let root = legion_text_fixtures_root();
    if !root.exists() {
        panic!("fixture root not found: {}", root.display());
    }

    let regenerate = regenerate_enabled();
    let cases = collect_fixture_cases_with_extensions(&root, &["v", "vx"]);

    assert!(!cases.is_empty(), "no .v/.vx fixtures found under {}", root.display());

    for fixture_path in cases {
        let source =
            fs::read_to_string(&fixture_path).unwrap_or_else(|error| panic!("failed to read fixture '{}': {}", fixture_path.display(), error));

        let lex_observed = dump_lex_snapshot(&source);
        assert_or_regenerate_text_sidecar(&fixture_path, "lex", &lex_observed, regenerate);

        let parse_observed = if is_vx_fixture(&fixture_path) { dump_parse_vx_snapshot(&source) } else { dump_parse_snapshot(&source) };
        assert_or_regenerate_text_sidecar(&fixture_path, "parse", &parse_observed, regenerate);
    }
}
