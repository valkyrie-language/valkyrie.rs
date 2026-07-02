//! 从 RenderIR / script 静态收集 Tailwind `@style` utility class。

use std::{collections::BTreeSet, path::Path};

use miette::{IntoDiagnostic, Result, WrapErr};

use crate::awsl::{
    LoweredComponent,
    expr_util::{collect_utility_tokens_from_expr, push_whitespace_tokens},
    render_ir::{RenderAttrValue, RenderIr, RenderModule, RenderNode, RenderNodeId, RenderRegionId, RenderTextSegment, region_nodes},
};

/// 去重后的 Tailwind utility class 集合。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleCollector {
    utilities: BTreeSet<String>,
}

impl StyleCollector {
    /// 新建空收集器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 按空白切分并登记 utility token。
    pub fn push(&mut self, classes: &str) {
        let mut tokens = Vec::new();
        push_whitespace_tokens(classes, &mut tokens);
        for token in tokens {
            self.utilities.insert(token);
        }
    }

    /// 合并另一收集器。
    pub fn merge(&mut self, other: &StyleCollector) {
        self.utilities.extend(other.utilities.iter().cloned());
    }

    /// 已收集的 utility（字典序）。
    pub fn utilities(&self) -> impl Iterator<Item = &str> {
        self.utilities.iter().map(String::as_str)
    }

    /// 写入 `dist/.asgard/tailwind-content.txt`（每行一个 class）。
    pub fn write_manifest(&self, output_dir: &Path) -> Result<()> {
        write_content_manifest(self, output_dir)
    }
}

/// 从已降级组件收集 utility（RenderIR + script 内 `StyleCollector.push` 字面量）。
pub fn collect_from_components(components: &[LoweredComponent]) -> StyleCollector {
    let mut collector = StyleCollector::new();
    for component in components {
        collect_from_ir(&component.render_ir, &mut collector);
        if let Some(script) = &component.script_source {
            collect_push_calls_from_script(script, &mut collector);
        }
    }
    collector
}

/// 遍历 RenderIR，保守收集 `class` 属性中的 utility。
pub fn collect_from_ir(module: &RenderIr, sink: &mut StyleCollector) {
    for &root_id in &module.roots {
        collect_from_node(module, root_id, sink);
    }
}

fn collect_from_region(module: &RenderModule, region: RenderRegionId, sink: &mut StyleCollector) {
    for &node_id in region_nodes(module, region) {
        collect_from_node(module, node_id, sink);
    }
}

fn collect_from_node(module: &RenderModule, node_id: RenderNodeId, sink: &mut StyleCollector) {
    let node = module.node(node_id);
    match node {
        RenderNode::Element(element) => {
            for attr in &element.attrs {
                if attr.name == "class" {
                    collect_from_attr_value(module, &attr.value, sink);
                }
            }
            collect_from_region(module, element.children, sink);
        }
        RenderNode::Component(component) => {
            for attr in &component.attrs {
                if attr.name == "class" {
                    collect_from_attr_value(module, &attr.value, sink);
                }
            }
            collect_from_region(module, component.children, sink);
        }
        RenderNode::If(render_if) => {
            collect_from_region(module, render_if.then_region, sink);
            collect_from_region(module, render_if.else_region, sink);
        }
        RenderNode::Loop(render_loop) => collect_from_region(module, render_loop.body_region, sink),
        RenderNode::Fragment(fragment) => collect_from_region(module, fragment.children, sink),
        RenderNode::Text { .. } => {}
    }
}

fn collect_from_attr_value(module: &RenderModule, value: &RenderAttrValue, sink: &mut StyleCollector) {
    match value {
        RenderAttrValue::Static(text) => push_tokens_from_expr(text, sink),
        RenderAttrValue::Expr(expr_id) => push_tokens_from_expr(module.expr_source(*expr_id), sink),
        RenderAttrValue::Template(segments) => {
            for segment in segments {
                match segment {
                    RenderTextSegment::Static(text) => sink.push(text),
                    RenderTextSegment::Expr(expr_id) => push_tokens_from_expr(module.expr_source(*expr_id), sink),
                }
            }
        }
    }
}

fn push_tokens_from_expr(expr: &str, sink: &mut StyleCollector) {
    let mut tokens = Vec::new();
    collect_utility_tokens_from_expr(expr, &mut tokens);
    if tokens.is_empty() {
        sink.push(expr);
    }
    else {
        for token in tokens {
            sink.utilities.insert(token);
        }
    }
}

/// 从 `<script>` 文本扫描 `StyleCollector.push("...")` / `StyleCollector.push('...')` 字面量。
pub fn collect_push_calls_from_script(script: &str, sink: &mut StyleCollector) {
    let bytes = script.as_bytes();
    let needle = b"StyleCollector.push(";
    let mut index = 0usize;
    while index + needle.len() <= bytes.len() {
        if &bytes[index..index + needle.len()] != needle {
            index += 1;
            continue;
        }
        let mut cursor = index + needle.len();
        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        if cursor >= bytes.len() {
            break;
        }
        let quote = bytes[cursor];
        if quote != b'"' && quote != b'\'' {
            index += needle.len();
            continue;
        }
        cursor += 1;
        let start = cursor;
        let mut escaped = false;
        while cursor < bytes.len() {
            let ch = bytes[cursor];
            if escaped {
                escaped = false;
                cursor += 1;
                continue;
            }
            if ch == b'\\' {
                escaped = true;
                cursor += 1;
                continue;
            }
            if ch == quote {
                break;
            }
            cursor += 1;
        }
        if cursor >= bytes.len() || bytes[cursor] != quote {
            index += needle.len();
            continue;
        }
        if let Ok(literal) = std::str::from_utf8(&bytes[start..cursor]) {
            sink.push(literal);
        }
        index = cursor + 1;
    }
}

