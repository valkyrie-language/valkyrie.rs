use std::collections::BTreeMap;

use crate::{
    metadata::{ClrMetadataBuilder, TableKind},
    msil::{MsilMethodSignature, MsilType},
};

/// 元数据表构建所需的堆偏移信息。
pub(crate) struct TableBuildInfo<'a> {
    /// 程序集名称在 `#Strings` 中的偏移。
    pub assembly_name_offset: u32,
    /// 模块名称在 `#Strings` 中的偏移。
    pub module_name_offset: u32,
    /// `<Module>` 类型名称在 `#Strings` 中的偏移。
    pub module_type_name_offset: u32,
    /// 各方法名称在 `#Strings` 中的偏移（全局方法 + 结构体方法）。
    pub method_name_offsets: &'a [u32],
    /// 各方法签名在 `#Blob` 中的偏移（全局方法 + 结构体方法）。
    pub method_sig_offsets: &'a [u32],
    /// 各方法的 `MethodDef` 标志（`MethodAttributes`，ECMA-335 II.23.1.10）。
    pub method_flags: &'a [u16],
    /// 全局方法数量（用于计算结构体方法的 MethodDef 起始行号）。
    pub global_method_count: u32,
    /// 用户自定义类型定义行。
    pub user_type_defs: &'a [UserTypeDefRow],
    /// `Field` 表行。
    pub field_rows: &'a [FieldRow],
    /// 外部程序集引用表。
    pub assembly_ref_rows: &'a [AssemblyRefRow],
    /// 外部类型引用表。
    pub type_ref_rows: &'a [TypeRefRow],
    /// 外部成员引用表。
    pub member_ref_rows: &'a [MemberRefRow],
    /// `StandAloneSig` 表中各行的 `#Blob` 偏移。
    pub standalone_sig_blob_offsets: &'a [u32],
    /// `System.Object` 的 `TypeRef` 行号（用于用户类型的 `Extends` 字段）。
    pub system_object_type_ref_row: u16,
    /// `System.ValueType` 的 `TypeRef` 行号（用于结构体的 `Extends` 字段）。
    pub system_value_type_type_ref_row: u16,
}

/// 用户自定义 `TypeDef` 表行信息。
pub(crate) struct UserTypeDefRow {
    /// 类型名在 `#Strings` 中的偏移。
    pub type_name_offset: u16,
    /// 类型命名空间在 `#Strings` 中的偏移。
    pub type_namespace_offset: u16,
    /// `FieldList`：指向 `Field` 表中第一个字段的行号（1-indexed）。
    pub field_list: u16,
    /// `MethodList`：指向 `MethodDef` 表中第一个方法的行号（1-indexed）。
    pub method_list: u16,
    /// 是否为值类型（`structure`）。`true` 时 `Extends` 指向 `System.ValueType`，
    /// `Flags` 使用 `SequentialLayout`；`false` 时 `Extends` 指向 `System.Object`，
    /// `Flags` 使用 `BeforeFieldInit`。
    pub is_value_type: bool,
}

/// `Field` 表行信息。
pub(crate) struct FieldRow {
    /// 字段标志（`FieldAttributes`，ECMA-335 II.23.1.5）。
    pub flags: u16,
    /// 字段名在 `#Strings` 中的偏移。
    pub name_offset: u16,
    /// 字段签名在 `#Blob` 中的偏移。
    pub signature_offset: u16,
}

/// `AssemblyRef` 表行。
pub(crate) struct AssemblyRefRow {
    pub name_offset: u16,
}

/// `TypeRef` 表行。
pub(crate) struct TypeRefRow {
    pub resolution_scope: u16,
    pub type_name_offset: u16,
    pub type_namespace_offset: u16,
}

/// `MemberRef` 表行。
pub(crate) struct MemberRefRow {
    pub class: u16,
    pub name_offset: u16,
    pub signature_offset: u16,
}

