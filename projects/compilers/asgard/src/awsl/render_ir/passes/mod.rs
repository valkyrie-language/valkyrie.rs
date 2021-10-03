//! Canonical RenderIR optimization passes.

mod drop_empty_text;
mod fold_static_attrs;
mod merge_static_text;
mod theme_registry;

use super::canonical::RenderModule;

pub use theme_registry::{ThemeRegistryOptions, theme_registry_pass};

/// A canonical RenderIR transform pass.
pub trait RenderPass {
    fn name(&self) -> &'static str;
    fn run(&self, module: &mut RenderModule);
}

/// Run default optimization passes on canonical RenderIR.
pub fn run_default_passes(module: &mut RenderModule, theme: ThemeRegistryOptions<'_>) {
    let passes: [&dyn RenderPass; 3] =
        [&merge_static_text::MergeAdjacentStaticText, &drop_empty_text::DropEmptyText, &fold_static_attrs::FoldStaticAttrs];
    for pass in passes {
        pass.run(module);
    }
    theme_registry_pass(module, theme);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awsl::render_ir::{
        canonical::{RenderAttr, RenderAttrValue, RenderElement, RenderNode, RenderRegion, RenderText, RenderTextSegment},
        canonicalize::canonicalize_surface,
        ids::{RenderNodeId, RenderRegionId},
        surface::{SurfaceAttr, SurfaceAttrValue, SurfaceIr, SurfaceNode, TemplateNodeKind},
    };

    fn sample_module() -> RenderModule {
        let surface = SurfaceIr::from([SurfaceNode::Tag {
            tag: "Box".into(),
            kind: TemplateNodeKind::Intrinsic,
            attrs: vec![
                SurfaceAttr { name: "class".into(), value: SurfaceAttrValue::Static("a".into()), is_event: false, is_prop: false },
                SurfaceAttr { name: "class".into(), value: SurfaceAttrValue::Static("b".into()), is_event: false, is_prop: false },
            ],
            children: SurfaceIr::from([
                SurfaceNode::Text {
                    parts: vec![
                        super::super::surface::SurfaceTextPart::Static("hello ".into()),
                        super::super::surface::SurfaceTextPart::Static("world".into()),
                    ],
                    span: 0..1,
                },
                SurfaceNode::Text { parts: vec![super::super::surface::SurfaceTextPart::Static("".into())], span: 1..2 },
            ]),
            span: 0..3,
        }]);
        canonicalize_surface(surface, &[])
    }

    #[test]
    fn merge_and_drop_passes_run() {
        let mut module = sample_module();
        run_default_passes(&mut module, ThemeRegistryOptions { single_theme: None, single_mode: None });
        let root = module.node(module.roots[0]);
        let RenderNode::Element(element) = root
        else {
            panic!("expected element")
        };
        assert_eq!(element.attrs.len(), 1);
        let region = module.region(element.children);
        assert_eq!(region.nodes.len(), 1);
    }
}
