//! Migrated from `src/binary/coff/mod.rs` (`tests`).

use super::*;

#[test]
fn writes_section_relocations() {
    let object = CoffObject {
        header: CoffHeader { machine: CoffMachine::Amd64, section_count: 1, symbol_table_offset: 0, symbol_count: 1, characteristics: 0 },
        object_kind: ObjectKind::ObjectFile,
        sections: vec![CoffSection {
            name: ".text".to_string(),
            data: vec![0x31, 0xC0, 0xC3],
            relocations: vec![CoffRelocation { offset: 0, symbol_name: "main".to_string(), kind: CoffRelocationKind::Rel32 }],
            characteristics: 0x6000_0020,
        }],
        symbols: vec![CoffSymbol { name: "main".to_string(), section_index: 1, value: 0, storage_class: 2 }],
    };
    let bytes = CoffObjectWriter::write(&object).expect("write coff");
    assert!(bytes.len() > 40);
    assert_eq!(u16::from_le_bytes([bytes[0], bytes[1]]), 0x8664);
}
