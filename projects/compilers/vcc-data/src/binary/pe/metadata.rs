use std::collections::{BTreeMap, BTreeSet};

use crate::text::msil::{MsilMethodSignature, MsilType};

use super::meta_builder::{ClrMetadataBuilder, TableKind};

/// 本地类型 token 映射，同时记录每个类型是否为值类型。
///
/// CLR 签名编码要求值类型使用 `ELEMENT_TYPE_VALUETYPE` (0x11)，
/// 引用类型使用 `ELEMENT_TYPE_CLASS` (0x12)。若混淆二者，
/// 运行时会在加载类型时抛出 `TypeLoadException: The signature is incorrect.`。
pub(crate) struct TypeTokenMap {
    /// 类型名到 `TypeDef` token 的映射。
    pub tokens: BTreeMap<String, u32>,
    /// 值类型（`structure`）的名称集合；不在此集合中的 `Named` 类型为引用类型。
    pub value_type_names: BTreeSet<String>,
    /// TypeDef tokens that are valuetypes — preferred over name lookup when encoding signatures.
    pub value_type_tokens: BTreeSet<u32>,
}

impl TypeTokenMap {
    /// 创建一个空映射，用于外部类型引用（不解析任何本地 `TypeDef`）。
    pub fn empty() -> Self {
        Self { tokens: BTreeMap::new(), value_type_names: BTreeSet::new(), value_type_tokens: BTreeSet::new() }
    }
}

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
    /// 各方法的 `MethodImplAttributes`（ECMA-335 II.23.1.11）。
    pub method_impl_flags: &'a [u16],
    /// 全局方法数量（用于计算结构体方法的 MethodDef 起始行号）。
    pub global_method_count: u32,
    /// `_GlobalMethods` 容器类型名在 `#Strings` 中的偏移。
    pub global_methods_type_name_offset: u32,
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
    /// `TypeSpec` 表行（`castclass`/`unbox.any` 的 `T[]` 等）。
    pub type_spec_rows: &'a [TypeSpecRow],
    /// `StandAloneSig` 表中各行的 `#Blob` 偏移。
    pub standalone_sig_blob_offsets: &'a [u32],
    /// `System.Object` 的 `TypeRef` 行号（用于用户类型的 `Extends` 字段）。
    pub system_object_type_ref_row: u16,
    /// `System.ValueType` 的 `TypeRef` 行号（用于结构体的 `Extends` 字段）。
    pub system_value_type_type_ref_row: u16,
    /// `Param` 表行（按 `MethodDef` 顺序追加）。
    pub param_rows: &'a [ParamRow],
    /// 各 `MethodDef` 行的 `ParamList`（1-indexed；无参时指向 `Param` 表末尾 + 1）。
    pub method_param_lists: &'a [u16],
}

/// `Param` 表行信息（ECMA-335 II.22.33）。
pub(crate) struct ParamRow {
    /// 参数标志（`ParamAttributes`）。
    pub flags: u16,
    /// 参数序号（从 1 开始；返回值固定为 0，此处仅写形参）。
    pub sequence: u16,
    /// 参数名在 `#Strings` 中的偏移（0 表示无名）。宽堆时为 4 字节索引。
    pub name_offset: u32,
}

/// 用户自定义 `TypeDef` 表行信息。
pub(crate) struct UserTypeDefRow {
    /// 类型名在 `#Strings` 中的偏移。
    pub type_name_offset: u32,
    /// 类型命名空间在 `#Strings` 中的偏移。
    pub type_namespace_offset: u32,
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
    pub name_offset: u32,
    /// 字段签名在 `#Blob` 中的偏移。
    pub signature_offset: u32,
}

/// `AssemblyRef` 表行。
pub(crate) struct AssemblyRefRow {
    pub name_offset: u32,
}

/// `TypeRef` 表行。
pub(crate) struct TypeRefRow {
    pub resolution_scope: u16,
    pub type_name_offset: u32,
    pub type_namespace_offset: u32,
}

/// `MemberRef` 表行。
pub(crate) struct MemberRefRow {
    pub class: u16,
    pub name_offset: u32,
    pub signature_offset: u32,
}

