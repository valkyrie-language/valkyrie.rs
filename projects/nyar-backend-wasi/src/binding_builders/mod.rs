use std::path::Path;

use miette::Result;
use nyar::{packaging::ArtifactDescriptor, BinaryTarget, HostProjectionBoundary};

mod component;
mod js_glue;

pub(crate) use component::WasiComponentBindingBuilder;
pub(crate) use js_glue::JsGlueBindingBuilder;

/// 宿主绑定生成阶段共享的输入上下文。
pub(crate) struct BindingGenerationContext<'a> {
    /// 逻辑产物名。
    pub artifact_name: &'a str,
    /// 输出目录。
    pub output_dir: &'a Path,
    /// 面向的目标。
    pub target: &'a BinaryTarget,
    /// `WASM` 模块声明的导入。
    pub imports: &'a [(String, String)],
}

/// 宿主绑定生成器的最小接口。
pub(crate) trait HostBindingBuilder {
    /// 生成宿主绑定产物。
    fn build(&self, context: &BindingGenerationContext<'_>) -> Result<Vec<ArtifactDescriptor>>;
}

/// 根据宿主边界选择对应的绑定生成器。
pub(crate) fn generate_host_binding_artifacts(
    boundary: HostProjectionBoundary,
    context: BindingGenerationContext<'_>,
) -> Result<Vec<ArtifactDescriptor>> {
    match boundary {
        HostProjectionBoundary::WasmJsGlue => JsGlueBindingBuilder.build(&context),
        HostProjectionBoundary::WasiComponent => WasiComponentBindingBuilder.build(&context),
        other => Err(miette::miette!("`WASM` 绑定生成不支持宿主边界 {:?}", other)),
    }
}