/// 写入 tailwind content manifest。
pub fn write_content_manifest(collector: &StyleCollector, output_dir: &Path) -> Result<()> {
    let manifest_dir = output_dir.join(".asgard");
    std::fs::create_dir_all(&manifest_dir).into_diagnostic().wrap_err("创建 .asgard 目录失败")?;
    let txt_path = manifest_dir.join("tailwind-content.txt");
    let content = collector.utilities().collect::<Vec<_>>().join("\n");
    std::fs::write(&txt_path, content).into_diagnostic().wrap_err("写入 tailwind-content.txt 失败")?;
    let json_path = manifest_dir.join("tailwind.content.json");
    let json = serde_json::json!({ "utilities": collector.utilities().collect::<Vec<_>>() });
    std::fs::write(json_path, serde_json::to_string_pretty(&json).unwrap_or_else(|_| "{}".into()))
        .into_diagnostic()
        .wrap_err("写入 tailwind.content.json 失败")?;
    Ok(())
}

use std::sync::{Mutex, OnceLock};

static RUNTIME_UTILITIES: OnceLock<Mutex<BTreeSet<String>>> = OnceLock::new();

fn runtime_registry() -> &'static Mutex<BTreeSet<String>> {
    RUNTIME_UTILITIES.get_or_init(|| Mutex::new(BTreeSet::new()))
}

/// 运行时登记 Tailwind utility（`StyleCollector.push` / dev safelist）。
pub fn style_collector_push(classes: &str) {
    let Ok(mut guard) = runtime_registry().lock()
    else {
        return;
    };
    let mut tokens = Vec::new();
    push_whitespace_tokens(classes, &mut tokens);
    for token in tokens {
        guard.insert(token);
    }
}

/// 取出并清空运行时登记的 utility（dev server 热重载可选合并）。
pub fn take_runtime_utilities() -> BTreeSet<String> {
    let Ok(mut guard) = runtime_registry().lock()
    else {
        return BTreeSet::new();
    };
    std::mem::take(&mut *guard)
}

#[cfg(test)]
mod collector_tests {
    use super::*;
    use crate::awsl::{LoweringOptions, RenderAttrValue, RenderNode, lower_component};
    use std_data::text::awsl::AwslParser;

    fn first_element_attrs(lowered: &crate::awsl::LoweredComponent) -> Option<&[crate::awsl::RenderAttr]> {
        let module = &lowered.render_ir;
        let root = module.roots.first().copied()?;
        match module.node(root) {
            RenderNode::Element(element) => Some(&element.attrs),
            RenderNode::Component(component) => Some(&component.attrs),
            _ => None,
        }
    }

    #[test]
    fn collects_static_style_utilities() {
        let source = r#"<widget><div @style="flex w-4"></div></widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let lowered = lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default());
        let collector = collect_from_components(&[lowered]);
        assert!(collector.utilities().any(|u| u == "flex"));
        assert!(collector.utilities().any(|u| u == "w-4"));
    }

    #[test]
    fn style_lowers_to_class_not_style_attr() {
        let source = r#"<widget><div @style="flex"></div></widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let lowered = lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default());
        let tag = first_element_attrs(&lowered).expect("tag");
        assert!(tag.iter().any(|attr| attr.name == "class"));
        assert!(!tag.iter().any(|attr| attr.name == "style"));
    }

    #[test]
    fn collects_script_push_literals() {
        let source = r#"<widget></widget>
<script>
StyleCollector.push("foo bar")
</script>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let lowered = lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default());
        let collector = collect_from_components(&[lowered]);
        assert!(collector.utilities().any(|u| u == "foo"));
        assert!(collector.utilities().any(|u| u == "bar"));
    }

    #[test]
    fn collects_if_branches() {
        let source = r#"<widget>
<if :condition="show">
    <div @style="flex p-2"></div>
<else/>
    <div @style="hidden"></div>
</if>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let lowered = lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default());
        let collector = collect_from_components(&[lowered]);
        assert!(collector.utilities().any(|u| u == "flex"));
        assert!(collector.utilities().any(|u| u == "p-2"));
        assert!(collector.utilities().any(|u| u == "hidden"));
    }

    #[test]
    fn mixed_template_collects_static_segments() {
        let source = r#"<widget><div @style="ui-{size} active"></div></widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let lowered = lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default());
        let tag = first_element_attrs(&lowered).expect("tag");
        let class_attr = tag.iter().find(|attr| attr.name == "class").expect("class");
        assert!(matches!(class_attr.value, RenderAttrValue::Template(_)));
        let collector = collect_from_components(&[lowered]);
        assert!(collector.utilities().any(|u| u == "ui-"));
        assert!(collector.utilities().any(|u| u == "active"));
    }
}
