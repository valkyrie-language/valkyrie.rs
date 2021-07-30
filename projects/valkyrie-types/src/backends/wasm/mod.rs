use crate::{backends::ConvertTo, ModuleItem, ModuleResolver};
use nyar_wasm::{WasmBuilder, WasmSymbol, WasmType};
use valkyrie_ast::ExpressionKind;

mod structures;

impl ModuleResolver {
    pub fn build_wasm(&self) -> WasmBuilder {
        let mut builder = WasmBuilder::default();

        for item in self.items.values() {
            match item {
                ModuleItem::Imported(_) => {}
                ModuleItem::Structure(v) => builder.register(v.convert()),
                ModuleItem::External(v) => builder.register(v.convert()),
                ModuleItem::Function(v) => builder.register(v.convert()),
            }
        }
        builder
    }
}

impl ConvertTo<WasmType> for ExpressionKind {
    fn convert(&self) -> WasmType {
        match self {
            ExpressionKind::Symbol(s) => {
                let name = s.to_string();
                match name.as_str() {
                    "bool" => WasmType::Bool,
                    "u8" => WasmType::U8,
                    "u16" => WasmType::U16,
                    "u32" => WasmType::U32,
                    "u64" => WasmType::U64,
                    "usize" => WasmType::U32,
                    "i8" => WasmType::I8,
                    "i16" => WasmType::I16,
                    "i32" => WasmType::I32,
                    "i64" => WasmType::I64,
                    "isize" => WasmType::I32,
                    "f32" => WasmType::F32,
                    "f64" => WasmType::F64,
                    "char" => WasmType::Unicode,
                    _ => WasmType::Reference { name: WasmSymbol::from(name), nullable: false },
                }
            }
            _ => unimplemented!("Unknown type item {self:?}"),
        }
    }
}
