//! AArch64 最小指令常量与 JNI 胶水（Android ET_DYN `.so`）。

mod abi_aapcs64;
mod encode;
mod instruction;
mod jni_glue;
mod reg;

pub use abi_aapcs64::Aapcs64FunctionBuilder;
pub use encode::{A64Encoder, A64Fixup, A64FixupKind, EncodedModule, apply_fixups};
pub use instruction::A64Instruction;
pub use jni_glue::{JniGlueModule, emit_jni_glue_module, merge_jni_and_logic};
pub use reg::RegX;

/// `ret` — A64 return。
pub const RET: u32 = 0xD65F_03C0;

/// 将 `ret` 编码为小端字节。
pub fn ret_bytes() -> [u8; 4] {
    RET.to_le_bytes()
}

/// 将若干 `u32` 指令编码为小端字节流。
pub fn encode_instructions(words: &[u32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(words.len() * 4);
    for word in words {
        out.extend_from_slice(&word.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ret_opcode_is_four_bytes() {
        assert_eq!(ret_bytes(), [0xC0, 0x03, 0x5F, 0xD6]);
    }
}
