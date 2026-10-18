//! Migrated from `src/binary/elf/read.rs` (`tests`).

use super::*;

use crate::binary::elf::{NativeElfImage, NativeElfWriter};

#[test]
fn parses_native_elf_writer_output() {
    let image = NativeElfImage { text: vec![0x31, 0xC0, 0xC3], rodata: Vec::new(), entry_point: 0 };
    let bytes = NativeElfWriter::write_executable(&image).expect("write elf");
    let header = parse_elf64(&bytes).expect("parse elf");
    assert_eq!(header.file_type, 2);
    assert_eq!(header.machine, 62);
}
