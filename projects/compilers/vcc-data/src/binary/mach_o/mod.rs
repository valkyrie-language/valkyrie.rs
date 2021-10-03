//! Mach-O 可执行文件构建（aarch64）。

use miette::Result;

const MH_MAGIC_64: u32 = 0xFEED_FACF;
const CPU_TYPE_ARM64: i32 = 0x0100_000C;
const CPU_SUBTYPE_ARM64_ALL: i32 = 0;
const MH_EXECUTE: u32 = 0x2;
const LC_SEGMENT_64: u32 = 0x19;
const LC_MAIN: u32 = 0x8000_0028;
const VM_PROT_READ: u32 = 1;
const VM_PROT_WRITE: u32 = 2;
const VM_PROT_EXECUTE: u32 = 4;

/// Mach-O 镜像构建器。
#[derive(Debug, Default)]
pub struct MachOImageBuilder {
    rodata: Vec<u8>,
    code: Vec<u8>,
}

impl MachOImageBuilder {
    /// 新建构建器。
    pub fn new() -> Self {
        Self { rodata: Vec::new(), code: vec![0xC0, 0x03, 0x5F, 0xD6] } // aarch64 ret
    }

    /// 追加只读数据（RenderIR 等）。
    pub fn add_rodata(&mut self, data: &[u8]) -> &mut Self {
        self.rodata.extend_from_slice(data);
        self
    }

    /// 替换可执行代码段。
    pub fn set_code(&mut self, code: &[u8]) -> &mut Self {
        self.code = code.to_vec();
        self
    }

    /// 写出 64-bit Mach-O 可执行文件。
    pub fn build_executable(self) -> Result<Vec<u8>> {
        let page = 0x4000u64;
        let header_size = 32u64;
        let load_main_size = 24u64;
        let seg_cmd_size = 72u64;
        let cmds_size = load_main_size + seg_cmd_size;
        let text_off = header_size + cmds_size;
        let text_size = align(self.code.len() as u64, page);
        let data_off = text_off + text_size;
        let data_size = align(self.rodata.len() as u64, page);
        let file_size = data_off + data_size;

        let mut out = Vec::with_capacity(file_size as usize);
        out.extend_from_slice(&MH_MAGIC_64.to_le_bytes());
        out.extend_from_slice(&CPU_TYPE_ARM64.to_le_bytes());
        out.extend_from_slice(&CPU_SUBTYPE_ARM64_ALL.to_le_bytes());
        out.extend_from_slice(&MH_EXECUTE.to_le_bytes());
        out.extend_from_slice(&2u32.to_le_bytes()); // ncmds
        out.extend_from_slice(&(cmds_size as u32).to_le_bytes());
        out.extend_from_slice(&0x200085u32.to_le_bytes()); // flags

        // LC_SEGMENT_64 __TEXT
        out.extend_from_slice(&LC_SEGMENT_64.to_le_bytes());
        out.extend_from_slice(&(seg_cmd_size as u32).to_le_bytes());
        out.extend_from_slice(b"__TEXT\0\0\0\0\0\0\0\0\0\0");
        out.extend_from_slice(&page.to_le_bytes());
        out.extend_from_slice(&text_size.to_le_bytes());
        out.extend_from_slice(&text_off.to_le_bytes());
        out.extend_from_slice(&(VM_PROT_READ | VM_PROT_EXECUTE).to_le_bytes());
        out.extend_from_slice(&text_size.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());

        // LC_MAIN
        out.extend_from_slice(&LC_MAIN.to_le_bytes());
        out.extend_from_slice(&(load_main_size as u32).to_le_bytes());
        out.extend_from_slice(&text_off.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());
        out.extend_from_slice(&0u64.to_le_bytes());

        pad_to(&mut out, text_off as usize);
        out.extend_from_slice(&self.code);
        pad_to(&mut out, data_off as usize);
        out.extend_from_slice(&self.rodata);
        pad_to(&mut out, file_size as usize);
        Ok(out)
    }
}

fn align(value: u64, boundary: u64) -> u64 {
    ((value + boundary - 1) / boundary) * boundary
}

fn pad_to(buf: &mut Vec<u8>, len: usize) {
    while buf.len() < len {
        buf.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mach_o_magic() {
        let exe = MachOImageBuilder::new().build_executable().unwrap();
        assert_eq!(&exe[..4], &MH_MAGIC_64.to_le_bytes());
    }
}
