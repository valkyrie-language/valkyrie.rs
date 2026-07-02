//! AArch64 两遍编码与 fixup。

use std::collections::BTreeMap;

use miette::{Result, miette};

use super::{A64Instruction, RegX};

/// fixup 种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A64FixupKind {
    /// `bl` 相对 `.text` 标签。
    TextRelative {
        /// 标签名。
        label: String,
    },
    /// `adr` 相对 `.rodata` 标签。
    RodataRelative {
        /// 标签名。
        label: String,
    },
    /// `adr` 相对 `.bss` 标签。
    BssRelative {
        /// 标签名。
        label: String,
    },
}

/// 编码阶段 fixup。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A64Fixup {
    /// fixup 在 `.text` 中的偏移（imm 字段起始）。
    pub offset: u32,
    /// fixup 种类。
    pub kind: A64FixupKind,
}

/// 编码后的模块。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EncodedModule {
    /// `.text` 字节。
    pub text: Vec<u8>,
    /// fixup 列表。
    pub fixups: Vec<A64Fixup>,
    /// `.text` 内标签偏移。
    pub labels: BTreeMap<String, u32>,
}

/// AArch64 编码器。
#[derive(Debug, Default)]
pub struct A64Encoder {
    instructions: Vec<A64Instruction>,
}

impl A64Encoder {
    /// 新建编码器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加指令。
    pub fn push(&mut self, instruction: A64Instruction) {
        self.instructions.push(instruction);
    }

    /// 估算编码后 `.text` 长度（标签不占空间）。
    pub fn estimated_text_len(&self) -> u32 {
        u32::try_from(self.instructions.iter().filter(|insn| !matches!(insn, A64Instruction::Label(_))).count() * 4).unwrap_or(0)
    }

    /// 完成第一遍编码。
    pub fn finish(self) -> EncodedModule {
        let mut module = EncodedModule::default();
        for instruction in self.instructions {
            match instruction {
                A64Instruction::Label(name) => {
                    module.labels.insert(name, u32::try_from(module.text.len()).unwrap_or(0));
                }
                A64Instruction::MovRegReg { dst, src } => emit_mov_reg_reg(&mut module.text, dst, src),
                A64Instruction::MovzImm16 { dst, imm } => emit_movz(&mut module.text, dst, imm, 0),
                A64Instruction::MovkImm16Shift16 { dst, imm } => emit_movk(&mut module.text, dst, imm, 1),
                A64Instruction::MovkImm16Shift32 { dst, imm } => emit_movk(&mut module.text, dst, imm, 2),
                A64Instruction::MovkImm16Shift48 { dst, imm } => emit_movk(&mut module.text, dst, imm, 3),
                A64Instruction::LdrRegOffset { dst, base, offset } => emit_ldr_u64(&mut module.text, dst, base, offset),
                A64Instruction::StrRegOffset { src, base, offset } => emit_str_u64(&mut module.text, src, base, offset),
                A64Instruction::Adr { dst, label } => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_adr_placeholder(&mut module.text, dst);
                    let kind =
                        if label.starts_with("bss_") { A64FixupKind::BssRelative { label } } else { A64FixupKind::RodataRelative { label } };
                    module.fixups.push(A64Fixup { offset, kind });
                }
                A64Instruction::AddImm12 { dst, src, imm } => emit_add_imm12(&mut module.text, dst, src, imm),
                A64Instruction::Bl { label } => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_bl_placeholder(&mut module.text);
                    module.fixups.push(A64Fixup { offset, kind: A64FixupKind::TextRelative { label } });
                }
                A64Instruction::Blr(reg) => emit_blr(&mut module.text, reg),
                A64Instruction::Cbz { reg, label } => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_cbz_placeholder(&mut module.text, reg);
                    module.fixups.push(A64Fixup { offset, kind: A64FixupKind::TextRelative { label } });
                }
                A64Instruction::Ret => emit_ret(&mut module.text),
            }
        }
        module
    }
}

/// 应用 `.text` / `.rodata` / `.bss` fixup。
pub fn apply_fixups(
    module: &mut EncodedModule,
    rodata_labels: &BTreeMap<String, u32>,
    rodata_base: u32,
    bss_labels: &BTreeMap<String, u32>,
    bss_base: u32,
) -> Result<()> {
    let fixups = module.fixups.clone();
    for fixup in &fixups {
        match &fixup.kind {
            A64FixupKind::TextRelative { label } => {
                let target = module.labels.get(label).ok_or_else(|| miette!("缺少 `.text` 标签 `{label}`"))?;
                patch_text_relative(&mut module.text, fixup.offset, *target)?;
            }
            A64FixupKind::RodataRelative { label } => {
                let target = rodata_labels.get(label).ok_or_else(|| miette!("缺少 `.rodata` 标签 `{label}`"))?;
                patch_adr_target(&mut module.text, fixup.offset, rodata_base + target)?;
            }
            A64FixupKind::BssRelative { label } => {
                let target = bss_labels.get(label).ok_or_else(|| miette!("缺少 `.bss` 标签 `{label}`"))?;
                patch_adr_target(&mut module.text, fixup.offset, bss_base + target)?;
            }
        }
    }
    Ok(())
}

