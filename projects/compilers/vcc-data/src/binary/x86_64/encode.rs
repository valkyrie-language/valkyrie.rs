use miette::{Result, miette};

use super::{ConditionCode, Reg64, X64Instruction};

/// 编码阶段待回填的 fixup。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct X64Fixup {
    /// fixup 在 `.text` 中的偏移。
    pub offset: u32,
    /// fixup 种类。
    pub kind: X64FixupKind,
}

/// fixup 种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum X64FixupKind {
    /// `rip + disp32` 指向 `.rdata` 标签。
    RipRelativeRData {
        /// 标签名。
        label: String,
    },
    /// `call [rip + disp32]` 指向导入 IAT 槽。
    ImportSlot {
        /// 槽索引。
        slot: usize,
    },
    /// `je/jmp` 等相对当前指令末尾的 `.text` 标签。
    TextRelative {
        /// 标签名。
        label: String,
    },
}

/// 编码后的模块。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EncodedModule {
    /// `.text` 节字节。
    pub text: Vec<u8>,
    /// 需要在 PE 布局完成后回填的 fixup。
    pub fixups: Vec<X64Fixup>,
    /// 标签在 `.text` 内的偏移。
    pub labels: std::collections::BTreeMap<String, u32>,
}

/// x86-64 两遍编码器。
#[derive(Debug, Default)]
pub struct X64Encoder {
    instructions: Vec<X64Instruction>,
}

impl X64Encoder {
    /// 创建编码器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加指令。
    pub fn push(&mut self, instruction: X64Instruction) {
        self.instructions.push(instruction);
    }

    /// 返回已排队指令。
    pub fn instructions(&self) -> &[X64Instruction] {
        &self.instructions
    }

