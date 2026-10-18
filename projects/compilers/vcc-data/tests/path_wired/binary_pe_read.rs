//! Migrated from `src/binary/pe/read.rs` (`tests`).

use super::*;

use crate::binary::pe::{NativePeImage, NativePeWriter};

#[test]
fn parses_native_pe_writer_output() {
    let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
    let bytes = NativePeWriter::write_executable(&image).expect("write pe");
    let header = parse_pe64(&bytes).expect("parse pe");
    assert_eq!(header.machine, 0x8664);
    assert_eq!(header.optional_magic, 0x020B);
}

#[test]
fn rejects_mz_only_stub() {
    assert_eq!(parse_pe64(b"MZ"), Err(Pe64ParseError::TooShort));
}
