//! Migrated from `src/binary/pe/native/mod.rs` (`tests`).

use super::*;

#[test]
fn writes_amd64_console_executable() {
    let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
    let bytes = NativePeWriter::write_executable(&image).expect("write pe");
    assert_eq!(&bytes[0..2], b"MZ");
    assert_eq!(&bytes[0x40..0x44], b"PE\0\0");
    assert_eq!(u16::from_le_bytes([bytes[0x44], bytes[0x45]]), 0x8664);

    let pe_offset = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
    let magic = u16::from_le_bytes(bytes[pe_offset + 24..pe_offset + 26].try_into().unwrap());
    assert_eq!(magic, 0x020B);
    let section_count = u16::from_le_bytes(bytes[pe_offset + 6..pe_offset + 8].try_into().unwrap());
    assert_eq!(section_count, 1);
    let import_rva = u32::from_le_bytes(bytes[pe_offset + 24 + 120..pe_offset + 24 + 124].try_into().unwrap());
    let import_size = u32::from_le_bytes(bytes[pe_offset + 24 + 124..pe_offset + 24 + 128].try_into().unwrap());
    assert_eq!(import_rva, 0);
    assert_eq!(import_size, 0);
}

#[test]
#[cfg(windows)]
fn runs_stub_executable() {
    let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
    let bytes = NativePeWriter::write_executable(&image).expect("write pe");
    let path = std::env::temp_dir().join(format!("valkyrie-native-stub-{}.exe", std::process::id()));
    std::fs::write(&path, &bytes).expect("write");
    let status = std::process::Command::new(&path).status().expect("run");
    let _ = std::fs::remove_file(&path);
    assert!(status.success(), "status={status:?}");
}