    /// 完成编码（第一遍：记录标签与 fixup，不解析跨节地址）。
    pub fn finish(self) -> EncodedModule {
        let mut module = EncodedModule::default();
        for instruction in self.instructions {
            match instruction {
                X64Instruction::Label(name) => {
                    module.labels.insert(name, u32::try_from(module.text.len()).unwrap_or(0));
                }
                X64Instruction::SubRsp(imm) => emit_sub_rsp(&mut module.text, imm),
                X64Instruction::AddRsp(imm) => emit_add_rsp(&mut module.text, imm),
                X64Instruction::MovRegImm32(reg, imm) => emit_mov_reg_imm32(&mut module.text, reg, imm),
                X64Instruction::MovRegImm64(reg, imm) => emit_mov_reg_imm64(&mut module.text, reg, imm),
                X64Instruction::MovRegReg { dst, src } => emit_mov_reg_reg(&mut module.text, dst, src),
                X64Instruction::XorReg { dst, src } => emit_xor_reg(&mut module.text, dst, src),
                X64Instruction::LeaRipRelative { dst, label } => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_lea_rip_relative(&mut module.text, dst);
                    module.fixups.push(X64Fixup { offset: offset + 3, kind: X64FixupKind::RipRelativeRData { label } });
                }
                X64Instruction::CallImport { slot } => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_call_import(&mut module.text);
                    module.fixups.push(X64Fixup { offset: offset + 2, kind: X64FixupKind::ImportSlot { slot } });
                }
                X64Instruction::MovStackArgQword { value } => emit_mov_qword_rsp32(&mut module.text, value),
                X64Instruction::LeaRspOffset { dst, offset } => emit_lea_rsp_offset(&mut module.text, dst, offset),
                X64Instruction::MovRegMemReg { dst, base, offset } => {
                    emit_mov_reg_mem_reg(&mut module.text, dst, base, offset);
                }
                X64Instruction::MovRegRspOffset { dst, offset } => emit_mov_reg_rsp_offset(&mut module.text, dst, offset),
                X64Instruction::MovRspOffsetImm32 { offset, value } => emit_mov_rsp_offset_imm32(&mut module.text, offset, value),
                X64Instruction::MovMemRegImm32 { base, offset, value } => emit_mov_mem_reg_imm32(&mut module.text, base, offset, value),
                X64Instruction::MovRspOffsetReg32 { offset, src } => {
                    emit_mov_rsp_offset_reg32(&mut module.text, offset, src);
                }
                X64Instruction::AddRegReg { dst, src } => emit_add_reg_reg(&mut module.text, dst, src),
                X64Instruction::SubRegReg { dst, src } => emit_sub_reg_reg(&mut module.text, dst, src),
                X64Instruction::ImulRegReg { dst, src } => emit_imul_reg_reg(&mut module.text, dst, src),
                X64Instruction::IdivReg { divisor } => emit_idiv_reg(&mut module.text, divisor),
                X64Instruction::CmpRegReg { dst, src } => emit_cmp_reg_reg(&mut module.text, dst, src),
                X64Instruction::SetccRax { cc } => emit_setcc_rax(&mut module.text, cc),
                X64Instruction::NegReg { dst } => emit_neg_reg(&mut module.text, dst),
                X64Instruction::CmpRegImm32(reg, imm) => emit_cmp_reg_imm32(&mut module.text, reg, imm),
                X64Instruction::TestRegReg { dst, src } => emit_test_reg_reg(&mut module.text, dst, src),
                X64Instruction::Je(label) => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_jcc_rel32(&mut module.text, 0x84);
                    module.fixups.push(X64Fixup { offset: offset + 2, kind: X64FixupKind::TextRelative { label } });
                }
                X64Instruction::Jne(label) => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_jcc_rel32(&mut module.text, 0x85);
                    module.fixups.push(X64Fixup { offset: offset + 2, kind: X64FixupKind::TextRelative { label } });
                }
                X64Instruction::Jmp(label) => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_jmp_rel32(&mut module.text);
                    module.fixups.push(X64Fixup { offset: offset + 1, kind: X64FixupKind::TextRelative { label } });
                }
                X64Instruction::CallLabel(label) => {
                    let offset = u32::try_from(module.text.len()).unwrap_or(0);
                    emit_call_rel32(&mut module.text);
                    module.fixups.push(X64Fixup { offset: offset + 1, kind: X64FixupKind::TextRelative { label } });
                }
                X64Instruction::CallReg(reg) => emit_call_reg(&mut module.text, reg),
                X64Instruction::Syscall => module.text.extend_from_slice(&[0x0F, 0x05]),
                X64Instruction::Ret => module.text.push(0xC3),
            }
        }
        module
    }
}

/// 应用 fixup：给定 `.text` rva、`.rdata` rva 与 IAT rva 列表。
pub fn apply_fixups(
    module: &mut EncodedModule,
    text_rva: u32,
    rdata_offsets: &std::collections::BTreeMap<String, u32>,
    rdata_rva: u32,
    iat_rvas: &[u32],
) -> Result<()> {
    for fixup in &module.fixups {
        match &fixup.kind {
            X64FixupKind::RipRelativeRData { label } => {
                let target_offset = rdata_offsets.get(label).ok_or_else(|| miette!("缺少 `.rdata` 标签 `{label}`"))?;
                let instr_end = fixup.offset + 4;
                let target_rva = rdata_rva + target_offset;
                let rip = text_rva + instr_end;
                let disp = i32::try_from(target_rva as i64 - rip as i64).map_err(|_| miette!("`.rdata` rip-relative 溢出"))?;
                module.text[fixup.offset as usize..fixup.offset as usize + 4].copy_from_slice(&disp.to_le_bytes());
            }
            X64FixupKind::ImportSlot { slot } => {
                let iat_rva = *iat_rvas.get(*slot).ok_or_else(|| miette!("导入槽 `{slot}` 不存在"))?;
                let instr_end = fixup.offset + 4;
                let rip = text_rva + instr_end;
                let disp = i32::try_from(iat_rva as i64 - rip as i64).map_err(|_| miette!("IAT rip-relative 溢出"))?;
                module.text[fixup.offset as usize..fixup.offset as usize + 4].copy_from_slice(&disp.to_le_bytes());
            }
            X64FixupKind::TextRelative { label } => {
                let target_offset = module.labels.get(label).ok_or_else(|| miette!("缺少 `.text` 标签 `{label}`"))?;
                let instr_end = fixup.offset + 4;
                let rip = text_rva + instr_end;
                let disp = i32::try_from(*target_offset as i64 - rip as i64).map_err(|_| miette!("`.text` 相对跳转溢出"))?;
                module.text[fixup.offset as usize..fixup.offset as usize + 4].copy_from_slice(&disp.to_le_bytes());
            }
        }
    }
    Ok(())
}

