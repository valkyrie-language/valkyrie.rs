//! Migrated from `src/binary/elf/writer.rs` (`tests`).

use super::*;

use crate::binary::{
    elf::NativeElfImageBuilder,
    x86_64::{Reg64, X64Instruction},
};

#[test]
fn writes_elf_magic_and_entry() {
    let mut builder = NativeElfImageBuilder::new();
    builder.push(X64Instruction::Label("_start".to_string()));
    // exit(0)
    builder.push(X64Instruction::MovRegImm32(Reg64::Rax, 60));
    builder.push(X64Instruction::XorReg { dst: Reg64::Rdi, src: Reg64::Rdi });
    builder.push(X64Instruction::Syscall);
    let bytes = builder.build_executable("_start").expect("build");
    assert_eq!(&bytes[0..4], b"\x7fELF");
    assert_eq!(bytes[4], 2); // class 64
    assert_eq!(u16::from_le_bytes([bytes[16], bytes[17]]), 2); // ET_EXEC
    assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), 62); // EM_X86_64
}
