use nyar::backends::clr::ClrImageKind;

/// 导入表布局信息。
pub(crate) struct ImportLayout {
    /// 完整导入表字节。
    pub bytes: Vec<u8>,
    /// 原生入口桩的 `RVA`。
    pub entry_stub_rva: u32,
    /// 导入目录的 `RVA`。
    pub import_directory_rva: u32,
    /// 导入目录大小。
    pub import_directory_size: u32,
    /// `IAT` 的 `RVA`。
    pub iat_rva: u32,
}

/// 构建 `_CorExeMain` / `_CorDllMain` 导入表。
pub(crate) fn build_import_table(image_kind: ClrImageKind, import_rva: u32) -> ImportLayout {
    let mut bytes = Vec::new();
    let thunk_name = match image_kind {
        ClrImageKind::Executable => "_CorExeMain",
        ClrImageKind::DynamicLibrary => "_CorDllMain",
    };
    let image_base = 0x0040_0000u32;
    let entry_stub_rva = import_rva;
    let import_directory_rva = import_rva + 8;
    let import_directory_size = 0x28;

    let ilt_rva = import_directory_rva + import_directory_size;
    let iat_rva = ilt_rva + 8;
    let name_rva = iat_rva + 8;
    let hint_name_rva = name_rva + 12;

    // x86 原生入口桩：jmp dword ptr [iat_va]
    bytes.push(0xFF);
    bytes.push(0x25);
    bytes.extend_from_slice(&(image_base + iat_rva).to_le_bytes());
    bytes.extend_from_slice(&[0u8; 2]);

    bytes.extend_from_slice(&ilt_rva.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&name_rva.to_le_bytes());
    bytes.extend_from_slice(&iat_rva.to_le_bytes());

    for _ in 0..5 {
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }

    bytes.extend_from_slice(&hint_name_rva.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());

    bytes.extend_from_slice(&hint_name_rva.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());

    bytes.extend_from_slice(b"mscoree.dll\0");
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(thunk_name.as_bytes());
    bytes.push(0);

    ImportLayout { bytes, entry_stub_rva, import_directory_rva, import_directory_size, iat_rva }
}