/// 构建 `#~` 表流。
pub(crate) fn build_tables_stream(metadata: &ClrMetadataBuilder, info: &TableBuildInfo) -> Vec<u8> {
    let method_count = metadata.method_rvas.len() as u32;
    let type_ref_count = info.type_ref_rows.len() as u32;
    let member_ref_count = info.member_ref_rows.len() as u32;
    let assembly_ref_count = info.assembly_ref_rows.len() as u32;
    let standalone_sig_count = info.standalone_sig_blob_offsets.len() as u32;
    let user_type_count = info.user_type_defs.len() as u32;
    let field_count = info.field_rows.len() as u32;
    let type_def_count = 1u32 + user_type_count;

    // 表顺序按 ECMA-335 II.22 排序：Module(0x00), TypeRef(0x01), TypeDef(0x02),
    // Field(0x04), MethodDef(0x06), Param(0x08), MemberRef(0x0A), StandAloneSig(0x11),
    // Assembly(0x20), AssemblyRef(0x23)。
    let valid: u64 = (1u64 << TableKind::Module as u8)
        | ((type_ref_count > 0) as u64) << TableKind::TypeRef as u8
        | (1u64 << TableKind::TypeDef as u8)
        | ((field_count > 0) as u64) << TableKind::Field as u8
        | (1u64 << TableKind::MethodDef as u8)
        | ((member_ref_count > 0) as u64) << TableKind::MemberRef as u8
        | ((standalone_sig_count > 0) as u64) << TableKind::StandAloneSig as u8
        | (1u64 << TableKind::Assembly as u8)
        | ((assembly_ref_count > 0) as u64) << TableKind::AssemblyRef as u8;

    let mut bytes = Vec::new();

    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.push(2);
    bytes.push(0);
    bytes.push(0);
    bytes.push(1);
    bytes.extend_from_slice(&valid.to_le_bytes());
    bytes.extend_from_slice(&0u64.to_le_bytes());

    // 行数按表编号升序写入。
    bytes.extend_from_slice(&1u32.to_le_bytes());
    if type_ref_count > 0 {
        bytes.extend_from_slice(&type_ref_count.to_le_bytes());
    }
    bytes.extend_from_slice(&type_def_count.to_le_bytes());
    if field_count > 0 {
        bytes.extend_from_slice(&field_count.to_le_bytes());
    }
    bytes.extend_from_slice(&method_count.to_le_bytes());
    if member_ref_count > 0 {
        bytes.extend_from_slice(&member_ref_count.to_le_bytes());
    }
    if standalone_sig_count > 0 {
        bytes.extend_from_slice(&standalone_sig_count.to_le_bytes());
    }
    bytes.extend_from_slice(&1u32.to_le_bytes());
    if assembly_ref_count > 0 {
        bytes.extend_from_slice(&assembly_ref_count.to_le_bytes());
    }

    // Module 表（1 行）。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&(info.module_name_offset as u16).to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());

    // TypeRef 表。
    for row in info.type_ref_rows {
        bytes.extend_from_slice(&row.resolution_scope.to_le_bytes());
        bytes.extend_from_slice(&row.type_name_offset.to_le_bytes());
        bytes.extend_from_slice(&row.type_namespace_offset.to_le_bytes());
    }

    // TypeDef 表：`<Module>` 行 + 用户类型行。
    // `<Module>` 行。
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(info.module_type_name_offset as u16).to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // FieldList=1：无 Field 表时指向末尾+1（ECMA-335 II.22.37）。
    bytes.extend_from_slice(&1u16.to_le_bytes());
    // MethodList=1：指向 MethodDef 第 1 行。
    bytes.extend_from_slice(&1u16.to_le_bytes());

    // 用户类型行：根据 `is_value_type` 选择 `Flags` 和 `Extends`。
    // - 值类型（structure）：`SequentialLayout | Public` (0x00000009)，`Extends` = `System.ValueType`。
    // - 引用类型（class）：`BeforeFieldInit | Public` (0x00100001)，`Extends` = `System.Object`。
    let system_object_extends = ((info.system_object_type_ref_row as u32) << 2) | 0x0001;
    let system_value_type_extends = ((info.system_value_type_type_ref_row as u32) << 2) | 0x0001;
    for row in info.user_type_defs {
        let (flags, extends) =
            if row.is_value_type { (0x00000009u32, system_value_type_extends) } else { (0x00100001u32, system_object_extends) };
        bytes.extend_from_slice(&flags.to_le_bytes());
        bytes.extend_from_slice(&row.type_name_offset.to_le_bytes());
        bytes.extend_from_slice(&row.type_namespace_offset.to_le_bytes());
        bytes.extend_from_slice(&(extends as u16).to_le_bytes());
        bytes.extend_from_slice(&row.field_list.to_le_bytes());
        bytes.extend_from_slice(&row.method_list.to_le_bytes());
    }

    // Field 表（ECMA-335 II.22.15）：Flags(2) + Name(2) + Signature(2)。
    for row in info.field_rows {
        bytes.extend_from_slice(&row.flags.to_le_bytes());
        bytes.extend_from_slice(&row.name_offset.to_le_bytes());
        bytes.extend_from_slice(&row.signature_offset.to_le_bytes());
    }

    // MethodDef 表。
    for i in 0..method_count as usize {
        let rva = metadata.method_rvas[i];
        let name_idx = info.method_name_offsets[i] as u16;
        let sig_idx = info.method_sig_offsets[i] as u16;
        let flags = info.method_flags[i];
        bytes.extend_from_slice(&rva.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&flags.to_le_bytes());
        bytes.extend_from_slice(&name_idx.to_le_bytes());
        bytes.extend_from_slice(&sig_idx.to_le_bytes());
        // ParamList=1：无 Param 表时，指向"末尾 + 1"（ECMA-335 II.22.27）。
        bytes.extend_from_slice(&1u16.to_le_bytes());
    }

    // MemberRef 表。
    for row in info.member_ref_rows {
        bytes.extend_from_slice(&row.class.to_le_bytes());
        bytes.extend_from_slice(&row.name_offset.to_le_bytes());
        bytes.extend_from_slice(&row.signature_offset.to_le_bytes());
    }

    // StandAloneSig 表：每行仅有一个 Blob 索引（`Count` 字段在表外）。
    // ECMA-335 II.22.15: StandAloneSig 表行 = Signature (Blob heap index)。
    for &blob_offset in info.standalone_sig_blob_offsets {
        bytes.extend_from_slice(&(blob_offset as u16).to_le_bytes());
    }

    // Assembly 表。
    // HashAlgId：CALG_SHA1（0x00008004）。
    bytes.extend_from_slice(&0x00008004u32.to_le_bytes());
    // MajorVersion。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // MinorVersion。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // BuildNumber。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // RevisionNumber。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // Flags。
    bytes.extend_from_slice(&0u32.to_le_bytes());
    // PublicKey。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // Name。
    bytes.extend_from_slice(&(info.assembly_name_offset as u16).to_le_bytes());
    // Culture。
    bytes.extend_from_slice(&0u16.to_le_bytes());

    // AssemblyRef 表。
    for row in info.assembly_ref_rows {
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&row.name_offset.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
    }

    bytes
}

