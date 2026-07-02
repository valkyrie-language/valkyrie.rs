/// 基址重定位节布局信息。
pub(crate) struct RelocationLayout {
    /// `.reloc` 节原始字节。
    pub bytes: Vec<u8>,
}

/// 为入口桩中的绝对地址操作数构建最小 `.reloc` 节。
pub(crate) fn build_relocation_section(entry_stub_rva: u32) -> RelocationLayout {
    let relocation_target_rva = entry_stub_rva + 2;
    let page_rva = relocation_target_rva & !0x0FFF;
    let page_offset = (relocation_target_rva - page_rva) as u16;
    let entry = 0x3000u16 | page_offset;

    let mut bytes = Vec::new();
    bytes.extend_from_slice(&page_rva.to_le_bytes());
    bytes.extend_from_slice(&12u32.to_le_bytes());
    bytes.extend_from_slice(&entry.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    RelocationLayout { bytes }
}
