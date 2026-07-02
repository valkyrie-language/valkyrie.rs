/// ELF 可执行镜像的节布局（文件偏移与虚拟地址）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ElfSectionLayout {
    /// `.text` 文件偏移。
    pub text_offset: u32,
    /// `.text` 虚拟地址。
    pub text_vaddr: u32,
    /// `.rodata` 文件偏移（无 rodata 时为 0）。
    pub rodata_offset: u32,
    /// `.rodata` 虚拟地址（无数据时为 0）。
    pub rodata_vaddr: u32,
    /// 文件总大小。
    pub file_size: u32,
}

/// 默认加载基址（非 PIE）。
pub const ELF_LOAD_BASE: u64 = 0x400000;

/// ELF64 头大小。
pub const ELF_EHDR_SIZE: u32 = 64;
/// 程序头表项大小。
pub const ELF_PHDR_SIZE: u32 = 56;

impl ElfSectionLayout {
    /// 根据 `.text` / `.rodata` 长度计算布局。
    pub fn compute(text_len: u32, rodata_len: u32) -> Self {
        let headers = ELF_EHDR_SIZE + ELF_PHDR_SIZE;
        let text_offset = align_up(headers, 16);
        let text_end = text_offset + text_len;
        let (rodata_offset, file_size) = if rodata_len == 0 {
            (0, text_end)
        }
        else {
            let offset = align_up(text_end, 16);
            (offset, offset + rodata_len)
        };
        let base = ELF_LOAD_BASE as u32;
        Self {
            text_offset,
            text_vaddr: base + text_offset,
            rodata_offset,
            rodata_vaddr: if rodata_len == 0 { 0 } else { base + rodata_offset },
            file_size,
        }
    }
}

fn align_up(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    let rem = value % alignment;
    if rem == 0 { value } else { value + (alignment - rem) }
}
