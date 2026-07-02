//! Android JNI 胶水机器码。

use std::collections::BTreeMap;

use miette::{Result, miette};

use super::{A64Encoder, A64Instruction, EncodedModule, RegX, apply_fixups, ret_bytes};
use crate::binary::elf::{SharedElfImage, SharedObjectExport};

const VM_GET_ENV: u32 = 48;
const ENV_FIND_CLASS: u32 = 48;
const ENV_GET_STATIC_METHOD_ID: u32 = 904;
const ENV_CALL_STATIC_VOID_METHOD: u32 = 912;
const ENV_NEW_STRING_UTF: u32 = 1336;
const ENV_GET_STRING_UTF_CHARS: u32 = 1352;
const ENV_RELEASE_STRING_UTF_CHARS: u32 = 1360;
const ENV_REGISTER_NATIVES: u32 = 1720;
const JNI_VERSION_1_6: u32 = 0x0001_0006;
const JNI_ERR: u32 = 0xFFFF_FFFF;

/// JNI 胶水段。
#[derive(Debug, Clone)]
pub struct JniGlueModule {
    /// JNI `.text` 与 fixup 元数据。
    pub module: EncodedModule,
    /// `.rodata`。
    pub rodata: Vec<u8>,
    /// `.rodata` 标签。
    pub rodata_labels: BTreeMap<String, u32>,
    /// `.bss` 标签。
    pub bss_labels: BTreeMap<String, u32>,
    /// `.bss` 大小。
    pub bss_size: u32,
    /// `JNINativeMethod` 在 rodata 中的偏移。
    pub jni_methods_off: u32,
    /// `bl asgard_invoke_export` 指令偏移。
    pub invoke_bl_off: u32,
}

