use std::{collections::BTreeMap, fs};

use miette::{IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::ArtifactFormat,
    packaging::{ArtifactDescriptor, TargetLane},
};

use crate::WitPackage;

use super::{BindingGenerationContext, HostBindingBuilder};

/// `WASI component model` 宿主绑定生成器。
pub(crate) struct WasiComponentBindingBuilder;

impl HostBindingBuilder for WasiComponentBindingBuilder {
    fn build(&self, context: &BindingGenerationContext<'_>) -> Result<Vec<ArtifactDescriptor>> {
        let wit_path = context.output_dir.join(format!("{}.component.wit", context.artifact_name));
        let package = build_component_wit_package(context.artifact_name, context.imports);
        fs::write(&wit_path, package.to_text())
            .into_diagnostic()
            .wrap_err_with(|| format!("写入 `WASI component` `WIT` 描述失败：{}", wit_path.display()))?;

        Ok(vec![ArtifactDescriptor {
            name: format!("{}.component", context.artifact_name),
            kind: nyar::ArtifactKind::AssemblyListing,
            format: ArtifactFormat::RawBinary,
            target: context.target.clone(),
            lane: TargetLane::Wasm,
        }])
    }
}

fn build_component_wit_package(artifact_name: &str, imports: &[(String, String)]) -> WitPackage {
    let mut grouped_imports: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (module, field) in imports {
        grouped_imports.entry(sanitize_wit_identifier(module)).or_default().push(format!("{}: func()", sanitize_wit_identifier(field)));
    }

    let mut package = WitPackage::new(format!("nyar:{}@0.1.0", sanitize_wit_identifier(artifact_name)));
    if grouped_imports.is_empty() {
        package.push_interface("entry", vec!["run: func()".to_string()]);
        return package;
    }

    for (interface_name, functions) in grouped_imports {
        package.push_interface(interface_name, functions);
    }
    package
}

fn sanitize_wit_identifier(raw: &str) -> String {
    let mut sanitized = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == ':' || ch == '@' || ch == '.' {
            sanitized.push(ch);
        }
        else {
            sanitized.push('_');
        }
    }

    if sanitized.is_empty() {
        "generated".to_string()
    }
    else {
        sanitized
    }
}
