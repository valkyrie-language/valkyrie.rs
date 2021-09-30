//! 元数据表行大小计算器。
//!
//! 根据 `ECMA-335` II.22 计算各元数据表的行大小，
//! 用于在 `#~` 流中正确跳过未解析的表。
//!
//! 行大小取决于：
//! 1. 堆索引大小（`#Strings` / `#GUID` / `#Blob`，2 或 4 字节）
//! 2. 表索引大小（2 或 4 字节，取决于行数）
//! 3. 编码索引大小（取决于多个表行数的最大值）

use super::pe_parser::MetadataRoot;

/// 编码索引类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodedIndex {
    /// `TypeDefOrRef`：`TypeDef` | `TypeRef` | `TypeSpec`。
    TypeDefOrRef,
    /// `HasConstant`：`Field` | `Param` | `Property`。
    HasConstant,
    /// `HasCustomAttribute`：22 种表。
    HasCustomAttribute,
    /// `HasFieldMarshal`：`Field` | `Param`。
    HasFieldMarshal,
    /// `HasDeclSecurity`：`TypeDef` | `MethodDef` | `Assembly`。
    HasDeclSecurity,
    /// `MemberRefParent`：`TypeDef` | `TypeRef` | `ModuleRef` | `MethodDef` | `TypeSpec`。
    MemberRefParent,
    /// `HasSemantics`：`Event` | `Property`。
    HasSemantics,
    /// `MethodDefOrRef`：`MethodDef` | `MemberRef`。
    MethodDefOrRef,
    /// `MemberForwarded`：`Field` | `MethodDef`。
    MemberForwarded,
    /// `Implementation`：`File` | `AssemblyRef` | `ExportedType`。
    Implementation,
    /// `CustomAttributeType`：`MethodDef` | `MemberRef`。
    CustomAttributeType,
    /// `ResolutionScope`：`Module` | `ModuleRef` | `AssemblyRef` | `TypeRef`。
    ResolutionScope,
    /// `TypeOrMethodDef`：`TypeDef` | `MethodDef`。
    TypeOrMethodDef,
}

impl CodedIndex {
    /// 返回该编码索引的标记位数。
    fn tag_bits(self) -> u32 {
        match self {
            Self::TypeDefOrRef => 2,
            Self::HasConstant => 2,
            Self::HasCustomAttribute => 5,
            Self::HasFieldMarshal => 1,
            Self::HasDeclSecurity => 2,
            Self::MemberRefParent => 3,
            Self::HasSemantics => 1,
            Self::MethodDefOrRef => 1,
            Self::MemberForwarded => 1,
            Self::Implementation => 2,
            Self::CustomAttributeType => 3,
            Self::ResolutionScope => 2,
            Self::TypeOrMethodDef => 1,
        }
    }

    /// 返回该编码索引引用的所有表种类。
    fn referenced_tables(self) -> &'static [u8] {
        match self {
            Self::TypeDefOrRef => &[0x02, 0x01, 0x1B],
            Self::HasConstant => &[0x04, 0x08, 0x17],
            Self::HasCustomAttribute => {
                &[0x06, 0x04, 0x01, 0x02, 0x08, 0x09, 0x0A, 0x00, 0x0E, 0x17, 0x14, 0x11, 0x1A, 0x1B, 0x20, 0x23, 0x26, 0x27, 0x28, 0x2A, 0x2C]
            }
            Self::HasFieldMarshal => &[0x04, 0x08],
            Self::HasDeclSecurity => &[0x02, 0x06, 0x20],
            Self::MemberRefParent => &[0x02, 0x01, 0x1A, 0x06, 0x1B],
            Self::HasSemantics => &[0x14, 0x17],
            Self::MethodDefOrRef => &[0x06, 0x0A],
            Self::MemberForwarded => &[0x04, 0x06],
            Self::Implementation => &[0x26, 0x23, 0x27],
            Self::CustomAttributeType => &[0x06, 0x0A],
            Self::ResolutionScope => &[0x00, 0x1A, 0x23, 0x01],
            Self::TypeOrMethodDef => &[0x02, 0x06],
        }
    }
}

/// 计算编码索引的字节大小（2 或 4）。
fn coded_index_size(md: &MetadataRoot, coded: CodedIndex) -> usize {
    let tag_bits = coded.tag_bits();
    let max_rows: u32 = coded.referenced_tables().iter().map(|&t| md.row_counts[t as usize]).max().unwrap_or(0);
    let threshold = 1u32 << (16 - tag_bits);
    if max_rows < threshold {
        2
    }
    else {
        4
    }
}

/// 计算简单表索引的字节大小（2 或 4）。
fn table_index_size(md: &MetadataRoot, table_id: u8) -> usize {
    if md.row_counts[table_id as usize] > 0xFFFF {
        4
    }
    else {
        2
    }
}