/// 根据方法签名构建方法签名 blob。
///
/// `has_this` 为 `true` 时添加 `HASTHIS` 标志（0x20），用于实例方法（含构造函数）。
/// `type_token_map` 用于将 `Named` 类型解析为 `VALUETYPE` + TypeDef token。
pub(crate) fn build_method_signature(signature: &MsilMethodSignature, has_this: bool, type_token_map: &BTreeMap<String, u32>) -> Vec<u8> {
    let mut bytes = Vec::new();
    let flags = if has_this { 0x20 } else { 0x00 };
    bytes.push(flags);
    bytes.push(signature.parameter_types.len() as u8);
    bytes.extend_from_slice(&encode_element_type(&signature.return_type, type_token_map));
    for parameter_type in &signature.parameter_types {
        bytes.extend_from_slice(&encode_element_type(parameter_type, type_token_map));
    }
    bytes
}

/// 根据字段类型构建字段签名 blob（`ECMA-335` II.23.2.4）。
///
/// 格式：`0x06 (FIELD) + ELEMENT_TYPE`。
pub(crate) fn build_field_signature(field_type: &MsilType, type_token_map: &BTreeMap<String, u32>) -> Vec<u8> {
    let mut bytes = vec![0x06];
    bytes.extend_from_slice(&encode_element_type(field_type, type_token_map));
    bytes
}

