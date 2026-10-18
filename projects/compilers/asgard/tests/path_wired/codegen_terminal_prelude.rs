//! Migrated from `src/codegen/terminal_prelude.rs` (`tests`).

use super::*;

#[test]
fn host_prelude_declares_all_primitives() {
    for sym in [
        "micro clear",
        "micro set_foreground",
        "micro set_background",
        "micro reset_color",
        "micro move_to",
        "micro size_rows",
        "micro size_cols",
        "micro poll_key",
        "micro enter_alt_screen",
        "micro exit_alt_screen",
        "micro flush",
        "micro put_char",
        "micro put_str",
        "micro set_reverse",
        "micro sleep_ms",
    ] {
        assert!(TERMINAL_HOST_PRELUDE.contains(sym), "prelude 缺少声明 {sym}");
    }
}

#[test]
fn tui_runtime_source_embedded() {
    assert!(TUI_RUNTIME_SOURCE.contains("structure TuiRuntime"), "tui.v 未嵌入");
    assert!(TUI_RUNTIME_SOURCE.contains("micro widget_column"), "widget_column 缺失");
    assert!(TUI_RUNTIME_SOURCE.contains("micro asgard_terminal_run"), "主入口缺失");
}

#[test]
fn terminal_prelude_combines_both() {
    let combined = terminal_prelude();
    assert!(combined.contains("[host_contract] micro clear"), "host_contract 缺失");
    assert!(combined.contains("structure TuiRuntime"), "运行时缺失");
}