fn emit_rex(prefix: &mut Vec<u8>, w: bool, r: bool, x: bool, b: bool) {
    let mut value = 0x40u8;
    if w {
        value |= 0x08;
    }
    if r {
        value |= 0x04;
    }
    if x {
        value |= 0x02;
    }
    if b {
        value |= 0x01;
    }
    if value != 0x40 {
        prefix.push(value);
    }
}

fn emit_sub_rsp(out: &mut Vec<u8>, imm: u32) {
    if imm == 0 {
        return;
    }
    if imm <= 0x7F {
        out.extend_from_slice(&[0x48, 0x83, 0xEC, imm as u8]);
    }
    else {
        out.extend_from_slice(&[0x48, 0x81, 0xEC]);
        out.extend_from_slice(&imm.to_le_bytes());
    }
}

fn emit_add_rsp(out: &mut Vec<u8>, imm: u32) {
    if imm == 0 {
        return;
    }
    if imm <= 0x7F {
        out.extend_from_slice(&[0x48, 0x83, 0xC4, imm as u8]);
    }
    else {
        out.extend_from_slice(&[0x48, 0x81, 0xC4]);
        out.extend_from_slice(&imm.to_le_bytes());
    }
}

fn emit_mov_reg_imm32(out: &mut Vec<u8>, reg: Reg64, imm: u32) {
    // `B8+rd id`：MOV r32, imm32（零扩展到 64 位）；REX.W 会变成 imm64 编码，故不置 W。
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, false, false, false, reg.is_extended());
    prefix.push(0xB8 + reg.low3());
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&imm.to_le_bytes());
}

fn emit_mov_reg_imm64(out: &mut Vec<u8>, reg: Reg64, imm: u64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, false, false, reg.is_extended());
    prefix.push(0xB8 + reg.low3());
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&imm.to_le_bytes());
}

fn emit_mov_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    // `89 /r`: MOV r/m64, r64 — ModRM.reg = src, ModRM.rm = dst
    emit_rex(&mut prefix, true, src.is_extended(), false, dst.is_extended());
    prefix.push(0x89);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

fn emit_xor_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    // `31 /r`: XOR r/m32, r32 — ModRM.reg = src, ModRM.rm = dst
    emit_rex(&mut prefix, false, src.is_extended(), false, dst.is_extended());
    prefix.push(0x31);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

fn emit_lea_rip_relative(out: &mut Vec<u8>, dst: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, dst.is_extended(), false, false);
    prefix.push(0x8D);
    prefix.push(modrm(0, dst.low3(), 5));
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&[0, 0, 0, 0]);
}

fn emit_call_import(out: &mut Vec<u8>) {
    out.extend_from_slice(&[0xFF, 0x15, 0, 0, 0, 0]);
}

fn emit_mov_qword_rsp32(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&[0x48, 0xC7, 0x44, 0x24, 0x20]);
    out.extend_from_slice(&(value as u32).to_le_bytes());
}

fn emit_lea_rsp_offset(out: &mut Vec<u8>, dst: Reg64, offset: i32) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, dst.is_extended(), false, false);
    prefix.push(0x8D);
    // mod=10 (disp32), rm=100 (SIB), reg=dst
    prefix.push(modrm(2, dst.low3(), 4));
    // SIB: scale=0, index=100 (none), base=100 (RSP)
    prefix.push(0x24);
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&offset.to_le_bytes());
}