/// 根据局部变量类型列表构建 `LocalVarSig` blob。
///
/// 格式参考 `ECMA-335` II.23.2：`0x07 (LOCAL_SIG) + count + N × ELEMENT_TYPE`。
pub(crate) fn build_local_var_sig(local_types: &[MsilType], type_token_map: &BTreeMap<String, u32>) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(0x07);
    bytes.push(local_types.len() as u8);
    for type_name in local_types {
        bytes.extend_from_slice(&encode_element_type(type_name, type_token_map));
    }
    bytes
}

/// 生成稳定的伪 `MVID`。
pub(crate) fn build_mvid(assembly_name: &str, module_name: &str) -> [u8; 16] {
    let mut guid = [0u8; 16];
    for (index, byte) in assembly_name.bytes().chain(module_name.bytes()).enumerate() {
        let slot = index % 16;
        guid[slot] = guid[slot].wrapping_mul(37).wrapping_add(byte).wrapping_add(slot as u8);
    }
    guid[6] = (guid[6] & 0x0F) | 0x40;
    guid[8] = (guid[8] & 0x3F) | 0x80;
    guid
}

/// 将 `MsilType` 编码为 `ELEMENT_TYPE` 序列（`ECMA-335` II.23.1）。
///
/// `Named` 类型通过 `type_token_map` 解析为 `ELEMENT_TYPE_VALUETYPE` (0x11) + 压缩 TypeDefOrRef token。
/// 若 `Named` 类型不在映射中，回退为 `ELEMENT_TYPE_OBJECT` (0x1C)。
fn encode_element_type(ty: &MsilType, type_token_map: &BTreeMap<String, u32>) -> Vec<u8> {
    match ty {
        MsilType::Void => vec![0x01],
        MsilType::Bool => vec![0x02],
        MsilType::Char => vec![0x03],
        MsilType::Int8 => vec![0x04],
        MsilType::UInt8 => vec![0x05],
        MsilType::Int16 => vec![0x06],
        MsilType::UInt16 => vec![0x07],
        MsilType::Int32 => vec![0x08],
        MsilType::UInt32 => vec![0x09],
        MsilType::Int64 => vec![0x0A],
        MsilType::UInt64 => vec![0x0B],
        MsilType::Float32 => vec![0x0C],
        MsilType::Float64 => vec![0x0D],
        MsilType::String => vec![0x0E],
        MsilType::IntPtr => vec![0x18],
        MsilType::UIntPtr => vec![0x19],
        MsilType::Object => vec![0x1C],
        MsilType::Named(name) => {
            if let Some(token) = type_token_map.get(name) {
                let row = token & 0x00FF_FFFF;
                let mut bytes = vec![0x11];
                bytes.extend_from_slice(&compress_type_def_or_ref(row, 0));
                bytes
            }
            else {
                vec![0x1C]
            }
        }
        MsilType::SzArray(element) => {
            let mut bytes = vec![0x1D];
            bytes.extend_from_slice(&encode_element_type(element, type_token_map));
            bytes
        }
    }
}

/// 压缩 `TypeDefOrRef` 编码（`ECMA-335` II.23.2）。
///
/// `tag`：0 = TypeDef，1 = TypeRef，2 = TypeSpec。
/// 压缩后的值 = `(row << 2) | tag`，再按压缩整数规则编码。
fn compress_type_def_or_ref(row: u32, tag: u32) -> Vec<u8> {
    let value = (row << 2) | (tag & 0x3);
    if value < 0x80 {
        vec![value as u8]
    }
    else if value < 0x4000 {
        vec![0x80 | ((value >> 8) as u8 & 0x3F), (value & 0xFF) as u8]
    }
    else {
        vec![0xC0 | ((value >> 24) as u8 & 0x1F), ((value >> 16) & 0xFF) as u8, ((value >> 8) & 0xFF) as u8, (value & 0xFF) as u8]
    }
}