/// 发射 JNI 胶水。
pub fn emit_jni_glue_module() -> Result<JniGlueModule> {
    let mut ro = Rodata::new();
    let class = ro.str("class_name", b"com/asgard/runtime/AsgardHostBridge\0");
    let invoke_name = ro.str("invoke_name", b"invokeExport\0");
    let invoke_sig = ro.str("invoke_sig", b"(Ljava/lang/String;)V\0");
    let patch_name = ro.str("patch_name", b"patchFromNative\0");
    let patch_sig = ro.str("patch_sig", b"(Ljava/lang/String;Ljava/lang/String;)V\0");
    let empty = ro.str("empty", b"\0");
    let jni_methods_off = ro.jni_methods("jni_methods", &invoke_name, &invoke_sig);

    let mut enc = A64Encoder::new();

    enc.push(A64Instruction::Label("jni_invoke".into()));
    enc.push(A64Instruction::MovRegReg { dst: RegX::X19, src: RegX::X0 });
    emit_env_call(&mut enc, ENV_GET_STRING_UTF_CHARS);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X20, src: RegX::X0 });
    enc.push(A64Instruction::Cbz { reg: RegX::X20, label: "invoke_ret".into() });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X20 });
    let invoke_bl_off = enc.estimated_text_len();
    enc.push(A64Instruction::Bl { label: "asgard_invoke_export".into() });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X19 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X1, src: RegX::X2 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X2, src: RegX::X20 });
    emit_env_call(&mut enc, ENV_RELEASE_STRING_UTF_CHARS);
    enc.push(A64Instruction::Label("invoke_ret".into()));
    enc.push(A64Instruction::Ret);

    enc.push(A64Instruction::Label("asgard_patch_native".into()));
    enc.push(A64Instruction::MovRegReg { dst: RegX::X19, src: RegX::X0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X20, src: RegX::X1 });
    enc.push(A64Instruction::Adr { dst: RegX::X8, label: "bss_g_vm".into() });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X21, base: RegX::X8, offset: 0 });
    enc.push(A64Instruction::Cbz { reg: RegX::X21, label: "patch_ret".into() });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X21 });
    enc.push(A64Instruction::Adr { dst: RegX::X1, label: "bss_env".into() });
    mov_u32(&mut enc, RegX::X2, JNI_VERSION_1_6);
    emit_vm_call(&mut enc, VM_GET_ENV);
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: "bss_env".into() });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X0, base: RegX::X0, offset: 0 });
    enc.push(A64Instruction::Cbz { reg: RegX::X0, label: "patch_ret".into() });
    enc.push(A64Instruction::Adr { dst: RegX::X1, label: class.clone() });
    emit_env_call(&mut enc, ENV_FIND_CLASS);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X22, src: RegX::X0 });
    enc.push(A64Instruction::Adr { dst: RegX::X1, label: patch_name.clone() });
    enc.push(A64Instruction::Adr { dst: RegX::X2, label: patch_sig.clone() });
    emit_env_call(&mut enc, ENV_GET_STATIC_METHOD_ID);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X23, src: RegX::X0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X19 });
    enc.push(A64Instruction::Cbz { reg: RegX::X0, label: "use_empty_key".into() });
    enc.push(A64Instruction::Bl { label: "have_key".into() });
    enc.push(A64Instruction::Label("use_empty_key".into()));
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: empty.clone() });
    enc.push(A64Instruction::Label("have_key".into()));
    emit_env_call(&mut enc, ENV_NEW_STRING_UTF);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X24, src: RegX::X0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X20 });
    enc.push(A64Instruction::Cbz { reg: RegX::X0, label: "use_empty_val".into() });
    enc.push(A64Instruction::Bl { label: "have_val".into() });
    enc.push(A64Instruction::Label("use_empty_val".into()));
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: empty.clone() });
    enc.push(A64Instruction::Label("have_val".into()));
    emit_env_call(&mut enc, ENV_NEW_STRING_UTF);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X25, src: RegX::X0 });
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: "bss_env".into() });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X0, base: RegX::X0, offset: 0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X1, src: RegX::X22 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X2, src: RegX::X23 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X3, src: RegX::X24 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X4, src: RegX::X25 });
    emit_env_call(&mut enc, ENV_CALL_STATIC_VOID_METHOD);
    enc.push(A64Instruction::Label("patch_ret".into()));
    enc.push(A64Instruction::Ret);

    enc.push(A64Instruction::Label("JNI_OnLoad".into()));
    enc.push(A64Instruction::Adr { dst: RegX::X8, label: "bss_g_vm".into() });
    enc.push(A64Instruction::StrRegOffset { src: RegX::X0, base: RegX::X8, offset: 0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X19, src: RegX::X0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X0, src: RegX::X19 });
    enc.push(A64Instruction::Adr { dst: RegX::X1, label: "bss_env".into() });
    mov_u32(&mut enc, RegX::X2, JNI_VERSION_1_6);
    emit_vm_call(&mut enc, VM_GET_ENV);
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: "bss_env".into() });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X0, base: RegX::X0, offset: 0 });
    enc.push(A64Instruction::Cbz { reg: RegX::X0, label: "onload_err".into() });
    enc.push(A64Instruction::Adr { dst: RegX::X1, label: class.clone() });
    emit_env_call(&mut enc, ENV_FIND_CLASS);
    enc.push(A64Instruction::MovRegReg { dst: RegX::X20, src: RegX::X0 });
    enc.push(A64Instruction::Cbz { reg: RegX::X20, label: "onload_err".into() });
    enc.push(A64Instruction::Adr { dst: RegX::X0, label: "bss_env".into() });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X0, base: RegX::X0, offset: 0 });
    enc.push(A64Instruction::MovRegReg { dst: RegX::X1, src: RegX::X20 });
    mov_u32(&mut enc, RegX::X2, 1);
    enc.push(A64Instruction::Adr { dst: RegX::X3, label: "jni_methods".into() });
    emit_env_call(&mut enc, ENV_REGISTER_NATIVES);
    mov_u32(&mut enc, RegX::X0, JNI_VERSION_1_6);
    enc.push(A64Instruction::Ret);
    enc.push(A64Instruction::Label("onload_err".into()));
    mov_u32(&mut enc, RegX::X0, JNI_ERR);
    enc.push(A64Instruction::Ret);

    let module = enc.finish();
    let mut rodata_labels = ro.labels;
    rodata_labels.insert("jni_methods".into(), jni_methods_off);

    let mut bss_labels = BTreeMap::new();
    bss_labels.insert("bss_g_vm".into(), 0);
    bss_labels.insert("bss_env".into(), 8);

    Ok(JniGlueModule { module, rodata: ro.bytes, rodata_labels, bss_labels, bss_size: 16, jni_methods_off, invoke_bl_off })
}