fn modrm_reg_reg(dst: Reg64, src: Reg64) -> u8 {
    modrm(3, src.low3(), dst.low3())
}

fn modrm(mod_: u8, reg: u8, rm: u8) -> u8 {
    (mod_ << 6) | ((reg & 0x7) << 3) | (rm & 0x7)
}

fn emit_mov_reg_mem_reg(out: &mut Vec<u8>, dst: Reg64, base: Reg64, offset: i8) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, dst.is_extended(), false, base.is_extended());
    prefix.push(0x8B);
    if offset == 0 && (base.low3() & 0x7) != 5 {
        prefix.push(modrm(0, dst.low3(), base.low3()));
    }
    else {
        prefix.push(modrm(1, dst.low3(), base.low3()));
        prefix.push(offset as u8);
    }
    out.extend_from_slice(&prefix);
}

fn emit_call_reg(out: &mut Vec<u8>, reg: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, false, false, false, reg.is_extended());
    prefix.push(0xFF);
    prefix.push(modrm(3, 2, reg.low3()));
    out.extend_from_slice(&prefix);
}

fn emit_mov_reg_rsp_offset(out: &mut Vec<u8>, dst: Reg64, offset: i32) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, dst.is_extended(), false, false);
    prefix.push(0x8B);
    prefix.push(modrm(2, dst.low3(), 4));
    prefix.push(0x24);
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&offset.to_le_bytes());
}

fn emit_mov_rsp_offset_imm32(out: &mut Vec<u8>, offset: i32, value: u32) {
    out.extend_from_slice(&[0x48, 0xC7, 0x44, 0x24]);
    out.push(offset as u8);
    out.extend_from_slice(&value.to_le_bytes());
}

fn emit_mov_mem_reg_imm32(out: &mut Vec<u8>, base: Reg64, offset: i8, value: u32) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, false, false, false, base.is_extended());
    prefix.push(0xC7);
    if offset == 0 && (base.low3() & 0x7) != 5 {
        prefix.push(modrm(0, 0, base.low3()));
    }
    else {
        prefix.push(modrm(1, 0, base.low3()));
        prefix.push(offset as u8);
    }
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&value.to_le_bytes());
}

fn emit_cmp_reg_imm32(out: &mut Vec<u8>, reg: Reg64, imm: u32) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, false, false, reg.is_extended());
    if reg.low3() == 0 {
        prefix.push(0x81);
        prefix.push(modrm(3, 7, reg.low3()));
    }
    else {
        prefix.push(0x81);
        prefix.push(modrm(3, 7, reg.low3()));
    }
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&imm.to_le_bytes());
}

fn emit_test_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, false, src.is_extended(), false, dst.is_extended());
    prefix.push(0x85);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

fn emit_jmp_rel32(out: &mut Vec<u8>) {
    out.push(0xE9);
    out.extend_from_slice(&[0, 0, 0, 0]);
}

/// `call rel32`：`E8 disp32`，disp32 在 fixup 阶段从 TextRelative 计算。
fn emit_call_rel32(out: &mut Vec<u8>) {
    out.push(0xE8);
    out.extend_from_slice(&[0, 0, 0, 0]);
}

fn emit_jcc_rel32(out: &mut Vec<u8>, condition: u8) {
    out.extend_from_slice(&[0x0F, condition]);
    out.extend_from_slice(&[0, 0, 0, 0]);
}