/// 堆索引大小。
fn heap_index_size(heap_size: usize) -> usize {
    if heap_size > 0xFFFF {
        4
    }
    else {
        2
    }
}

/// 计算指定表种类的单行字节数。
///
/// 返回 `Some(size)` 表示已知结构，`None` 表示未实现。
pub fn table_row_size(md: &MetadataRoot, table_id: u8) -> Option<usize> {
    let str_idx = heap_index_size(md.strings.len());
    let guid_idx = heap_index_size(md.guid.len());
    let blob_idx = heap_index_size(md.blob.len());

    match table_id {
        // 0x00 Module: Generation(2) + Name(str) + Mvid(guid) + EncId(guid) + EncBaseId(guid)
        0x00 => Some(2 + str_idx + 3 * guid_idx),
        // 0x01 TypeRef: ResolutionScope(coded) + Name(str) + Namespace(str)
        0x01 => Some(coded_index_size(md, CodedIndex::ResolutionScope) + 2 * str_idx),
        // 0x02 TypeDef: Flags(4) + Name(str) + Namespace(str) + Extends(coded) + FieldList(table) + MethodList(table)
        0x02 => {
            Some(4 + 2 * str_idx + coded_index_size(md, CodedIndex::TypeDefOrRef) + table_index_size(md, 0x04) + table_index_size(md, 0x06))
        }
        // 0x03 FieldPtr: Field(table)
        0x03 => Some(table_index_size(md, 0x04)),
        // 0x04 Field: Flags(2) + Name(str) + Signature(blob)
        0x04 => Some(2 + str_idx + blob_idx),
        // 0x05 MethodPtr: MethodDef(table)
        0x05 => Some(table_index_size(md, 0x06)),
        // 0x06 MethodDef: RVA(4) + ImplFlags(2) + Flags(2) + Name(str) + Signature(blob) + ParamList(table)
        0x06 => Some(4 + 2 + 2 + str_idx + blob_idx + table_index_size(md, 0x08)),
        // 0x07 ParamPtr: Param(table)
        0x07 => Some(table_index_size(md, 0x08)),
        // 0x08 Param: Flags(2) + Sequence(2) + Name(str)
        0x08 => Some(2 + 2 + str_idx),
        // 0x09 InterfaceImpl: Class(table) + Interface(coded)
        0x09 => Some(table_index_size(md, 0x02) + coded_index_size(md, CodedIndex::TypeDefOrRef)),
        // 0x0A MemberRef: Class(coded) + Name(str) + Signature(blob)
        0x0A => Some(coded_index_size(md, CodedIndex::MemberRefParent) + str_idx + blob_idx),
        // 0x0B Constant: Type(1) + Padding(1) + Parent(coded) + Value(blob)
        0x0B => Some(2 + coded_index_size(md, CodedIndex::HasConstant) + blob_idx),
        // 0x0C CustomAttribute: Parent(coded) + Type(coded) + Value(blob)
        0x0C => Some(coded_index_size(md, CodedIndex::HasCustomAttribute) + coded_index_size(md, CodedIndex::CustomAttributeType) + blob_idx),
        // 0x0D FieldMarshal: Parent(coded) + NativeType(blob)
        0x0D => Some(coded_index_size(md, CodedIndex::HasFieldMarshal) + blob_idx),
        // 0x0E DeclSecurity: Action(2) + Parent(coded) + PermissionSet(blob)
        0x0E => Some(2 + coded_index_size(md, CodedIndex::HasDeclSecurity) + blob_idx),
        // 0x0F ClassLayout: PackingSize(2) + ClassSize(4) + Parent(table)
        0x0F => Some(2 + 4 + table_index_size(md, 0x02)),
        // 0x10 FieldLayout: Offset(4) + Field(table)
        0x10 => Some(4 + table_index_size(md, 0x04)),
        // 0x11 StandAloneSig: Signature(blob)
        0x11 => Some(blob_idx),
        // 0x12 EventMap: Parent(table) + EventList(table)
        0x12 => Some(table_index_size(md, 0x02) + table_index_size(md, 0x14)),
        // 0x13 EventPtr: Event(table)
        0x13 => Some(table_index_size(md, 0x14)),
        // 0x14 Event: EventFlags(2) + Name(str) + EventType(coded)
        0x14 => Some(2 + str_idx + coded_index_size(md, CodedIndex::TypeDefOrRef)),
        // 0x15 PropertyMap: Parent(table) + PropertyList(table)
        0x15 => Some(table_index_size(md, 0x02) + table_index_size(md, 0x17)),
        // 0x16 PropertyPtr: Property(table)
        0x16 => Some(table_index_size(md, 0x17)),
        // 0x17 Property: Flags(2) + Name(str) + Type(blob)
        0x17 => Some(2 + str_idx + blob_idx),
        // 0x18 MethodSemantics: Semantics(2) + Method(table) + Association(coded)
        0x18 => Some(2 + table_index_size(md, 0x06) + coded_index_size(md, CodedIndex::HasSemantics)),
        // 0x19 MethodImpl: Class(table) + MethodBody(coded) + MethodDeclaration(coded)
        0x19 => Some(table_index_size(md, 0x02) + 2 * coded_index_size(md, CodedIndex::MethodDefOrRef)),
        // 0x1A ModuleRef: Name(str)
        0x1A => Some(str_idx),
        // 0x1B TypeSpec: Signature(blob)
        0x1B => Some(blob_idx),
        // 0x1C ImplMap: MappingFlags(2) + MemberForwarded(coded) + ImportName(str) + ImportScope(table)
        0x1C => Some(2 + coded_index_size(md, CodedIndex::MemberForwarded) + str_idx + table_index_size(md, 0x1A)),
        // 0x1D FieldRVA: RVA(4) + Field(table)
        0x1D => Some(4 + table_index_size(md, 0x04)),
        // .NET 在 0x1E/0x1F 插入 ENCLog/ENCMap 两张内部表，
        // 导致后续所有 Assembly 系列表编号比 ECMA-335 偏移 +2。
        // 0x1E ENCLog: Token(4) + FuncCode(4)
        0x1E => Some(8),
        // 0x1F ENCMap: Token(4)
        0x1F => Some(4),
        // 0x20 Assembly: HashAlgId(4) + MajorVersion(2) + MinorVersion(2) + BuildNumber(2) + RevisionNumber(2) + Flags(4) + PublicKey(blob) + Name(str) + Culture(str)
        0x20 => Some(4 + 2 + 2 + 2 + 2 + 4 + blob_idx + str_idx + str_idx),
        // 0x21 AssemblyProcessor: Processor(4)
        0x21 => Some(4),
        // 0x22 AssemblyOS: OSPlatformID(4) + OSMajorVersion(4) + OSMinorVersion(4)
        0x22 => Some(12),
        // 0x23 AssemblyRef: MajorVersion(2) + MinorVersion(2) + BuildNumber(2) + RevisionNumber(2) + Flags(4) + PublicKeyOrToken(blob) + Name(str) + Culture(str) + HashValue(blob)
        0x23 => Some(2 + 2 + 2 + 2 + 4 + blob_idx + str_idx + str_idx + blob_idx),
        // 0x24 AssemblyRefProcessor: Processor(4) + AssemblyRef(table)
        0x24 => Some(4 + table_index_size(md, 0x23)),
        // 0x25 AssemblyRefOS: OSPlatformID(4) + OSMajorVersion(4) + OSMinorVersion(4) + AssemblyRef(table)
        0x25 => Some(4 + 4 + 4 + table_index_size(md, 0x23)),
        // 0x26 File: Flags(4) + Name(str) + HashValue(blob)
        0x26 => Some(4 + str_idx + blob_idx),
        // 0x27 ExportedType: Flags(4) + TypeDefId(4) + TypeName(str) + TypeNamespace(str) + Implementation(coded)
        0x27 => Some(4 + 4 + 2 * str_idx + coded_index_size(md, CodedIndex::Implementation)),
        // 0x28 ManifestResource: Offset(4) + Flags(4) + Name(str) + Implementation(coded)
        0x28 => Some(4 + 4 + str_idx + coded_index_size(md, CodedIndex::Implementation)),
        // 0x29 NestedClass: NestedClass(table) + EnclosingClass(table)
        0x29 => Some(2 * table_index_size(md, 0x02)),
        // 0x2A GenericParam: Number(2) + Flags(2) + Owner(coded) + Name(str)
        0x2A => Some(2 + 2 + coded_index_size(md, CodedIndex::TypeOrMethodDef) + str_idx),
        // 0x2B MethodSpec: Method(coded) + Instantiation(blob)
        0x2B => Some(coded_index_size(md, CodedIndex::MethodDefOrRef) + blob_idx),
        // 0x2C GenericParamConstraint: Owner(table) + Constraint(coded)
        0x2C => Some(table_index_size(md, 0x2A) + coded_index_size(md, CodedIndex::TypeDefOrRef)),
        _ => None,
    }
}

/// 计算指定表在 `#~` 流表数据区中的字节偏移。
///
/// 返回 `Some(offset)` 表示成功计算，`None` 表示遇到未知的表结构。
pub fn table_data_offset(md: &MetadataRoot, target_table: u8) -> Option<usize> {
    let mut offset = 0usize;
    for id in 0..64u8 {
        if (md.valid_tables >> id) & 1 == 0 {
            continue;
        }
        if id == target_table {
            return Some(offset);
        }
        let rows = md.row_counts[id as usize] as usize;
        if rows == 0 {
            continue;
        }
        let row_size = table_row_size(md, id)?;
        offset += rows * row_size;
    }
    None
}
