//! 各平台最小宿主可执行封装（PE / ELF / Mach-O），用于嵌入 V 原生逻辑与 Asgard UI wire 尾段。

use miette::Result;

use crate::binary::{elf::NativeElfImageBuilder, mach_o::MachOImageBuilder, pe::NativeImageBuilder, x86_64::X64Instruction};

/// 按 OS 平台写出宿主可执行字节。
pub fn build_host_executable(platform: &str, native_logic: Option<&[u8]>, rodata_tail: &[u8]) -> Result<Vec<u8>> {
    match platform {
        "windows" => build_pe_executable(native_logic, rodata_tail),
        "linux" => build_elf_executable(native_logic, rodata_tail),
        _ => build_macho_executable(native_logic, rodata_tail),
    }
}

fn build_pe_executable(native_logic: Option<&[u8]>, rodata_tail: &[u8]) -> Result<Vec<u8>> {
    let mut builder = NativeImageBuilder::new();
    if let Some(code) = native_logic {
        builder.add_rdata("host_logic", code);
    }
    if !rodata_tail.is_empty() {
        builder.add_rdata("asgard_rodata", rodata_tail);
    }
    builder.push(X64Instruction::Label("_start".into()));
    builder.push(X64Instruction::Ret);
    builder.build_executable("_start")
}

fn build_elf_executable(native_logic: Option<&[u8]>, rodata_tail: &[u8]) -> Result<Vec<u8>> {
    let mut builder = NativeElfImageBuilder::new();
    if let Some(code) = native_logic {
        builder.add_rodata("host_logic", code);
    }
    if !rodata_tail.is_empty() {
        builder.add_rodata("asgard_rodata", rodata_tail);
    }
    builder.push(X64Instruction::Label("_start".into()));
    builder.push(X64Instruction::Ret);
    builder.build_executable("_start")
}

fn build_macho_executable(native_logic: Option<&[u8]>, rodata_tail: &[u8]) -> Result<Vec<u8>> {
    let mut builder = MachOImageBuilder::new();
    if let Some(code) = native_logic {
        builder.set_code(code);
    }
    let mut rodata = Vec::new();
    rodata.extend_from_slice(rodata_tail);
    if !rodata.is_empty() {
        builder.add_rodata(&rodata);
    }
    builder.build_executable()
}
