use miette::{Result, miette};

use super::layout::{ELF_EHDR_SIZE, ELF_PHDR_SIZE};

const ET_DYN: u16 = 3;
const EM_AARCH64: u16 = 183;
const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PF_R: u32 = 4;
const PF_W: u32 = 2;
const PF_X: u32 = 1;

const DT_NULL: u64 = 0;
const DT_HASH: u64 = 4;
const DT_STRTAB: u64 = 5;
const DT_SYMTAB: u64 = 6;
const DT_STRSZ: u64 = 10;
const DT_SYMENT: u64 = 11;

const STT_FUNC: u8 = 2;
const STB_GLOBAL: u8 = 1;
const STV_DEFAULT: u8 = 0;
const SHN_UNDEF: u16 = 0;

/// 动态导出符号（`.text` 内偏移 + 名称）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedObjectExport {
    /// 导出符号名（如 `asgard_invoke_export`）。
    pub name: String,
    /// 函数在 `.text` 内的字节偏移。
    pub text_offset: u32,
}

/// AArch64 ET_DYN 共享库镜像。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedElfImage {
    /// `.text` 节。
    pub text: Vec<u8>,
    /// `.rodata` 节（可选）。
    pub rodata: Vec<u8>,
    /// `.bss` 节（可选）。
    pub bss: Vec<u8>,
}

/// AArch64 ET_DYN 共享库写入器。
pub struct SharedElfWriter;

impl SharedElfWriter {
    /// 写出最小可加载 `ET_DYN` `.so`（含 `.dynsym` 导出表）。
    pub fn write_aarch64(image: &SharedElfImage, exports: &[SharedObjectExport]) -> Result<Vec<u8>> {
        if image.text.is_empty() {
            return Err(miette!("`.text` 节不能为空"));
        }
        if exports.is_empty() {
            return Err(miette!("共享库至少需要一个导出符号"));
        }

        let text_len = align_up(image.text.len() as u32, 16);
        let rodata_len = align_up(image.rodata.len() as u32, 16);
        let bss_len = align_up(image.bss.len() as u32, 16);

        let phnum: u16 = if bss_len > 0 { 3 } else { 2 };
        let headers = ELF_EHDR_SIZE + ELF_PHDR_SIZE * u32::from(phnum);
        let text_off = align_up(headers, 16);
        let rodata_off = if rodata_len == 0 { 0 } else { text_off + text_len };
        let bss_off = if bss_len == 0 {
            0
        }
        else if rodata_len == 0 {
            text_off + text_len
        }
        else {
            rodata_off + rodata_len
        };
        let hash_off = if bss_len > 0 {
            bss_off + bss_len
        }
        else if rodata_len == 0 {
            text_off + text_len
        }
        else {
            rodata_off + rodata_len
        };

        let sym_count = exports.len() + 1;
        let hash_len = (8 + sym_count * 4) as u32;
        let dynsym_off = hash_off + hash_len;
        let dynsym_len = (sym_count * 24) as u32;

        let mut dynstr = vec![0u8];
        let mut sym_name_offsets = Vec::with_capacity(exports.len());
        for export in exports {
            sym_name_offsets.push(dynstr.len() as u32);
            dynstr.extend_from_slice(export.name.as_bytes());
            dynstr.push(0);
        }
        let dynstr_len = align_up(dynstr.len() as u32, 1);
        let dynstr_off = dynsym_off + dynsym_len;

        let dynamic_count = 6;
        let dynamic_len = dynamic_count * 16;
        let dynamic_off = dynstr_off + dynstr_len;

        let file_size = dynamic_off + dynamic_len;
        let load_vaddr: u64 = 0;
        let bss_vaddr = align_up(text_len + rodata_len, 16) as u64;

        let mut out = Vec::with_capacity(file_size as usize);
        write_ehdr(&mut out, phnum, file_size, dynamic_off, load_vaddr);
        write_phdr_load(&mut out, 0, hash_off, load_vaddr, PF_R | PF_X);
        if bss_len > 0 {
            write_phdr_load(&mut out, bss_off, bss_len, bss_vaddr, PF_R | PF_W);
        }
        write_phdr_dynamic(&mut out, dynamic_off, dynamic_len, load_vaddr + u64::from(dynamic_off));

        pad_to(&mut out, text_off as usize);
        out.extend_from_slice(&image.text);
        pad_to(&mut out, (text_off + text_len) as usize);

        if rodata_len > 0 {
            out.extend_from_slice(&image.rodata);
            pad_to(&mut out, (rodata_off + rodata_len) as usize);
        }

        if bss_len > 0 {
            out.extend_from_slice(&image.bss);
            pad_to(&mut out, (bss_off + bss_len) as usize);
        }

        write_hash(&mut out, sym_count);
        write_dynsym(&mut out, exports, &sym_name_offsets, load_vaddr + u64::from(text_off));
        out.extend_from_slice(&dynstr);
        pad_to(&mut out, (dynstr_off + dynstr_len) as usize);

        write_dynamic(
            &mut out,
            load_vaddr + u64::from(hash_off),
            load_vaddr + u64::from(dynsym_off),
            load_vaddr + u64::from(dynstr_off),
            dynstr.len() as u64,
        );

        debug_assert_eq!(out.len(), file_size as usize);
        Ok(out)
    }
}

