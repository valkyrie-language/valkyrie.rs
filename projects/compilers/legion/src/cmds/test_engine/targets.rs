//! 测试目标解析。

use miette::{Result, miette};
use nyar_language::CanonicalTarget;

/// 解析 `--target` 参数（默认 `legion`，`all` 展开为 legion/clr/jvm/node）。
pub fn resolve_test_targets(target: Option<&str>) -> Vec<String> {
    match target {
        None | Some("") => vec!["legion".into()],
        Some(value) if value.eq_ignore_ascii_case("all") => {
            vec!["legion".into(), "clr".into(), "jvm".into(), "node".into()]
        }
        Some(value) => value.split(',').map(str::trim).filter(|item| !item.is_empty()).map(|item| item.to_ascii_lowercase()).collect(),
    }
}

/// 将短标签解析为 `CanonicalTarget`。
pub fn parse_target_label(label: &str) -> Result<CanonicalTarget> {
    label.parse::<CanonicalTarget>().map_err(|error| miette!("无法解析目标 '{label}': {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_nyar() {
        assert_eq!(resolve_test_targets(None), vec!["legion"]);
    }

    #[test]
    fn expands_all() {
        assert_eq!(resolve_test_targets(Some("all")), vec!["legion", "clr", "jvm", "node"]);
    }

    #[test]
    fn splits_list() {
        assert_eq!(resolve_test_targets(Some("legion,clr")), vec!["legion", "clr"]);
        assert_eq!(resolve_test_targets(Some("nyar,node")), vec!["nyar", "node"]);
    }

    #[test]
    fn legion_alias_parses_to_nyar_target() {
        let target = parse_target_label("legion").expect("legion");
        assert_eq!(target.to_string(), "nyar-unknown-unknown-managed");
    }

    #[test]
    fn nyar_vm_alias_parses_to_nyar_target() {
        let target = parse_target_label("nyar-vm").expect("nyar-vm");
        assert_eq!(target.to_string(), "nyar-unknown-unknown-managed");
    }
}