/// `TypeSpec` 表行（ECMA-335 II.22.39）：仅 `#Blob` 中的 TypeSpecBlob。
pub(crate) struct TypeSpecRow {
    pub signature_offset: u32,
}

/// ECMA-335 II.24.2.6 `HeapSizes`：bit0=#Strings、bit1=#GUID、bit2=#Blob 使用 4 字节索引。
fn heap_sizes_flags(strings_len: usize, guid_len: usize, blob_len: usize) -> u8 {
    let mut flags = 0u8;
    if strings_len > 0xFFFF {
        flags |= 0x01;
    }
    if guid_len > 0xFFFF {
        flags |= 0x02;
    }
    if blob_len > 0xFFFF {
        flags |= 0x04;
    }
    flags
}

fn push_heap_index(bytes: &mut Vec<u8>, offset: u32, wide: bool) {
    if wide {
        bytes.extend_from_slice(&offset.to_le_bytes());
    }
    else {
        bytes.extend_from_slice(&(offset as u16).to_le_bytes());
    }
}

/// 构建 `#~` 表流。
pub(crate) fn build_tables_stream(metadata: &ClrMetadataBuilder, info: &TableBuildInfo) -> Vec<u8> {
    let method_count = metadata.method_rvas.len() as u32;
    let type_ref_count = info.type_ref_rows.len() as u32;
    let member_ref_count = info.member_ref_rows.len() as u32;
    let assembly_ref_count = info.assembly_ref_rows.len() as u32;
    let type_spec_count = info.type_spec_rows.len() as u32;
    let standalone_sig_count = info.standalone_sig_blob_offsets.len() as u32;
    let user_type_count = info.user_type_defs.len() as u32;
    let field_count = info.field_rows.len() as u32;
    let param_count = info.param_rows.len() as u32;
    // TypeDef 行数：`<Module>` + `_GlobalMethods` + 用户类型。
    let type_def_count = 2u32 + user_type_count;

    // 表顺序按 ECMA-335 II.22 排序：Module(0x00), TypeRef(0x01), TypeDef(0x02),
    // Field(0x04), MethodDef(0x06), Param(0x08), MemberRef(0x0A), StandAloneSig(0x11),
    // TypeSpec(0x1B), Assembly(0x20), AssemblyRef(0x23)。
    let valid: u64 = (1u64 << TableKind::Module as u8)
        | ((type_ref_count > 0) as u64) << TableKind::TypeRef as u8
        | (1u64 << TableKind::TypeDef as u8)
        | ((field_count > 0) as u64) << TableKind::Field as u8
        | (1u64 << TableKind::MethodDef as u8)
        | ((param_count > 0) as u64) << TableKind::Param as u8
        | ((member_ref_count > 0) as u64) << TableKind::MemberRef as u8
        | ((standalone_sig_count > 0) as u64) << TableKind::StandAloneSig as u8
        | ((type_spec_count > 0) as u64) << TableKind::TypeSpec as u8
        | (1u64 << TableKind::Assembly as u8)
        | ((assembly_ref_count > 0) as u64) << TableKind::AssemblyRef as u8;

    let mut bytes = Vec::new();

    let str_wide = metadata.strings.data().len() > 0xFFFF;
    let guid_wide = metadata.guid.data().len() > 0xFFFF;
    let blob_wide = metadata.blob.data().len() > 0xFFFF;
    let heap_sizes = heap_sizes_flags(metadata.strings.data().len(), metadata.guid.data().len(), metadata.blob.data().len());

    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.push(2);
    bytes.push(0);
    bytes.push(heap_sizes);
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
    if param_count > 0 {
        bytes.extend_from_slice(&param_count.to_le_bytes());
    }
    if member_ref_count > 0 {
        bytes.extend_from_slice(&member_ref_count.to_le_bytes());
    }
    if standalone_sig_count > 0 {
        bytes.extend_from_slice(&standalone_sig_count.to_le_bytes());
    }
    if type_spec_count > 0 {
        bytes.extend_from_slice(&type_spec_count.to_le_bytes());
    }
    bytes.extend_from_slice(&1u32.to_le_bytes());
    if assembly_ref_count > 0 {
        bytes.extend_from_slice(&assembly_ref_count.to_le_bytes());
    }

    // Module 表（1 行）。
    bytes.extend_from_slice(&0u16.to_le_bytes());
    push_heap_index(&mut bytes, info.module_name_offset, str_wide);
    push_heap_index(&mut bytes, 1, guid_wide);
    push_heap_index(&mut bytes, 0, guid_wide);
    push_heap_index(&mut bytes, 0, guid_wide);

    // TypeRef 表。
    for row in info.type_ref_rows {
        bytes.extend_from_slice(&row.resolution_scope.to_le_bytes());
        push_heap_index(&mut bytes, row.type_name_offset, str_wide);
        push_heap_index(&mut bytes, row.type_namespace_offset, str_wide);
    }

    // TypeDef 表：`<Module>` 行 + `_GlobalMethods` 容器行 + 用户类型行。
    // `Extends` 编码（ECMA-335 II.23.2.23）：TypeRef 行号左移 2 位 + TypeDefOrRefTag。
    let system_object_extends = ((info.system_object_type_ref_row as u32) << 2) | 0x0001;
    let system_value_type_extends = ((info.system_value_type_type_ref_row as u32) << 2) | 0x0001;

    // `<Module>` 行：不拥有任何方法或字段。
    bytes.extend_from_slice(&0u32.to_le_bytes());
    push_heap_index(&mut bytes, info.module_type_name_offset, str_wide);
    push_heap_index(&mut bytes, 0, str_wide);
    bytes.extend_from_slice(&0u16.to_le_bytes());
    // FieldList=1：与下一个 TypeDef 的 FieldList 相同，拥有空字段集合。
    bytes.extend_from_slice(&1u16.to_le_bytes());
    // MethodList=1：与 `_GlobalMethods` 的 MethodList 相同，使 `<Module>` 拥有空方法集合。
    // TypeDef 的 MethodList 必须单调非递减（ECMA-335 II.22.37），因此不能使用 global_method_count+1。
    bytes.extend_from_slice(&1u16.to_le_bytes());

    // `_GlobalMethods` 容器行：承载所有全局方法，使 `.NET Core` 能识别入口点。
    // .NET Core 移除了对 `<Module>` 全局方法的支持，所有全局方法必须归属于一个真实类型。
    // Flags = BeforeFieldInit | Public (0x00100001)，Extends = System.Object。
    bytes.extend_from_slice(&0x00100001u32.to_le_bytes());
    push_heap_index(&mut bytes, info.global_methods_type_name_offset, str_wide);
    push_heap_index(&mut bytes, 0, str_wide);
    bytes.extend_from_slice(&(system_object_extends as u16).to_le_bytes());
    // FieldList=1：与下一个 TypeDef 的 FieldList 相同，拥有空字段集合。
    bytes.extend_from_slice(&1u16.to_le_bytes());
    // MethodList=1：指向 MethodDef 第 1 行（第一个全局方法）。
    bytes.extend_from_slice(&1u16.to_le_bytes());

    // 用户类型行：根据 `is_value_type` 选择 `Flags` 和 `Extends`。
    // - 值类型（structure）：`Public | SequentialLayout | Sealed` (0x00000109)，`Extends` = `System.ValueType`。
    // - 引用类型（class）：`BeforeFieldInit | Public` (0x00100001)，`Extends` = `System.Object`。
    // ECMA-335: valuetype / enum / delegate TypeDefs MUST be Sealed; omitting Sealed yields
    // PEVerify `[MD] TypeDef is a Value Type ... but not marked Sealed` and runtime BadImage.
    for row in info.user_type_defs {
        let (flags, extends) =
            if row.is_value_type { (0x00000109u32, system_value_type_extends) } else { (0x00100001u32, system_object_extends) };
        bytes.extend_from_slice(&flags.to_le_bytes());
        push_heap_index(&mut bytes, row.type_name_offset, str_wide);
        push_heap_index(&mut bytes, row.type_namespace_offset, str_wide);
        bytes.extend_from_slice(&(extends as u16).to_le_bytes());
        bytes.extend_from_slice(&row.field_list.to_le_bytes());
        bytes.extend_from_slice(&row.method_list.to_le_bytes());
    }

    // Field 表（ECMA-335 II.22.15）：Flags(2) + Name(str) + Signature(blob)。
    for row in info.field_rows {
        bytes.extend_from_slice(&row.flags.to_le_bytes());
        push_heap_index(&mut bytes, row.name_offset, str_wide);
        push_heap_index(&mut bytes, row.signature_offset, blob_wide);
    }

    // MethodDef 表。
    for i in 0..method_count as usize {
        let rva = metadata.method_rvas[i];
        let impl_flags = info.method_impl_flags[i];
        let flags = info.method_flags[i];
        bytes.extend_from_slice(&rva.to_le_bytes());
        bytes.extend_from_slice(&impl_flags.to_le_bytes());
        bytes.extend_from_slice(&flags.to_le_bytes());
        push_heap_index(&mut bytes, info.method_name_offsets[i], str_wide);
        push_heap_index(&mut bytes, info.method_sig_offsets[i], blob_wide);
        let param_list = info.method_param_lists.get(i).copied().unwrap_or(1);
        bytes.extend_from_slice(&param_list.to_le_bytes());
    }

    // Param 表（ECMA-335 II.22.33）：Flags(2) + Sequence(2) + Name(str)。
    for row in info.param_rows {
        bytes.extend_from_slice(&row.flags.to_le_bytes());
        bytes.extend_from_slice(&row.sequence.to_le_bytes());
        push_heap_index(&mut bytes, row.name_offset, str_wide);
    }

    // MemberRef 表。
    for row in info.member_ref_rows {
        bytes.extend_from_slice(&row.class.to_le_bytes());
        push_heap_index(&mut bytes, row.name_offset, str_wide);
        push_heap_index(&mut bytes, row.signature_offset, blob_wide);
    }

    // StandAloneSig 表：每行仅有一个 Blob 索引（`Count` 字段在表外）。
    // ECMA-335 II.22.15: StandAloneSig 表行 = Signature (Blob heap index)。
    for &blob_offset in info.standalone_sig_blob_offsets {
        push_heap_index(&mut bytes, blob_offset, blob_wide);
    }

    // TypeSpec 表（ECMA-335 II.22.39）：Signature (Blob heap index)。
    for row in info.type_spec_rows {
        push_heap_index(&mut bytes, row.signature_offset, blob_wide);
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
    push_heap_index(&mut bytes, 0, blob_wide);
    // Name。
    push_heap_index(&mut bytes, info.assembly_name_offset, str_wide);
    // Culture。
    push_heap_index(&mut bytes, 0, str_wide);

    // AssemblyRef 表。
    for row in info.assembly_ref_rows {
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        push_heap_index(&mut bytes, 0, blob_wide);
        push_heap_index(&mut bytes, row.name_offset, str_wide);
        push_heap_index(&mut bytes, 0, str_wide);
        push_heap_index(&mut bytes, 0, blob_wide);
    }

    bytes
}

/// 根据方法签名构建方法签名 blob。
///
/// `has_this` 为 `true` 时添加 `HASTHIS` 标志（0x20），用于实例方法（含构造函数）。
/// `type_token_map` 用于将 `Named` 类型解析为 `VALUETYPE`/`CLASS` + TypeDef token。
pub(crate) fn build_method_signature(signature: &MsilMethodSignature, has_this: bool, type_token_map: &TypeTokenMap) -> Vec<u8> {
    let mut bytes = Vec::new();
    let flags = if has_this || signature.has_this { 0x20 } else { 0x00 };
    bytes.push(flags);
    bytes.extend_from_slice(&compress_unsigned(signature.parameter_types.len() as u32));
    bytes.extend_from_slice(&encode_element_type(&signature.return_type, type_token_map));
    for parameter_type in &signature.parameter_types {
        bytes.extend_from_slice(&encode_element_type(parameter_type, type_token_map));
    }
    bytes
}

/// 根据字段类型构建字段签名 blob（`ECMA-335` II.23.2.4）。
///
/// 格式：`0x06 (FIELD) + ELEMENT_TYPE`。
pub(crate) fn build_field_signature(field_type: &MsilType, type_token_map: &TypeTokenMap) -> Vec<u8> {
    let mut bytes = vec![0x06];
    bytes.extend_from_slice(&encode_element_type(field_type, type_token_map));
    bytes
}

/// 根据局部变量类型列表构建 `LocalVarSig` blob。
///
/// 格式参考 `ECMA-335` II.23.2：`0x07 (LOCAL_SIG) + Count + N × LocalVarType`。
/// `Count` 必须是压缩无符号整数；单字节截断在 ≥128 个局部时会把下一字节
/// （真实 `ELEMENT_TYPE`）吞进 count，导致 StandAloneSig 出现非法 `ELEMENT_TYPE`
///（如 PEVerify 报告的 `0x204`）并让方法验证失败。
pub(crate) fn build_local_var_sig(local_types: &[MsilType], type_token_map: &TypeTokenMap) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.push(0x07);
    bytes.extend_from_slice(&compress_unsigned(local_types.len() as u32));
    for type_name in local_types {
        // ECMA-335 II.23.2.6: LocalVarSig must not use ELEMENT_TYPE_VOID.
        let ty = match type_name {
            MsilType::Void => &MsilType::Int32 { signed: true },
            other => other,
        };
        bytes.extend_from_slice(&encode_element_type(ty, type_token_map));
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
/// `Named` 类型通过 `type_token_map` 解析为 `ELEMENT_TYPE_VALUETYPE` (0x11) 或
/// `ELEMENT_TYPE_CLASS` (0x12) + 压缩 TypeDefOrRef token，取决于该类型是否为值类型。
/// token 前缀决定压缩编码的 tag 位：`0x0200_0000` 为 TypeDef (tag=0)，
/// `0x0100_0000` 为 TypeRef (tag=1)。若 `Named` 类型不在映射中，回退为
/// `ELEMENT_TYPE_OBJECT` (0x1C)。
pub(crate) fn encode_element_type(ty: &MsilType, type_token_map: &TypeTokenMap) -> Vec<u8> {
    match ty {
        MsilType::Void => vec![0x01],
        MsilType::Bool => vec![0x02],
        MsilType::Char => vec![0x03],
        MsilType::Int8 { signed: true } => vec![0x04],
        MsilType::Int8 { signed: false } => vec![0x05],
        MsilType::Int16 { signed: true } => vec![0x06],
        MsilType::Int16 { signed: false } => vec![0x07],
        MsilType::Int32 { signed: true } => vec![0x08],
        MsilType::Int32 { signed: false } => vec![0x09],
        MsilType::Int64 { signed: true } => vec![0x0A],
        MsilType::Int64 { signed: false } => vec![0x0B],
        MsilType::Float32 => vec![0x0C],
        MsilType::Float64 => vec![0x0D],
        MsilType::String => vec![0x0E],
        MsilType::IntPtr { signed: true } => vec![0x18],
        MsilType::IntPtr { signed: false } => vec![0x19],
        MsilType::Object => vec![0x1C],
        MsilType::Named(name) => {
            if let Some(token) = type_token_map.tokens.get(name) {
                let row = token & 0x00FF_FFFF;
                let tag = if (token & 0xFF00_0000) == 0x0100_0000 { 1 } else { 0 };
                // Prefer token membership: name-set lookup can miss alternate qualifications
                // (`TextSpan` vs `std.text.TextSpan`) and emit CLASS for a ValueType TypeDef.
                let element = if type_token_map.value_type_tokens.contains(token) || type_token_map.value_type_names.contains(name) {
                    0x11
                }
                else {
                    0x12
                };
                let mut bytes = vec![element];
                bytes.extend_from_slice(&compress_type_def_or_ref(row, tag));
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
    compress_unsigned((row << 2) | (tag & 0x3))
}

/// ECMA-335 II.23.2 compressed unsigned integer.
fn compress_unsigned(value: u32) -> Vec<u8> {
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
