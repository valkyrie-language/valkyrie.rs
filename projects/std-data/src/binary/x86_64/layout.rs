use crate::binary::coff::{CoffMachine, CoffObject, CoffRelocation, CoffRelocationKind, CoffSection, CoffSymbol, coff_object_from_sections};

use super::{EncodedModule, X64FixupKind};

/// 将 x86-64 编码结果编排为 `COFF` 对象布局。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectLayout {
    /// `.text` 节。
    pub text: CoffSection,
    /// 符号表。
    pub symbols: Vec<CoffSymbol>,
}

impl ObjectLayout {
    /// 从编码模块与入口符号构建 `COFF` 布局。
    pub fn from_encoded_module(encoded: &EncodedModule, entry_symbol: &str, import_symbols: &[String]) -> Self {
        let mut relocations = Vec::new();
        for fixup in &encoded.fixups {
            match &fixup.kind {
                X64FixupKind::ImportSlot { slot } => {
                    if let Some(symbol) = import_symbols.get(*slot) {
                        relocations.push(CoffRelocation {
                            offset: fixup.offset,
                            symbol_name: format!("__imp_{symbol}"),
                            kind: CoffRelocationKind::Rel32,
                        });
                    }
                }
                X64FixupKind::RipRelativeRData { label } => {
                    relocations.push(CoffRelocation { offset: fixup.offset, symbol_name: label.clone(), kind: CoffRelocationKind::Rel32 });
                }
                X64FixupKind::TextRelative { .. } => {}
            }
        }

        let entry_value = encoded.labels.get(entry_symbol).copied().unwrap_or(0);
        Self {
            text: CoffSection { name: ".text".to_string(), data: encoded.text.clone(), relocations, characteristics: 0x6000_0020 },
            symbols: vec![CoffSymbol { name: entry_symbol.to_string(), section_index: 1, value: entry_value, storage_class: 2 }],
        }
    }

    /// 转为 `COFF` 对象。
    pub fn into_coff_object(self) -> CoffObject {
        coff_object_from_sections(CoffMachine::Amd64, vec![self.text], self.symbols)
    }
}