fn write_ehdr(out: &mut Vec<u8>, phnum: u16, _file_size: u32, dynamic_off: u32, _load_vaddr: u64) {
    out.extend_from_slice(&[0x7F, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    out.extend_from_slice(&ET_DYN.to_le_bytes());
    out.extend_from_slice(&EM_AARCH64.to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&(ELF_EHDR_SIZE as u64).to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(ELF_EHDR_SIZE as u16).to_le_bytes());
    out.extend_from_slice(&(ELF_PHDR_SIZE as u16).to_le_bytes());
    out.extend_from_slice(&phnum.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    let _ = dynamic_off;
}

fn write_phdr_load(out: &mut Vec<u8>, offset: u32, filesz: u32, vaddr: u64, flags: u32) {
    out.extend_from_slice(&PT_LOAD.to_le_bytes());
    out.extend_from_slice(&flags.to_le_bytes());
    out.extend_from_slice(&offset.to_le_bytes());
    out.extend_from_slice(&vaddr.to_le_bytes());
    out.extend_from_slice(&vaddr.to_le_bytes());
    out.extend_from_slice(&(filesz as u64).to_le_bytes());
    out.extend_from_slice(&(filesz as u64).to_le_bytes());
    out.extend_from_slice(&0x1000u64.to_le_bytes());
}

fn write_phdr_dynamic(out: &mut Vec<u8>, offset: u32, filesz: u32, vaddr: u64) {
    out.extend_from_slice(&PT_DYNAMIC.to_le_bytes());
    out.extend_from_slice(&(PF_R).to_le_bytes());
    out.extend_from_slice(&offset.to_le_bytes());
    out.extend_from_slice(&vaddr.to_le_bytes());
    out.extend_from_slice(&vaddr.to_le_bytes());
    out.extend_from_slice(&(filesz as u64).to_le_bytes());
    out.extend_from_slice(&(filesz as u64).to_le_bytes());
    out.extend_from_slice(&0x8u64.to_le_bytes());
}

fn write_hash(out: &mut Vec<u8>, sym_count: usize) {
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&(sym_count as u32).to_le_bytes());
    out.extend_from_slice(&1u32.to_le_bytes());
    for _ in 1..sym_count {
        out.extend_from_slice(&0u32.to_le_bytes());
    }
}

fn write_dynsym(out: &mut Vec<u8>, exports: &[SharedObjectExport], name_offsets: &[u32], text_vaddr: u64) {
    write_sym(out, 0, 0, 0, 0, 0);
    for (export, name_off) in exports.iter().zip(name_offsets) {
        let value = text_vaddr + u64::from(export.text_offset);
        let info = (STB_GLOBAL << 4) | STT_FUNC;
        write_sym(out, *name_off, info, STV_DEFAULT, 1, value);
    }
}

fn write_sym(out: &mut Vec<u8>, name: u32, info: u8, other: u8, shndx: u16, value: u64) {
    out.extend_from_slice(&name.to_le_bytes());
    out.push(info);
    out.push(other);
    out.extend_from_slice(&shndx.to_le_bytes());
    out.extend_from_slice(&value.to_le_bytes());
    out.extend_from_slice(&0u64.to_le_bytes());
}

fn write_dynamic(out: &mut Vec<u8>, hash_vaddr: u64, symtab_vaddr: u64, strtab_vaddr: u64, strsz: u64) {
    write_dyn(out, DT_HASH, hash_vaddr);
    write_dyn(out, DT_SYMTAB, symtab_vaddr);
    write_dyn(out, DT_STRTAB, strtab_vaddr);
    write_dyn(out, DT_STRSZ, strsz);
    write_dyn(out, DT_SYMENT, 24);
    write_dyn(out, DT_NULL, 0);
}

fn write_dyn(out: &mut Vec<u8>, tag: u64, value: u64) {
    out.extend_from_slice(&tag.to_le_bytes());
    out.extend_from_slice(&value.to_le_bytes());
}

fn align_up(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    let rem = value % alignment;
    if rem == 0 { value } else { value + (alignment - rem) }
}

fn pad_to(out: &mut Vec<u8>, offset: usize) {
    while out.len() < offset {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
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
}