/// `mov qword [rsp + disp32], src`（64 位写，与 64 位 `MovRegRspOffset` 加载对称）。
/// 编码：`89 /r`，mod=10 (disp32)，rm=100 (SIB)，reg=src，REX.W=1。
///
/// 此前为 32 位写（`mov dword [rsp + disp32], r32`），但加载端
/// `emit_mov_reg_rsp_offset` 始终是 64 位 `mov r64, [rsp + disp32]`，
/// 导致高 32 位为栈上残留值，破坏后续比较与控制流。统一为 64 位写。
fn emit_mov_rsp_offset_reg32(out: &mut Vec<u8>, offset: i32, src: Reg64) {
    let mut prefix = Vec::new();
    // REX.W=1：64 位写，与 64 位加载对称。
    emit_rex(&mut prefix, true, src.is_extended(), false, false);
    prefix.push(0x89);
    prefix.push(modrm(2, src.low3(), 4));
    prefix.push(0x24);
    out.extend_from_slice(&prefix);
    out.extend_from_slice(&offset.to_le_bytes());
}

/// `add dst, src`（64 位）。
/// 编码：`01 /r`（add r/m64, r64），REX.W。
fn emit_add_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, src.is_extended(), false, dst.is_extended());
    prefix.push(0x01);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

/// `sub dst, src`（64 位）。
/// 编码：`29 /r`（sub r/m64, r64），REX.W。
fn emit_sub_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, src.is_extended(), false, dst.is_extended());
    prefix.push(0x29);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

/// `imul dst, src`（64 位有符号乘法）。
/// 编码：`0F AF /r`，REX.W。
fn emit_imul_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, dst.is_extended(), false, src.is_extended());
    prefix.push(0x0F);
    prefix.push(0xAF);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

/// `cqo; idiv divisor`（64 位有符号除法）。
/// 被除数在 RDX:RAX，商在 RAX，余数在 RDX。
fn emit_idiv_reg(out: &mut Vec<u8>, divisor: Reg64) {
    // cqo: 48 99
    out.extend_from_slice(&[0x48, 0x99]);
    // idiv r/m64: REX.W F7 /7
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, false, false, divisor.is_extended());
    prefix.push(0xF7);
    prefix.push(modrm(3, 7, divisor.low3()));
    out.extend_from_slice(&prefix);
}

/// `cmp dst, src`（64 位比较）。
/// 编码：`39 /r`（cmp r/m64, r64），REX.W。
fn emit_cmp_reg_reg(out: &mut Vec<u8>, dst: Reg64, src: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, src.is_extended(), false, dst.is_extended());
    prefix.push(0x39);
    prefix.push(modrm_reg_reg(dst, src));
    out.extend_from_slice(&prefix);
}

/// `setcc al; movzx rax, al`（将比较结果转为 0/1 存入 RAX）。
/// 先发射 `setcc al`（0F 90+cc /0），再 `movzx rax, al`（48 0F B6 C0）。
fn emit_setcc_rax(out: &mut Vec<u8>, cc: ConditionCode) {
    // setcc al: 0F 90+cc /0 (mod=3, reg=0, rm=0)
    out.extend_from_slice(&[0x0F, cc.setcc_opcode(), 0xC0]);
    // movzx rax, al: 48 0F B6 C0
    out.extend_from_slice(&[0x48, 0x0F, 0xB6, 0xC0]);
}

/// `neg dst`（64 位取负）。
/// 编码：`F7 /3`，REX.W。
fn emit_neg_reg(out: &mut Vec<u8>, dst: Reg64) {
    let mut prefix = Vec::new();
    emit_rex(&mut prefix, true, false, false, dst.is_extended());
    prefix.push(0xF7);
    prefix.push(modrm(3, 3, dst.low3()));
    out.extend_from_slice(&prefix);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_return_zero_stub() {
        let mut encoder = X64Encoder::new();
        encoder.push(X64Instruction::XorReg { dst: Reg64::Rax, src: Reg64::Rax });
        encoder.push(X64Instruction::Ret);
        let module = encoder.finish();
        assert_eq!(module.text, vec![0x31, 0xC0, 0xC3]);
    }

    #[test]
    fn records_label_offsets() {
        let mut encoder = X64Encoder::new();
        encoder.push(X64Instruction::Label("entry".to_string()));
        encoder.push(X64Instruction::Ret);
        let module = encoder.finish();
        assert_eq!(module.labels.get("entry"), Some(&0));
    }
}
