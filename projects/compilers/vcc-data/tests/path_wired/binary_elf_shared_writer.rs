//! Migrated from `src/binary/elf/shared_writer.rs` (`tests`).

use super::*;

use crate::binary::{aarch64::ret_bytes, elf::parse_elf64};

#[test]
fn writes_et_dyn_aarch64_with_export() {
    let text = ret_bytes().to_vec();
    let image = SharedElfImage { text, rodata: Vec::new(), bss: Vec::new() };
    let exports = vec![
        SharedObjectExport { name: "asgard_invoke_export".into(), text_offset: 0 },
        SharedObjectExport { name: "awsl_call_on_tap".into(), text_offset: 0 },
    ];
    let bytes = SharedElfWriter::write_aarch64(&image, &exports).expect("write");
    let header = parse_elf64(&bytes).expect("parse");
    assert_eq!(header.file_type, ET_DYN);
    assert_eq!(header.machine, EM_AARCH64);
    let haystack = String::from_utf8_lossy(&bytes);
    assert!(haystack.contains("asgard_invoke_export"));
    assert!(haystack.contains("awsl_call_on_tap"));
}
