//! RenderIR → 静态 HTML（SSG / 文档站，无 hydration）。

use std::fmt::Write as _;

use serde_json::Value;
use std_data::text::awsl::is_html_void_element;

use crate::{
    awsl::{
        LoweredComponent, RenderAttr, RenderAttrValue, RenderLoopNode, RenderModule, RenderNode, RenderNodeId, RenderRegionId,
        RenderTextSegment, ScriptBinding, TemplateNodeKind,
        render_ir::{attr_value_source, region_nodes},
    },
    codegen::expr_eval::ExprContext,
};

/// 静态 HTML 渲染结果。
#[derive(Debug, Clone)]
pub struct StaticRenderResult {
    /// 组件 HTML 片段（含根 `div` 包装）。
    pub html: String,
    /// 作用域 CSS。
    pub css: String,
    /// 作用域 class 前缀。
    pub scope: String,
}

/// 将已降级的组件 RenderIR 渲染为静态 HTML。
pub fn render_static_ir(component: &LoweredComponent, data: &Value) -> StaticRenderResult {
    let scope = format!("asgard-static-{}", to_kebab_case(&component.name));
    let mut html_body = String::new();
    let mut ctx = HtmlRenderCtx::new(component, data, &scope);
    ctx.emit_roots(&mut html_body, &component.render_ir);

    let css = component.style.as_deref().unwrap_or("").to_string();
    let html = format!(
        "<div class=\"{scope}\" data-asgard-static=\"{name}\">\n{html_body}\n</div>",
        scope = scope,
        name = escape_attr(&component.route_name),
        html_body = html_body.trim_end()
    );

    StaticRenderResult { html, css, scope }
}

struct HtmlRenderCtx<'a> {
    module: &'a RenderModule,
    expr: ExprContext,
    scope: &'a str,
}

impl<'a> HtmlRenderCtx<'a> {
    fn new(component: &'a LoweredComponent, data: &Value, scope: &'a str) -> Self {
        Self { module: &component.render_ir, expr: ExprContext::from_bindings(&component.script_bindings, data), scope }
    }

    fn emit_roots(&mut self, out: &mut String, module: &RenderModule) {
        self.emit_region(out, module, &module.roots);
    }

    fn emit_region(&mut self, out: &mut String, module: &RenderModule, node_ids: &[RenderNodeId]) {
        for &node_id in node_ids {
            self.emit_node(out, module, node_id);
        }
    }

    fn emit_region_id(&mut self, out: &mut String, module: &RenderModule, region: RenderRegionId) {
        self.emit_region(out, module, region_nodes(module, region));
    }

    fn emit_node(&mut self, out: &mut String, module: &RenderModule, node_id: RenderNodeId) {
        match module.node(node_id) {
            RenderNode::Element(element) => {
                let tag = &element.tag;
                let kind = element.kind;
                if matches!(kind, TemplateNodeKind::Intrinsic) && tag.eq_ignore_ascii_case("slot") {
                    return;
                }
                if tag.eq_ignore_ascii_case("head") || tag.eq_ignore_ascii_case("script") {
                    return;
                }
                let html_tag = map_html_tag(tag, kind);
                write!(out, "<{html_tag}").unwrap();
                self.emit_attrs(out, module, &element.attrs);
                let children = region_nodes(module, element.children);
                if children.is_empty() && is_html_void_element(&html_tag) {
                    write!(out, " />").unwrap();
                }
                else {
                    write!(out, ">").unwrap();
                    self.emit_region(out, module, children);
                    write!(out, "</{html_tag}>").unwrap();
                }
            }
            RenderNode::Component(component) => {
                let html_tag = map_html_tag(&component.tag, TemplateNodeKind::Component);
                write!(out, "<{html_tag}").unwrap();
                self.emit_attrs(out, module, &component.attrs);
                write!(out, ">").unwrap();
                self.emit_region_id(out, module, component.children);
                write!(out, "</{html_tag}>").unwrap();
            }
            RenderNode::Text(text) => {
                let body = self.emit_text_segments(module, &text.segments);
                out.push_str(&escape_html(&body));
            }
            RenderNode::If(render_if) => {
                if self.expr.eval_truthy(module.expr_source(render_if.condition)) {
                    self.emit_region_id(out, module, render_if.then_region);
                }
                else {
                    self.emit_region_id(out, module, render_if.else_region);
                }
            }
            RenderNode::Loop(render_loop) => self.emit_loop(out, module, render_loop),
            RenderNode::Fragment(fragment) => self.emit_region_id(out, module, fragment.children),
        }
    }

