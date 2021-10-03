use miette::{Result, miette};

use super::layout::{ELF_EHDR_SIZE, ELF_LOAD_BASE, ELF_PHDR_SIZE, ElfSectionLayout};

/// 待写入的原生 ELF 镜像。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeElfImage {
    /// `.text` 节。
    pub text: Vec<u8>,
    /// `.rodata` 节。
    pub rodata: Vec<u8>,
    /// 入口点在 `.text` 内的偏移。
    pub entry_point: u32,
}

/// 原生 ELF 可执行文件写入器。
pub struct NativeElfWriter;

impl NativeElfWriter {
    /// 将镜像写入静态 `ELF64` `ET_EXEC` 可执行文件。
    pub fn write_executable(image: &NativeElfImage) -> Result<Vec<u8>> {
        if image.text.is_empty() {
            return Err(miette!("`.text` 节不能为空"));
        }

        let text_len = u32::try_from(image.text.len()).map_err(|_| miette!("`.text` 过大"))?;
        let rodata_len = u32::try_from(image.rodata.len()).map_err(|_| miette!("`.rodata` 过大"))?;
        let layout = ElfSectionLayout::compute(text_len, rodata_len);
        let entry = u64::from(layout.text_vaddr + image.entry_point);

        let mut out = Vec::with_capacity(layout.file_size as usize);
        write_ehdr(&mut out, entry, layout.file_size);
        write_phdr(&mut out, layout.file_size);
        pad_to(&mut out, layout.text_offset as usize);
        out.extend_from_slice(&image.text);
        if !image.rodata.is_empty() {
            pad_to(&mut out, layout.rodata_offset as usize);
            out.extend_from_slice(&image.rodata);
        }
        Ok(out)
    }
}

fn write_ehdr(out: &mut Vec<u8>, entry: u64, _file_size: u32) {
    // e_ident
    out.extend_from_slice(&[
        0x7F, b'E', b'L', b'F', // magic
        2,    // ELFCLASS64
        1,    // ELFDATA2LSB
        1,    // EV_CURRENT
        0,    // ELFOSABI_NONE
        0, 0, 0, 0, 0, 0, 0, 0, // padding
    ]);
    out.extend_from_slice(&2u16.to_le_bytes()); // ET_EXEC
    out.extend_from_slice(&62u16.to_le_bytes()); // EM_X86_64
    out.extend_from_slice(&1u32.to_le_bytes()); // e_version
    out.extend_from_slice(&entry.to_le_bytes());
    out.extend_from_slice(&(ELF_EHDR_SIZE as u64).to_le_bytes()); // e_phoff
    out.extend_from_slice(&0u64.to_le_bytes()); // e_shoff
    out.extend_from_slice(&0u32.to_le_bytes()); // e_flags
    out.extend_from_slice(&(ELF_EHDR_SIZE as u16).to_le_bytes());
    out.extend_from_slice(&(ELF_PHDR_SIZE as u16).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // e_phnum
    out.extend_from_slice(&0u16.to_le_bytes()); // e_shentsize
    out.extend_from_slice(&0u16.to_le_bytes()); // e_shnum
    out.extend_from_slice(&0u16.to_le_bytes()); // e_shstrndx
}

fn write_phdr(out: &mut Vec<u8>, file_size: u32) {
    // PT_LOAD covering the whole file (identity map at ELF_LOAD_BASE)
    out.extend_from_slice(&1u32.to_le_bytes()); // p_type = PT_LOAD
    out.extend_from_slice(&5u32.to_le_bytes()); // PF_R | PF_X
    out.extend_from_slice(&0u64.to_le_bytes()); // p_offset
    out.extend_from_slice(&ELF_LOAD_BASE.to_le_bytes()); // p_vaddr
    out.extend_from_slice(&ELF_LOAD_BASE.to_le_bytes()); // p_paddr
    let size = file_size as u64;
    out.extend_from_slice(&size.to_le_bytes()); // p_filesz
    out.extend_from_slice(&size.to_le_bytes()); // p_memsz
    out.extend_from_slice(&0x1000u64.to_le_bytes()); // p_align
}

fn pad_to(out: &mut Vec<u8>, size: usize) {
    while out.len() < size {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
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
}
