//! 模板表达式依赖分析（Solid 细粒度订阅）。

use super::lower::ScriptBinding;

/// 从 AWSL/JS 风格表达式中提取标识符（含函数调用名）。
pub fn extract_expr_idents(expr: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut in_string = None::<char>;
    for ch in expr.chars() {
        if let Some(quote) = in_string {
            if ch == quote {
                in_string = None;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = Some(ch);
            current.clear();
            continue;
        }
        if ch.is_ascii_alphanumeric() || ch == '_' {
            current.push(ch);
            continue;
        }
        if !current.is_empty() {
            if !is_js_keyword(&current) {
                out.push(current.clone());
            }
            current.clear();
        }
    }
    if !current.is_empty() && !is_js_keyword(&current) {
        out.push(current);
    }
    out.sort();
    out.dedup();
    out
}

/// 表达式依赖的响应式 `let mut` 绑定名。
pub fn reactive_deps(expr: &str, bindings: &[ScriptBinding]) -> Vec<String> {
    let reactive_names: std::collections::BTreeSet<String> =
        bindings.iter().filter(|binding| binding.reactive).map(|binding| binding.name.clone()).collect();
    extract_expr_idents(expr).into_iter().filter(|name| reactive_names.contains(name)).collect()
}

fn is_js_keyword(word: &str) -> bool {
    matches!(word, "true" | "false" | "null" | "undefined" | "if" | "else" | "return" | "new" | "typeof" | "in" | "not" | "and" | "or")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_idents_from_expr() {
        let idents = extract_expr_idents("filter == 'all' ? 'active' : ''");
        assert!(idents.contains(&"filter".to_string()));
    }

    #[test]
    fn reactive_deps_filters_mut_bindings() {
        let bindings = vec![
            ScriptBinding {
                name: "filter".into(),
                init_expr: "\"all\"".into(),
                kind: super::super::lower::BindingKind::ReactiveState,
                reactive: true,
                sig_var: "filter_sig".into(),
                value_type: super::super::lower::SignalValueType::Utf8,
            },
            ScriptBinding {
                name: "label".into(),
                init_expr: "\"x\"".into(),
                kind: super::super::lower::BindingKind::LocalConst,
                reactive: false,
                sig_var: String::new(),
                value_type: super::super::lower::SignalValueType::Utf8,
            },
        ];
        let deps = reactive_deps("filter == 'all'", &bindings);
        assert_eq!(deps, vec!["filter"]);
    }
}
