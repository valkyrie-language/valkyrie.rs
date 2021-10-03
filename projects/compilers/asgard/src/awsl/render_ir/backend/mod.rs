//! Shared canonical RenderIR query helpers for backend emitters.

use super::{
    canonical::{RenderAttrValue, RenderModule, RenderNode, RenderTextSegment},
    ids::{RenderNodeId, RenderRegionId},
    surface::TemplateNodeKind,
};

pub fn region_nodes<'a>(module: &'a RenderModule, region: RenderRegionId) -> &'a [RenderNodeId] {
    &module.region(region).nodes
}

pub fn node_children_region(node: &RenderNode) -> Option<RenderRegionId> {
    match node {
        RenderNode::Element(element) => Some(element.children),
        RenderNode::Component(component) => Some(component.children),
        RenderNode::Fragment(fragment) => Some(fragment.children),
        RenderNode::If(_) | RenderNode::Loop(_) | RenderNode::Text(_) => None,
    }
}

pub fn node_kind(node: &RenderNode) -> Option<TemplateNodeKind> {
    match node {
        RenderNode::Element(element) => Some(element.kind),
        RenderNode::Component(_) => Some(TemplateNodeKind::Component),
        _ => None,
    }
}

pub fn attr_value_source(module: &RenderModule, value: &RenderAttrValue) -> String {
    match value {
        RenderAttrValue::Static(text) => text.clone(),
        RenderAttrValue::Expr(expr_id) => module.expr_source(*expr_id).to_string(),
        RenderAttrValue::Template(segments) => segments
            .iter()
            .map(|segment| match segment {
                RenderTextSegment::Static(text) => text.clone(),
                RenderTextSegment::Expr(expr_id) => module.expr_source(*expr_id).to_string(),
            })
            .collect::<Vec<_>>()
            .join(""),
    }
}

pub fn text_segments_source(module: &RenderModule, segments: &[RenderTextSegment]) -> String {
    segments
        .iter()
        .map(|segment| match segment {
            RenderTextSegment::Static(text) => text.clone(),
            RenderTextSegment::Expr(expr_id) => module.expr_source(*expr_id).to_string(),
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn walk_region<F>(module: &RenderModule, region: RenderRegionId, mut visit: F)
where
    F: FnMut(&RenderModule, RenderNodeId, &RenderNode),
{
    for node_id in region_nodes(module, region).to_vec() {
        let node = module.node(node_id).clone();
        visit(module, node_id, &node);
        if let Some(child_region) = node_children_region(&node) {
            if child_region.is_valid() {
                walk_region(module, child_region, &mut visit);
            }
        }
        if let RenderNode::If(render_if) = &node {
            if render_if.then_region.is_valid() {
                walk_region(module, render_if.then_region, &mut visit);
            }
            if render_if.else_region.is_valid() {
                walk_region(module, render_if.else_region, &mut visit);
            }
        }
        if let RenderNode::Loop(render_loop) = &node {
            if render_loop.body_region.is_valid() {
                walk_region(module, render_loop.body_region, &mut visit);
            }
        }
    }
}
