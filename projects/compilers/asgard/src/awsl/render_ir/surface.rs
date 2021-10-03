//! Surface RenderIR: AST-like template tree with source-faithful strings.

use std::ops::Range;

/// Surface render tree root list.
pub type SurfaceIr = Vec<SurfaceNode>;

/// Template node kind (unchanged from legacy surface lowering).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateNodeKind {
    Component,
    Intrinsic,
    HostView,
}

/// Surface render node.
#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceNode {
    Tag { tag: String, kind: TemplateNodeKind, attrs: Vec<SurfaceAttr>, children: SurfaceIr, span: Range<usize> },
    Text { parts: Vec<SurfaceTextPart>, span: Range<usize> },
    If(SurfaceIf),
    Loop(SurfaceLoop),
    Fragment { children: SurfaceIr, span: Range<usize> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceAttr {
    pub name: String,
    pub value: SurfaceAttrValue,
    pub is_event: bool,
    pub is_prop: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceAttrValue {
    Static(String),
    Dynamic(String),
    Mixed(Vec<SurfaceTextPart>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurfaceTextPart {
    Static(String),
    Dynamic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceIf {
    pub condition: String,
    pub then_branch: SurfaceIr,
    pub else_branch: SurfaceIr,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceLoop {
    pub items_expr: String,
    pub item_var: String,
    pub index_var: String,
    pub key_expr: Option<String>,
    pub body: SurfaceIr,
    pub span: Range<usize>,
}

pub fn is_fragment_root(ir: &SurfaceIr) -> bool {
    matches!(ir.as_slice(), [SurfaceNode::Fragment { .. }])
}

pub fn is_intrinsic_tag(tag: &str) -> bool {
    matches!(
        tag,
        "Column"
            | "Box"
            | "Row"
            | "Text"
            | "Button"
            | "Flex"
            | "Slot"
            | "List"
            | "Item"
            | "Checkbox"
            | "Radio"
            | "RadioGroup"
            | "column"
            | "box"
            | "row"
            | "text"
            | "button"
            | "flex"
            | "slot"
            | "list"
            | "item"
            | "checkbox"
            | "radio"
            | "radiogroup"
    )
}