    fn emit_loop(&mut self, out: &mut String, module: &RenderModule, render_loop: &RenderLoopNode) {
        let items = self.expr.eval_value(module.expr_source(render_loop.items));
        let Some(array) = items.as_array()
        else {
            return;
        };
        for (index, item) in array.iter().enumerate() {
            self.expr.push_loop(&render_loop.item_var, &render_loop.index_var, item.clone(), index);
            self.emit_region_id(out, module, render_loop.body_region);
            self.expr.pop_loop();
        }
    }

    fn emit_attrs(&self, out: &mut String, module: &RenderModule, attrs: &[RenderAttr]) {
        for attr in attrs {
            if attr.is_event {
                continue;
            }
            let name = normalize_attr_name(&attr.name);
            if name.is_empty() {
                continue;
            }
            let value = match &attr.value {
                RenderAttrValue::Static(text) => text.clone(),
                RenderAttrValue::Expr(expr_id) => self.expr.eval_string(module.expr_source(*expr_id)),
                RenderAttrValue::Template(segments) => self.emit_text_segments(module, segments),
            };
            if !value.is_empty() || name == "class" || name == "href" {
                write!(out, " {name}=\"{}\"", escape_attr(&value)).unwrap();
            }
        }
    }

    fn emit_text_segments(&self, module: &RenderModule, segments: &[RenderTextSegment]) -> String {
        let mut out = String::new();
        for segment in segments {
            match segment {
                RenderTextSegment::Static(text) => out.push_str(text),
                RenderTextSegment::Expr(expr_id) => out.push_str(&self.expr.eval_string(module.expr_source(*expr_id))),
            }
        }
        out
    }
}

fn map_html_tag(tag: &str, kind: TemplateNodeKind) -> String {
    match kind {
        TemplateNodeKind::Component if tag.eq_ignore_ascii_case("link") => "a".into(),
        TemplateNodeKind::Component => "div".into(),
        TemplateNodeKind::Intrinsic => match tag {
            "Text" | "text" => "span".into(),
            "Button" | "button" => "button".into(),
            "Column" | "column" | "Box" | "box" | "Row" | "row" | "Flex" | "flex" => "div".into(),
            "Slot" | "slot" => "div".into(),
            other => other.to_ascii_lowercase(),
        },
        TemplateNodeKind::HostView => tag.to_ascii_lowercase(),
    }
}

fn normalize_attr_name(name: &str) -> String {
    match name {
        "href" | "class" | "id" | "title" | "rel" | "target" | "type" | "name" | "content" => name.to_string(),
        n if n.starts_with(':') => n[1..].to_string(),
        n if n.starts_with('@') => String::new(),
        n if n == "on:click" || n.starts_with("on:") => String::new(),
        n => n.to_string(),
    }
}

