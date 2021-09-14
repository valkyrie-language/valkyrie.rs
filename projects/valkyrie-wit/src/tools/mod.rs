use crate::bindings::{self, DecodeConfig, EncodeConfig, Guest, PolyfillConfig, ToolsError, export};
use js_component_bindgen::{BindingsMode, InstantiationMode, TranspileOpts};
use std::{
    collections::HashMap,
};
use wasmprinter::PrintFmtWrite;
use wat::GenerateDwarf;

export!(ToolsContext with_types_in bindings);

pub struct ToolsContext {}

impl Guest for ToolsContext {
    fn wat_encode(input: String, config: EncodeConfig) -> Result<Vec<u8>, ToolsError> {
        let mut parser = wat::Parser::new();
        if config.generate_dwarf {
            parser.generate_dwarf(GenerateDwarf::Full);
        }
        Ok(parser.parse_str(None, input)?)
    }

    fn wasm_decode(input: Vec<u8>, config: DecodeConfig) -> Result<String, ToolsError> {
        let mut parser = wasmprinter::Config::new();
        parser.print_offsets(false);
        parser.print_skeleton(config.skeleton_only);
        parser.name_unnamed(config.name_unnamed);
        parser.fold_instructions(config.fold_instructions);
        let mut dst = String::new();
        parser.print(&input, &mut PrintFmtWrite(&mut dst))?;
        Ok(dst)
    }

    fn wasi_polyfill(input: Vec<u8>, config: PolyfillConfig) -> Result<Vec<(String, Vec<u8>)>, ToolsError> {
        let mut map = HashMap::default();
        map.insert("wasi:*".to_string(), "@bytecodealliance/preview2-shim/*".to_string());
        for (k, v) in config.shim {
            map.insert(k, v);
        }
        let cfg = TranspileOpts {
            name: config.name,
            no_typescript: false,
            instantiation: Some(InstantiationMode::Async),
            import_bindings: Some(BindingsMode::Js),
            map: Some(map),
            no_nodejs_compat: false,
            base64_cutoff: 0,
            tla_compat: true,
            valid_lifting_optimization: false,
            tracing: false,
            no_namespaced_exports: true,
            multi_memory: true,
        };
        let result = js_component_bindgen::transpile(&input, cfg)?;
        Ok(result.files)
    }
}