/// 合并 JNI 与 V 逻辑段。
pub fn merge_jni_and_logic(
    mut jni: JniGlueModule,
    logic_text: Vec<u8>,
    logic_exports: Vec<SharedObjectExport>,
) -> Result<(SharedElfImage, Vec<SharedObjectExport>)> {
    let logic_start = align16(jni.module.text.len() as u32) as usize;
    while jni.module.text.len() < logic_start {
        jni.module.text.push(0);
    }
    let invoke_target =
        logic_start as u32 + logic_exports.iter().find(|e| e.name == "asgard_invoke_export").map(|e| e.text_offset).unwrap_or(0);
    patch_bl(&mut jni.module.text, jni.invoke_bl_off, invoke_target)?;
    jni.module.labels.insert("asgard_invoke_export".into(), invoke_target);
    jni.module.text.extend_from_slice(&logic_text);

    let rodata_base = align16(jni.module.text.len() as u32);
    let bss_base = align16(rodata_base + align16(jni.rodata.len() as u32));
    apply_fixups(&mut jni.module, &jni.rodata_labels, rodata_base, &jni.bss_labels, bss_base)?;

    let invoke_fn = *jni.module.labels.get("jni_invoke").unwrap_or(&0);
    let mut rodata = jni.rodata;
    let ptr_off = jni.jni_methods_off as usize + 16;
    if rodata.len() >= ptr_off + 8 {
        rodata[ptr_off..ptr_off + 8].copy_from_slice(&u64::from(invoke_fn).to_le_bytes());
    }
    patch_u32_in_rodata(&mut rodata, jni.jni_methods_off as usize, rodata_base + jni.rodata_labels["invoke_name"]);
    patch_u32_in_rodata(&mut rodata, jni.jni_methods_off as usize + 4, rodata_base + jni.rodata_labels["invoke_sig"]);

    let mut exports = vec![
        SharedObjectExport { name: "JNI_OnLoad".into(), text_offset: *jni.module.labels.get("JNI_OnLoad").unwrap_or(&0) },
        SharedObjectExport { name: "Java_com_asgard_runtime_AsgardHostBridge_invokeExport".into(), text_offset: invoke_fn },
        SharedObjectExport { name: "asgard_patch_native".into(), text_offset: *jni.module.labels.get("asgard_patch_native").unwrap_or(&0) },
    ];
    for export in logic_exports {
        exports.push(SharedObjectExport { name: export.name, text_offset: logic_start as u32 + export.text_offset });
    }

    let bss = vec![0u8; jni.bss_size as usize];
    Ok((SharedElfImage { text: jni.module.text, rodata, bss }, exports))
}

struct Rodata {
    bytes: Vec<u8>,
    labels: BTreeMap<String, u32>,
}

impl Rodata {
    fn new() -> Self {
        Self { bytes: Vec::new(), labels: BTreeMap::new() }
    }

    fn str(&mut self, name: &str, value: &[u8]) -> String {
        let off = self.bytes.len() as u32;
        self.labels.insert(name.to_string(), off);
        self.bytes.extend_from_slice(value);
        name.to_string()
    }

    fn jni_methods(&mut self, name: &str, method: &str, sig: &str) -> u32 {
        align8(&mut self.bytes);
        let off = self.bytes.len() as u32;
        self.labels.insert(name.to_string(), off);
        self.bytes.extend_from_slice(&u64::from(self.labels[method]).to_le_bytes());
        self.bytes.extend_from_slice(&u64::from(self.labels[sig]).to_le_bytes());
        self.bytes.extend_from_slice(&0u64.to_le_bytes());
        off
    }
}

fn patch_u32_in_rodata(rodata: &mut [u8], offset: usize, value: u32) {
    if rodata.len() >= offset + 4 {
        rodata[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
}

fn patch_u64_in_rodata(rodata: &mut [u8], offset: usize, value: u64) {
    if rodata.len() >= offset + 8 {
        rodata[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
}

fn align8(bytes: &mut Vec<u8>) {
    let rem = bytes.len() % 8;
    if rem != 0 {
        bytes.extend(vec![0u8; 8 - rem]);
    }
}

fn align16(v: u32) -> u32 {
    (v + 15) & !15
}

fn emit_vm_call(enc: &mut A64Encoder, off: u32) {
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X8, base: RegX::X0, offset: 0 });
    enc.push(A64Instruction::LdrRegOffset { dst: RegX::X9, base: RegX::X8, offset: off });
    enc.push(A64Instruction::Blr(RegX::X9));
}

fn emit_env_call(enc: &mut A64Encoder, off: u32) {
    emit_vm_call(enc, off);
}

fn mov_u32(enc: &mut A64Encoder, dst: RegX, value: u32) {
    enc.push(A64Instruction::MovzImm16 { dst, imm: value as u16 });
    if value > 0xFFFF {
        enc.push(A64Instruction::MovkImm16Shift16 { dst, imm: (value >> 16) as u16 });
    }
}

fn patch_bl(text: &mut [u8], insn_off: u32, target: u32) -> Result<()> {
    let imm26 = i32::try_from((target as i64 - insn_off as i64) / 4).map_err(|_| miette!("bl 溢出"))?;
    let patched = 0x94000000u32 | (imm26 as u32 & 0x03FF_FFFF);
    text[insn_off as usize..insn_off as usize + 4].copy_from_slice(&patched.to_le_bytes());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::elf::SharedElfWriter;

    #[test]
    fn jni_glue_writes_loadable_so() {
        let jni = emit_jni_glue_module().expect("emit");
        assert!(jni.module.text.len() > 4);
        let (image, exports) =
            merge_jni_and_logic(jni, ret_bytes().to_vec(), vec![SharedObjectExport { name: "asgard_invoke_export".into(), text_offset: 0 }])
                .expect("merge");
        let so = SharedElfWriter::write_aarch64(&image, &exports).expect("so");
        assert!(so.starts_with(b"\x7fELF"));
        let names = String::from_utf8_lossy(&so);
        assert!(names.contains("JNI_OnLoad"));
        assert!(names.contains("asgard_invoke_export"));
        assert!(names.contains("asgard_patch_native"));
    }
}