fn to_kebab_case(name: &str) -> String {
    let mut out = String::new();
    for (index, ch) in name.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
        }
        else if ch == '_' {
            out.push('-');
        }
        else {
            out.push(ch);
        }
    }
    out
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn escape_attr(text: &str) -> String {
    escape_html(text).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awsl::{LoweringOptions, lower_component};
    use std_data::text::awsl::AwslParser;

    #[test]
    fn static_render_simple_component() {
        let source = r#"<widget>
    <div>Hello World</div>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "HelloWorld", "hello.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &Value::Null);
        assert!(result.html.contains("Hello World"));
        assert!(result.html.contains("data-asgard-static=\"HelloWorld\""));
        assert_eq!(result.scope, "asgard-static-hello-world");
    }

    #[test]
    fn static_render_interpolation() {
        let source = r#"<widget>
    <h1>{title}</h1>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Page", "page.awsl", &LoweringOptions::default());
        let data = serde_json::json!({ "title": "文档首页" });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("文档首页"));
    }

    #[test]
    fn static_render_loop_items() {
        let source = r#"<widget>
<loop item in items>
    <a :href="item.href">{item.title}</a>
</loop>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "List", "list.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "items": [
                { "href": "/a", "title": "A" },
                { "href": "/b", "title": "B" }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("href=\"/a\""));
        assert!(result.html.contains(">A<"));
        assert!(result.html.contains("href=\"/b\""));
    }

    #[test]
    fn static_render_link_component() {
        let source = r#"<widget>
    <Link href="/docs">文档</Link>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Nav", "nav.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &Value::Null);
        assert!(result.html.contains("<a href=\"/docs\">"));
    }

    #[test]
    fn static_render_if_else_from_script_binding() {
        let source = r#"<widget>
    <if visible>
        <div>Shown</div>
    <else/>
        <div>Hidden</div>
    </if>
</widget>
<script>
    let visible: bool = false
</script>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Panel", "panel.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &Value::Null);
        assert!(result.html.contains("Hidden"));
        assert!(!result.html.contains(">Shown<"));
    }

    #[test]
    fn static_render_loop_string_items() {
        let source = r#"<widget>
<loop item in items>
    <span>{item}</span>
</loop>
</widget>
<script>
let items: list = ["a", "b", "c"]
</script>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Tags", "tags.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &serde_json::json!({ "items": ["a", "b", "c"] }));
        assert!(result.html.contains(">a<"));
        assert!(result.html.contains(">b<"));
        assert!(result.html.contains(">c<"));
    }

    #[test]
    fn static_render_class_ternary() {
        let source = r#"<widget>
<a href="/x" @class="active ? 'active' : ''">Link</a>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Nav", "nav.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &serde_json::json!({ "active": true }));
        assert!(result.html.contains("class=\"active\""));
    }

    #[test]
    fn static_render_nav_sections_nested() {
        let source = r#"<widget doc_page>
<loop section in sections>
    <div class="vp-nav-section">
        <span class="vp-nav-section-title">{section.title}</span>
        <loop page in section.pages>
            <a :href="page.href" @class="page.link_class">{page.title}</a>
        </loop>
    </div>
</loop>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "doc-page", "doc-page.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "sections": [
                {
                    "title": "指南",
                    "pages": [
                        { "title": "工作流", "href": "workflow.html", "link_class": "vp-nav-link active" },
                        { "title": "架构", "href": "architecture.html", "link_class": "vp-nav-link" }
                    ]
                }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("vp-nav-section-title"));
        assert!(result.html.contains("指南"));
        assert!(result.html.contains("vp-nav-link active"));
        assert!(result.html.contains("工作流"));
    }

    #[test]
    fn static_render_is_active_helper() {
        let source = r#"<widget>
<loop page in pages>
    <a :href="page.href" @class="isActive(page.href) ? 'active' : ''">{page.title}</a>
</loop>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Nav", "nav.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "__current_path": "workflow.html",
            "pages": [
                { "title": "工作流", "href": "workflow.html" },
                { "title": "架构", "href": "architecture.html" }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("class=\"active\""));
        assert!(result.html.contains("工作流"));
        assert!(result.html.contains("href=\"architecture.html\""));
        assert!(!result.html.contains("architecture.html\" class=\"active\""));
    }

    #[test]
    fn static_render_string_concat_in_attr() {
        let source = r###"<widget>
<loop section in sections>
    <a :href="'#' + section.id">{section.title}</a>
</loop>
</widget>"###;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Nav", "nav.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "sections": [
                { "id": "button", "title": "Button" },
                { "id": "card", "title": "Card" }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("href=\"#button\""));
        assert!(result.html.contains(">Button<"));
    }

    #[test]
    fn static_render_if_hides_empty_section_title() {
        let source = r#"<widget>
    <loop section in sections>
        <if section.title != ''>
            <span>{section.title}</span>
        </if>
    </loop>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Side", "side.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "sections": [
                { "title": "指南", "pages": [] },
                { "title": "", "pages": [] }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("指南"));
        assert_eq!(result.html.matches("指南").count(), 1);
    }

    #[test]
    fn static_render_nav_items_with_active_class() {
        let source = r#"<widget doc_page>
<loop item in nav_items>
    <a :href="item.href" @class="item.link_class">{item.title}</a>
</loop>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "doc-page", "doc-page.awsl", &LoweringOptions::default());
        let data = serde_json::json!({
            "nav_items": [
                { "title": "工作流", "href": "workflow.html", "link_class": "vp-nav-link active" },
                { "title": "架构", "href": "architecture.html", "link_class": "vp-nav-link" }
            ]
        });
        let result = render_static_ir(&component, &data);
        assert!(result.html.contains("vp-nav-link active"));
        assert!(result.html.contains("工作流"));
    }

    #[test]
    fn static_render_escapes_interpolation() {
        let source = r#"<widget>
    <div>{msg}</div>
</widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        let component = lower_component(&root, "Msg", "msg.awsl", &LoweringOptions::default());
        let result = render_static_ir(&component, &serde_json::json!({ "msg": "<script>" }));
        assert!(!result.html.contains("<script>"));
        assert!(result.html.contains("&lt;"));
    }
}