fn patch_text_relative(text: &mut [u8], insn_off: u32, target: u32) -> Result<()> {
    let word = u32::from_le_bytes(text[insn_off as usize..insn_off as usize + 4].try_into().unwrap());
    if (word & 0xFC000000) == 0x94000000 {
        // BL
        let pc = insn_off;
        let imm26 = i32::try_from((target as i64 - pc as i64) / 4).map_err(|_| miette!("bl 偏移溢出"))?;
        let patched = 0x94000000u32 | (imm26 as u32 & 0x03FF_FFFF);
        text[insn_off as usize..insn_off as usize + 4].copy_from_slice(&patched.to_le_bytes());
    }
    else if (word & 0x9F000000) == 0x10000000 {
        patch_adr_target(text, insn_off, target)?;
    }
    else if (word & 0xFF000000) == 0xB4000000 || (word & 0x7F000000) == 0x34000000 {
        // CBZ
        let pc = insn_off;
        let imm19 = i32::try_from((target as i64 - pc as i64) / 4).map_err(|_| miette!("cbz 偏移溢出"))?;
        let rd = word & 0x1F;
        let patched = 0xB4000000u32 | ((imm19 as u32 & 0x7FFFF) << 5) | rd;
        text[insn_off as usize..insn_off as usize + 4].copy_from_slice(&patched.to_le_bytes());
    }
    else {
        return Err(miette!("未知 text-relative 指令 @ {insn_off}"));
    }
    Ok(())
}

fn patch_adr_target(text: &mut [u8], insn_off: u32, target: u32) -> Result<()> {
    let pc = insn_off;
    let diff = i32::try_from(target as i64 - pc as i64).map_err(|_| miette!("adr 偏移溢出"))?;
    let word = u32::from_le_bytes(text[insn_off as usize..insn_off as usize + 4].try_into().unwrap());
    let rd = word & 0x1F;
    let immlo = (diff as u32) & 0x3;
    let immhi = ((diff as u32) >> 2) & 0x7FFFF;
    let patched = 0x10000000u32 | (immlo << 29) | (immhi << 5) | rd;
    text[insn_off as usize..insn_off as usize + 4].copy_from_slice(&patched.to_le_bytes());
    Ok(())
}

fn emit_mov_reg_reg(out: &mut Vec<u8>, dst: RegX, src: RegX) {
    let word = 0xAA0003E0u32 | (src.id() << 16) | dst.id();
    push_u32(out, word);
}

fn emit_movz(out: &mut Vec<u8>, dst: RegX, imm: u16, hw: u32) {
    let word = 0xD2800000u32 | (hw << 21) | (u32::from(imm) << 5) | dst.id();
    push_u32(out, word);
}

fn emit_movk(out: &mut Vec<u8>, dst: RegX, imm: u16, hw: u32) {
    let word = 0xF2800000u32 | (hw << 21) | (u32::from(imm) << 5) | dst.id();
    push_u32(out, word);
}

fn emit_ldr_u64(out: &mut Vec<u8>, dst: RegX, base: RegX, offset: u32) {
    let imm12 = offset / 8;
    let word = 0xF9400000u32 | (imm12 << 10) | (base.id() << 5) | dst.id();
    push_u32(out, word);
}

fn emit_str_u64(out: &mut Vec<u8>, src: RegX, base: RegX, offset: u32) {
    let imm12 = offset / 8;
    let word = 0xF9000000u32 | (imm12 << 10) | (base.id() << 5) | src.id();
    push_u32(out, word);
}

fn emit_add_imm12(out: &mut Vec<u8>, dst: RegX, src: RegX, imm: u32) {
    let word = 0x91000000u32 | ((imm & 0xFFF) << 10) | (src.id() << 5) | dst.id();
    push_u32(out, word);
}

fn emit_adr_placeholder(out: &mut Vec<u8>, dst: RegX) {
    push_u32(out, 0x10000000 | dst.id());
}

fn emit_bl_placeholder(out: &mut Vec<u8>) {
    push_u32(out, 0x94000000);
}

fn emit_blr(out: &mut Vec<u8>, reg: RegX) {
    push_u32(out, 0xD63F0000 | (reg.id() << 5));
}

fn emit_cbz_placeholder(out: &mut Vec<u8>, reg: RegX) {
    push_u32(out, 0xB4000000 | reg.id());
}

fn emit_ret(out: &mut Vec<u8>) {
    push_u32(out, 0xD65F03C0);
}

fn push_u32(out: &mut Vec<u8>, word: u32) {
    out.extend_from_slice(&word.to_le_bytes());
}
