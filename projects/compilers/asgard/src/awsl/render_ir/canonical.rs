//! Canonical RenderIR module and node schema.

use std::ops::Range;

use super::{
    expr::{RenderBinding, RenderDiagnostic, RenderEvent, RenderExpr},
    ids::{RenderBindingId, RenderEventId, RenderExprId, RenderNodeId, RenderRegionId},
    surface::TemplateNodeKind,
};

/// Canonical RenderIR module (side tables + rooted node forest).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderModule {
    pub roots: Vec<RenderNodeId>,
    pub nodes: Vec<RenderNode>,
    pub regions: Vec<RenderRegion>,
    pub exprs: Vec<RenderExpr>,
    pub events: Vec<RenderEvent>,
    pub bindings: Vec<RenderBinding>,
    pub diagnostics: Vec<RenderDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct RenderRegion {
    pub nodes: Vec<RenderNodeId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderNode {
    Element(RenderElement),
    Text(RenderText),
    If(RenderIfNode),
    Loop(RenderLoopNode),
    Fragment(RenderFragment),
    Component(RenderComponentNode),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderElement {
    pub tag: String,
    pub kind: TemplateNodeKind,
    pub attrs: Vec<RenderAttr>,
    pub children: RenderRegionId,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderComponentNode {
    pub tag: String,
    pub attrs: Vec<RenderAttr>,
    pub children: RenderRegionId,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderFragment {
    pub children: RenderRegionId,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderText {
    pub segments: Vec<RenderTextSegment>,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderAttr {
    pub name: String,
    pub value: RenderAttrValue,
    pub is_event: bool,
    pub is_prop: bool,
    pub event_id: Option<RenderEventId>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderAttrValue {
    Static(String),
    Expr(RenderExprId),
    Template(Vec<RenderTextSegment>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum RenderTextSegment {
    Static(String),
    Expr(RenderExprId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderIfNode {
    pub condition: RenderExprId,
    pub then_region: RenderRegionId,
    pub else_region: RenderRegionId,
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderLoopNode {
    pub items: RenderExprId,
    pub key: Option<RenderExprId>,
    pub item_binding: RenderBindingId,
    pub index_binding: Option<RenderBindingId>,
    pub item_var: String,
    pub index_var: String,
    pub body_region: RenderRegionId,
    pub span: Range<usize>,
}

/// Canonical RenderIR alias (public contract name).
pub type RenderIr = RenderModule;

impl RenderModule {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn node(&self, id: RenderNodeId) -> &RenderNode {
        &self.nodes[id.0 as usize]
    }

    pub fn region(&self, id: RenderRegionId) -> &RenderRegion {
        if id == RenderRegionId::EMPTY {
            return &EMPTY_REGION;
        }
        &self.regions[id.0 as usize]
    }

    pub fn expr(&self, id: RenderExprId) -> &RenderExpr {
        &self.exprs[id.0 as usize]
    }

    pub fn expr_source(&self, id: RenderExprId) -> &str {
        &self.expr(id).source
    }

    pub fn binding(&self, id: RenderBindingId) -> &RenderBinding {
        &self.bindings[id.0 as usize]
    }
}

static EMPTY_REGION: RenderRegion = RenderRegion { nodes: Vec::new() };

pub fn is_fragment_root(module: &RenderModule) -> bool {
    module.roots.len() == 1 && matches!(module.node(module.roots[0]), RenderNode::Fragment(_))
}

pub fn is_intrinsic_tag(tag: &str) -> bool {
    super::surface::is_intrinsic_tag(tag)
}
