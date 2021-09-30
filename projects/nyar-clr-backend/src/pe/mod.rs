//! `PE/COFF` 二进制写入器。
//!
//! 直接写入 `PE/COFF` 二进制，不依赖 `ilasm`。
//! 参考 `ECMA-335` II.25 和 `valkyrie.cs` 的 `Standard.Data.Binary.Pe`。

use std::{collections::BTreeMap, fmt};

use miette::{Diagnostic, Severity};
use nyar::backends::clr::ClrImageKind;

mod imports;
mod metadata;
mod relocations;
mod tokens;

use self::{
    imports::build_import_table,
    metadata::{
        build_field_signature, build_local_var_sig, build_method_signature, build_mvid, build_tables_stream, FieldRow, TableBuildInfo,
        UserTypeDefRow,
    },
    relocations::build_relocation_section,
    tokens::resolve_module_tokens,
};
use crate::{
    metadata::ClrMetadataBuilder,
    msil::{MethodBodyEncoder, MethodBodyError, MsilModule},
};

const FILE_ALIGNMENT: u32 = 0x200;
const SECTION_ALIGNMENT: u32 = 0x2000;
const TEXT_SECTION_RVA: u32 = SECTION_ALIGNMENT;
const CLI_HEADER_SIZE: u32 = 72;
const METHOD_DEF_TOKEN_BASE: u32 = 0x0600_0000;

/// `PE` 写入选项。
#[derive(Debug, Clone)]
pub struct PeWriterOptions {
    /// 程序集名称。
    pub assembly_name: String,
    /// 模块名称。
    pub module_name: String,
    /// 镜像口味。
    pub image_kind: ClrImageKind,
}

/// `PE` 写入错误。
#[derive(Debug)]
pub enum PeWriterError {
    /// `IO` 错误。
    Io(std::io::Error),
    /// 方法体编码失败。
    MethodBody(MethodBodyError),
    /// 本地方法 token 缺失。
    MissingLocalMethodToken(String),
    /// 外部方法缺少 owner。
    MissingExternalMethodOwner(String),
    /// 外部类型 owner 缺少右中括号。
    ExternalTypeOwnerMissingBracket(String),
    /// 外部类型 owner 无效。
    InvalidExternalTypeOwner(String),
    /// 当前不支持的外部类型 owner。
    UnsupportedExternalTypeOwner(String),
    /// 元数据表项超出范围。
    MetadataIndexOverflow {
        /// 超出范围的元数据项名称。
        what: String,
        /// 超出范围的实际值。
        value: u32,
    },
    /// 无入口点。
    NoEntryPoint,
    /// 模块中没有任何方法体。
    NoMethodBody,
}

impl fmt::Display for PeWriterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "IO 错误: {error}"),
            Self::MethodBody(error) => write!(f, "方法体编码失败: {error}"),
            Self::MissingLocalMethodToken(name) => write!(f, "找不到本地方法 token: {name}"),
            Self::MissingExternalMethodOwner(name) => write!(f, "外部方法缺少 owner: {name}"),
            Self::ExternalTypeOwnerMissingBracket(owner) => write!(f, "外部类型 owner 缺少 `]`: {owner}"),
            Self::InvalidExternalTypeOwner(owner) => write!(f, "外部类型 owner 无效: {owner}"),
            Self::UnsupportedExternalTypeOwner(owner) => write!(f, "当前仅支持显式程序集 owner: {owner}"),
            Self::MetadataIndexOverflow { what, value } => write!(f, "{what} 超出 16 位范围: {value}"),
            Self::NoEntryPoint => write!(f, "无入口点"),
            Self::NoMethodBody => write!(f, "模块中没有任何方法体"),
        }
    }
}

impl std::error::Error for PeWriterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::MethodBody(error) => Some(error),
            Self::MissingLocalMethodToken(_)
            | Self::MissingExternalMethodOwner(_)
            | Self::ExternalTypeOwnerMissingBracket(_)
            | Self::InvalidExternalTypeOwner(_)
            | Self::UnsupportedExternalTypeOwner(_)
            | Self::MetadataIndexOverflow { .. }
            | Self::NoEntryPoint
            | Self::NoMethodBody => None,
        }
    }
}

impl Diagnostic for PeWriterError {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(match self {
            Self::Io(_) => "nyar::clr::pe::io",
            Self::MethodBody(_) => "nyar::clr::pe::method_body",
            Self::MissingLocalMethodToken(_) => "nyar::clr::pe::missing_local_method_token",
            Self::MissingExternalMethodOwner(_) => "nyar::clr::pe::missing_external_method_owner",
            Self::ExternalTypeOwnerMissingBracket(_) => "nyar::clr::pe::external_type_owner_missing_bracket",
            Self::InvalidExternalTypeOwner(_) => "nyar::clr::pe::invalid_external_type_owner",
            Self::UnsupportedExternalTypeOwner(_) => "nyar::clr::pe::unsupported_external_type_owner",
            Self::MetadataIndexOverflow { .. } => "nyar::clr::pe::metadata_index_overflow",
            Self::NoEntryPoint => "nyar::clr::pe::no_entry_point",
            Self::NoMethodBody => "nyar::clr::pe::no_method_body",
        }))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(match self {
            Self::Io(_) => "请检查输出目录是否存在，以及当前进程是否具备写入权限",
            Self::MethodBody(_) => "请检查 `MSIL` 指令与操作数是否匹配",
            Self::MissingLocalMethodToken(_) => "请确认被引用的方法已经在当前模块中完成 token 分配",
            Self::MissingExternalMethodOwner(_) => "请为外部方法补充显式 owner，例如 `[System.Console]System.Console`",
            Self::ExternalTypeOwnerMissingBracket(_) => "外部类型 owner 应使用 `[程序集]命名空间.类型` 形式",
            Self::InvalidExternalTypeOwner(_) => "请检查外部类型 owner 中的程序集名和类型名是否都非空",
            Self::UnsupportedExternalTypeOwner(_) => "当前仅支持带显式程序集前缀的外部类型 owner",
            Self::MetadataIndexOverflow { .. } => "请检查字符串堆、blob 堆或元数据表项数量是否超过当前编码上限",
            Self::NoEntryPoint => "可执行 `CLR` 镜像必须包含唯一入口点",
            Self::NoMethodBody => "请确认模块至少生成了一个可编码的方法体",
        }))
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            Self::MethodBody(error) => Some(error),
            Self::Io(_)
            | Self::MissingLocalMethodToken(_)
            | Self::MissingExternalMethodOwner(_)
            | Self::ExternalTypeOwnerMissingBracket(_)
            | Self::InvalidExternalTypeOwner(_)
            | Self::UnsupportedExternalTypeOwner(_)
            | Self::MetadataIndexOverflow { .. }
            | Self::NoEntryPoint
            | Self::NoMethodBody => None,
        }
    }
}

impl From<std::io::Error> for PeWriterError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// `PE` 写入器。
pub struct PeWriter {
    options: PeWriterOptions,
}

impl PeWriter {
    /// 创建一个新的 `PE` 写入器。
    pub fn new(options: PeWriterOptions) -> Self {
        Self { options }
    }

    /// 将 `MsilModule` 写入为 `PE` 二进制。
    pub fn write_module(&self, module: &MsilModule) -> Result<Vec<u8>, PeWriterError> {
        // 查找入口方法。
        let entry_method = module
            .global_methods
            .iter()
            .find(|m| m.is_entry_point)
            .or_else(|| module.types.iter().flat_map(|type_def| type_def.methods.iter()).find(|m| m.is_entry_point));
        if matches!(self.options.image_kind, ClrImageKind::Executable) && entry_method.is_none() {
            return Err(PeWriterError::NoEntryPoint);
        }

        let local_method_tokens = build_local_method_token_map(module);

        // 构建用户类型名 -> TypeDef token 映射。
        // TypeDef 行号：`<Module>` = 1，第一个用户类型 = 2，以此类推。
        let type_token_map: BTreeMap<String, u32> =
            module.types.iter().enumerate().map(|(index, type_def)| (type_def.full_name.clone(), 0x0200_0000 | (index as u32 + 2))).collect();

        // 构建元数据。
        let mut metadata = ClrMetadataBuilder::new();
        let assembly_name_offset = metadata.strings.add(&self.options.assembly_name);
        let module_name_offset = metadata.strings.add(&self.options.module_name);
        let module_type_name_offset = metadata.strings.add("<Module>");
        let _mvid_index = metadata.guid.add(build_mvid(&self.options.assembly_name, &self.options.module_name));

        let resolved = resolve_module_tokens(module, &mut metadata, &local_method_tokens)?;
        let resolved_module = resolved.module;

        // 先确定 .text 节内方法体起始 RVA，供后续表项引用。
        let cli_header_size = CLI_HEADER_SIZE;
        let text_section_rva = TEXT_SECTION_RVA;
        let method_bodies_rva = text_section_rva + cli_header_size;

        // 收集所有需要写入的方法体，同时记录方法名和签名到堆中。
        // 对于有局部变量的方法，构建 LocalVarSig blob 并分配 StandAloneSig token。
        let mut method_bodies: Vec<Vec<u8>> = Vec::new();
        let mut method_name_offsets: Vec<u32> = Vec::new();
        let mut method_sig_offsets: Vec<u32> = Vec::new();
        let mut method_flags: Vec<u16> = Vec::new();
        let mut standalone_sig_blob_offsets: Vec<u32> = Vec::new();
        let mut next_sig_row: u32 = 1;

        for method in &resolved_module.global_methods {
            let local_var_sig_token = if method.locals.is_empty() {
                0
            }
            else {
                let blob = build_local_var_sig(&method.locals, &type_token_map);
                let blob_offset = metadata.blob.add(&blob);
                standalone_sig_blob_offsets.push(blob_offset);
                let token = 0x1100_0000 | next_sig_row;
                next_sig_row += 1;
                token
            };
            method_bodies.push(MethodBodyEncoder::encode(method, local_var_sig_token).map_err(PeWriterError::MethodBody)?);
            method_name_offsets.push(metadata.strings.add(&method.method.name));
            method_sig_offsets.push(metadata.blob.add(&build_method_signature(&method.method.signature, false, &type_token_map)));
            // 全局方法：Public | Static (0x0006 | 0x0010 = 0x0016)。
            method_flags.push(0x0016);
        }
        for type_def in &resolved_module.types {
            for method in &type_def.methods {
                let local_var_sig_token = if method.locals.is_empty() {
                    0
                }
                else {
                    let blob = build_local_var_sig(&method.locals, &type_token_map);
                    let blob_offset = metadata.blob.add(&blob);
                    standalone_sig_blob_offsets.push(blob_offset);
                    let token = 0x1100_0000 | next_sig_row;
                    next_sig_row += 1;
                    token
                };
                method_bodies.push(MethodBodyEncoder::encode(method, local_var_sig_token).map_err(PeWriterError::MethodBody)?);
                method_name_offsets.push(metadata.strings.add(&method.method.name));
                method_sig_offsets.push(metadata.blob.add(&build_method_signature(&method.method.signature, true, &type_token_map)));
                // 构造函数：Public | HideBySig | SpecialName | RTSpecialName (0x1886)。
                // 普通方法：Public | HideBySig (0x0086)。
                let flags = if method.method.name == ".ctor" { 0x1886 } else { 0x0086 };
                method_flags.push(flags);
            }
        }
        if method_bodies.is_empty() {
            return Err(PeWriterError::NoMethodBody);
        }

        // 计算各方法体的 RVA（相对于镜像基址）。
        let mut current_method_offset = 0u32;
        for body in &method_bodies {
            metadata.method_rvas.push(method_bodies_rva + current_method_offset);
            current_method_offset += align_up(body.len() as u32, 4);
        }

        // 计算各部分偏移。
        // .text 节布局：
        //   [CLI header (72 bytes)]
        //   [method bodies]
        //   [metadata]
        //   [import table]
        let method_bodies_size: u32 = method_bodies.iter().map(|b| align_up(b.len() as u32, 4)).sum();
        let method_bodies_aligned = align_up(method_bodies_size, 4);

        // 构建元数据二进制。
        // 收集用户类型定义和字段数据，供 TypeDef/Field 表生成。
        let (user_type_defs, field_rows) = self.build_type_and_field_info(&resolved_module, &mut metadata);
        let global_method_count = resolved_module.global_methods.len() as u32;

        let table_info = TableBuildInfo {
            assembly_name_offset,
            module_name_offset,
            module_type_name_offset,
            method_name_offsets: &method_name_offsets,
            method_sig_offsets: &method_sig_offsets,
            method_flags: &method_flags,
            global_method_count,
            user_type_defs: &user_type_defs,
            field_rows: &field_rows,
            assembly_ref_rows: &resolved.assembly_ref_rows,
            type_ref_rows: &resolved.type_ref_rows,
            member_ref_rows: &resolved.member_ref_rows,
            standalone_sig_blob_offsets: &standalone_sig_blob_offsets,
            system_object_type_ref_row: resolved.system_object_type_ref_row,
            system_value_type_type_ref_row: resolved.system_value_type_type_ref_row,
        };
        let metadata_bytes = self.build_metadata_bytes(&metadata, table_info);
        let metadata_size = metadata_bytes.len() as u32;
        let metadata_aligned = align_up(metadata_size, 4);
        let metadata_rva = method_bodies_rva + method_bodies_aligned;

        // 构建导入表（用于 _CorExeMain）。
        // 导入表位于 .text 节末尾，需先计算其 RVA 以填充内部 RVA 字段。
        let import_rva = metadata_rva + metadata_aligned;
        let import_layout = build_import_table(self.options.image_kind, import_rva);
        let import_bytes = import_layout.bytes;
        let import_size = import_bytes.len() as u32;
        let relocation_layout = build_relocation_section(import_layout.entry_stub_rva);
        let reloc_virtual_size = relocation_layout.bytes.len() as u32;

        // .text 节总大小。
        let text_section_virtual_size = cli_header_size + method_bodies_aligned + metadata_aligned + import_size;
        let text_section_raw_size = align_up(text_section_virtual_size, FILE_ALIGNMENT);
        let reloc_section_rva = align_up(text_section_rva + text_section_virtual_size, SECTION_ALIGNMENT);
        let reloc_section_raw_pointer = FILE_ALIGNMENT + text_section_raw_size;
        let reloc_section_raw_size = align_up(reloc_virtual_size, FILE_ALIGNMENT);
        let size_of_image = align_up(reloc_section_rva + reloc_virtual_size, SECTION_ALIGNMENT);

        let entry_token = if matches!(self.options.image_kind, ClrImageKind::Executable) {
            entry_method.and_then(|m| local_method_tokens.get(&m.method.name)).copied().unwrap_or(METHOD_DEF_TOKEN_BASE | 1)
        }
        else {
            0
        };
        let entry_rva = import_layout.entry_stub_rva;

        // 构建 PE 二进制。
        let mut writer = PeBinaryWriter::new();

        // 1. DOS 头（128 字节）。
        self.write_dos_header(&mut writer);

        // 2. PE 签名。
        writer.write_u32(0x00004550); // "PE\0\0"

        // 3. COFF 头（20 字节）。
        self.write_coff_header(&mut writer, text_section_raw_size);

        // 4. 可选头（PE32，224 字节含数据目录）。
        self.write_optional_header(
            &mut writer,
            text_section_rva,
            text_section_virtual_size,
            text_section_raw_size,
            size_of_image,
            metadata_rva,
            metadata_size,
            entry_rva,
            import_layout.import_directory_rva,
            import_layout.import_directory_size,
            import_layout.iat_rva,
            reloc_section_rva,
            reloc_virtual_size,
        );

        // 5. 节头。
        self.write_text_section_header(&mut writer, text_section_rva, text_section_virtual_size, text_section_raw_size);
        self.write_reloc_section_header(&mut writer, reloc_section_rva, reloc_virtual_size, reloc_section_raw_size, reloc_section_raw_pointer);

        // 5.1 头部填充：将文件偏移对齐到 file_alignment (0x200)，
        // 使得 .text 节内容起始偏移与节头中声明的 pointer_to_raw_data 一致。
        let headers_written = writer.len() as u32;
        let headers_padded = align_up(headers_written, FILE_ALIGNMENT);
        let headers_padding = headers_padded - headers_written;
        for _ in 0..headers_padding {
            writer.write_u8(0);
        }

        // 6. .text 节内容。
        // CLI 头。
        self.write_cli_header(&mut writer, metadata_rva, metadata_size, entry_token);

        // 方法体。
        for body in &method_bodies {
            writer.write_bytes(body);
            let body_padding = align_up(body.len() as u32, 4) - body.len() as u32;
            for _ in 0..body_padding {
                writer.write_u8(0);
            }
        }

        // 元数据。
        writer.write_bytes(&metadata_bytes);
        let metadata_padding = metadata_aligned - metadata_size;
        for _ in 0..metadata_padding {
            writer.write_u8(0);
        }

        // 导入表。
        writer.write_bytes(&import_bytes);

        // 填充 .text 节到对齐大小。
        let text_written = text_section_virtual_size;
        let text_padding = text_section_raw_size - text_written;
        for _ in 0..text_padding {
            writer.write_u8(0);
        }

        // 7. .reloc 节内容。
        writer.write_bytes(&relocation_layout.bytes);
        let reloc_padding = reloc_section_raw_size - reloc_virtual_size;
        for _ in 0..reloc_padding {
            writer.write_u8(0);
        }

        Ok(writer.into_bytes())
    }

    /// 写入 `DOS` 头。
    fn write_dos_header(&self, writer: &mut PeBinaryWriter) {
        // MZ 魔数。
        writer.write_u16(0x5A4D);
        // DOS 头剩余字段（58 字节，29 个 u16）。
        for _ in 0..29 {
            writer.write_u16(0);
        }
        // PE 头偏移：0x40（紧接 DOS 头之后，无 DOS stub）。
        writer.write_u32(0x40);
    }

    /// 写入 `COFF` 头。
    fn write_coff_header(&self, writer: &mut PeBinaryWriter, _text_size: u32) {
        // 机器类型：I386 (0x14C)。
        // 注意：不使用 CoffMachine::I386 as u16，因为枚举 discriminant 不符合 PE 规范。
        writer.write_u16(0x014C);
        // 节数量：2（`.text` + `.reloc`）。
        writer.write_u16(2);
        // 时间戳。
        writer.write_u32(0);
        // 符号表偏移：0（无符号表）。
        writer.write_u32(0);
        // 符号数量：0。
        writer.write_u32(0);
        // 可选头大小：224（PE32）。
        writer.write_u16(224);
        // 特征位：EXECUTABLE_IMAGE | 32BIT_MACHINE，动态库追加 DLL 标记。
        let mut characteristics = 0x0102u16;
        if matches!(self.options.image_kind, ClrImageKind::DynamicLibrary) {
            characteristics |= 0x2000;
        }
        writer.write_u16(characteristics);
    }

    /// 写入可选头。
    fn write_optional_header(
        &self,
        writer: &mut PeBinaryWriter,
        text_section_rva: u32,
        text_section_virtual_size: u32,
        text_section_raw_size: u32,
        size_of_image: u32,
        metadata_rva: u32,
        metadata_size: u32,
        entry_rva: u32,
        import_rva: u32,
        import_size: u32,
        iat_rva: u32,
        reloc_rva: u32,
        reloc_size: u32,
    ) {
        // 魔数：PE32 (0x10B)。
        writer.write_u16(0x10B);
        // 主链接器版本。
        writer.write_u8(11);
        // 次链接器版本。
        writer.write_u8(0);
        // 代码大小。
        writer.write_u32(text_section_raw_size);
        // 初始化数据大小。
        writer.write_u32(0);
        // 未初始化数据大小。
        writer.write_u32(0);
        // 入口点 RVA。
        writer.write_u32(entry_rva);
        // 代码基址 RVA。
        writer.write_u32(text_section_rva);
        // 数据基址 RVA。
        writer.write_u32(text_section_rva);
        // 镜像基址。
        writer.write_u32(0x00400000);
        // 节对齐。
        writer.write_u32(0x2000);
        // 文件对齐。
        writer.write_u32(0x0200);
        // 主 OS 版本。
        writer.write_u16(4);
        // 次 OS 版本。
        writer.write_u16(0);
        // 镜像主版本。
        writer.write_u16(0);
        // 镜像次版本。
        writer.write_u16(0);
        // 子系统主版本。
        writer.write_u16(4);
        // 子系统次版本。
        writer.write_u16(0);
        // 保留。
        writer.write_u32(0);
        // 镜像大小。
        writer.write_u32(size_of_image);
        // 头大小。
        writer.write_u32(0x0200);
        // 校验和：0。
        writer.write_u32(0);
        // 子系统：CONSOLE (3)。
        writer.write_u16(3);
        // DLL 特征：0。
        writer.write_u16(0);
        // 栈保留大小。
        writer.write_u32(0x100000);
        // 栈提交大小。
        writer.write_u32(0x1000);
        // 堆保留大小。
        writer.write_u32(0x100000);
        // 堆提交大小。
        writer.write_u32(0x1000);
        // 加载器标志：0。
        writer.write_u32(0);
        // 数据目录数量：16。
        writer.write_u32(16);

        // 数据目录（16 个，每个 8 字节）。
        // 0: Export。
        writer.write_u32(0);
        writer.write_u32(0);
        // 1: Import。
        writer.write_u32(import_rva);
        writer.write_u32(import_size);
        // 2: Resource。
        writer.write_u32(0);
        writer.write_u32(0);
        // 3: Exception。
        writer.write_u32(0);
        writer.write_u32(0);
        // 4: Certificate。
        writer.write_u32(0);
        writer.write_u32(0);
        // 5: Base Relocation。
        writer.write_u32(reloc_rva);
        writer.write_u32(reloc_size);
        // 6: Debug。
        writer.write_u32(0);
        writer.write_u32(0);
        // 7: Architecture。
        writer.write_u32(0);
        writer.write_u32(0);
        // 8: Global Ptr。
        writer.write_u32(0);
        writer.write_u32(0);
        // 9: TLS。
        writer.write_u32(0);
        writer.write_u32(0);
        // 10: Load Config。
        writer.write_u32(0);
        writer.write_u32(0);
        // 11: Bound Import。
        writer.write_u32(0);
        writer.write_u32(0);
        // 12: IAT。
        writer.write_u32(iat_rva);
        writer.write_u32(8);
        // 13: Delay Import。
        writer.write_u32(0);
        writer.write_u32(0);
        // 14: CLR Runtime Header（CLI 目录）。
        writer.write_u32(text_section_rva);
        writer.write_u32(72);
        // 15: Reserved。
        writer.write_u32(0);
        writer.write_u32(0);

        // 避免未使用参数警告。
        let _ = (metadata_rva, metadata_size, text_section_virtual_size);
    }

    /// 写入 `.text` 节头。
    fn write_text_section_header(
        &self,
        writer: &mut PeBinaryWriter,
        text_section_rva: u32,
        text_section_virtual_size: u32,
        text_section_raw_size: u32,
    ) {
        // 节名：.text（8 字节）。
        writer.write_bytes(b".text\0\0\0");
        // 虚拟大小。
        writer.write_u32(text_section_virtual_size);
        // 虚拟地址。
        writer.write_u32(text_section_rva);
        // 原始数据大小。
        writer.write_u32(text_section_raw_size);
        // 原始数据偏移：0x200（文件对齐后）。
        writer.write_u32(0x0200);
        // 重定位偏移：0。
        writer.write_u32(0);
        // 行号偏移：0。
        writer.write_u32(0);
        // 重定位数量：0。
        writer.write_u16(0);
        // 行号数量：0。
        writer.write_u16(0);
        // 特征位：CNT_CODE | MEM_EXECUTE | MEM_READ。
        writer.write_u32(0x60000020);
    }

    /// 写入 `.reloc` 节头。
    fn write_reloc_section_header(
        &self,
        writer: &mut PeBinaryWriter,
        reloc_section_rva: u32,
        reloc_virtual_size: u32,
        reloc_raw_size: u32,
        reloc_raw_pointer: u32,
    ) {
        writer.write_bytes(b".reloc\0\0");
        writer.write_u32(reloc_virtual_size);
        writer.write_u32(reloc_section_rva);
        writer.write_u32(reloc_raw_size);
        writer.write_u32(reloc_raw_pointer);
        writer.write_u32(0);
        writer.write_u32(0);
        writer.write_u16(0);
        writer.write_u16(0);
        writer.write_u32(0x42000040);
    }

    /// 写入 `CLI` 头。
    fn write_cli_header(&self, writer: &mut PeBinaryWriter, metadata_rva: u32, metadata_size: u32, entry_token: u32) {
        // CLI 头大小：72 字节。
        writer.write_u32(72);
        // 主运行时版本。
        writer.write_u16(2);
        // 次运行时版本。
        writer.write_u16(5);
        // 元数据 RVA。
        writer.write_u32(metadata_rva);
        // 元数据大小。
        writer.write_u32(metadata_size);
        // COM Image Flags：IL_ONLY。
        writer.write_u32(0x01);
        // 入口点 token：可执行文件指向 `MethodDef`，动态库为 0。
        writer.write_u32(entry_token);
        // 资源数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
        // 强名称数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
        // 代码管理器数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
        // VTable 数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
        // 导出地址数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
        // 导入调试数据目录。
        writer.write_u32(0);
        writer.write_u32(0);
    }

    /// 构建元数据二进制。
    fn build_metadata_bytes(&self, metadata: &ClrMetadataBuilder, table_info: TableBuildInfo) -> Vec<u8> {
        let mut bytes = Vec::new();

        // 元数据根签名：0x424A5342 ("BSJB")。
        bytes.extend_from_slice(&0x424A5342u32.to_le_bytes());
        // 主版本：1。
        bytes.extend_from_slice(&1u16.to_le_bytes());
        // 次版本：1。
        bytes.extend_from_slice(&1u16.to_le_bytes());
        // 保留：0。
        bytes.extend_from_slice(&0u32.to_le_bytes());
        // 版本字符串长度（4 字节对齐）。
        let version = "v4.0.30319";
        let version_len = version.len() as u32;
        let version_padded = align_up(version_len, 4);
        bytes.extend_from_slice(&version_padded.to_le_bytes());
        // 版本字符串。
        bytes.extend_from_slice(version.as_bytes());
        // 版本字符串填充。
        let version_padding = version_padded - version_len;
        for _ in 0..version_padding {
            bytes.push(0);
        }

        // 准备流数据。
        let strings_data = metadata.strings.data().to_vec();
        let us_data = metadata.user_strings.data().to_vec();
        let guid_data = metadata.guid.data().to_vec();
        let blob_data = metadata.blob.data().to_vec();
        // #~ 流：包含 Module、MethodDef、Assembly 三张表。
        let tables_data = build_tables_stream(metadata, &table_info);

        // 流定义：名称 + 数据。
        let streams: Vec<(&str, &[u8])> =
            vec![("#Strings", &strings_data), ("#US", &us_data), ("#GUID", &guid_data), ("#Blob", &blob_data), ("#~", &tables_data)];

        // 计算元数据根头大小（BSJB 签名到流头之前）。
        let metadata_root_header_size = 4 + 2 + 2 + 4 + 4 + version_padded + 2 + 2;

        // 第一遍：计算流头总大小，以确定流数据起始偏移。
        let mut stream_headers_size = 0u32;
        for (name, _data) in &streams {
            stream_headers_size += 8 + align_name_size(name.len() as u32 + 1);
        }

        // 流数据起始偏移（相对于元数据根起始）。
        let stream_data_base = metadata_root_header_size + stream_headers_size;

        // 第二遍：计算每个流的数据偏移（相对于元数据根起始）。
        let mut current_data_offset = stream_data_base;
        let mut stream_entries: Vec<(u32, u32, &str, &[u8])> = Vec::new();
        for (name, data) in &streams {
            let size = data.len() as u32;
            stream_entries.push((current_data_offset, size, *name, *data));
            current_data_offset += align_up(size, 4);
        }

        // Flags：0。
        bytes.extend_from_slice(&0u16.to_le_bytes());
        // 流数量。
        bytes.extend_from_slice(&(streams.len() as u16).to_le_bytes());

        // 第三遍：写入流头。
        for (offset, size, name, _data) in &stream_entries {
            bytes.extend_from_slice(&offset.to_le_bytes());
            bytes.extend_from_slice(&size.to_le_bytes());
            // 名称：null-terminated + 4 字节对齐填充。
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
            let name_padded = align_name_size(name.len() as u32 + 1);
            let padding = name_padded - (name.len() as u32 + 1);
            for _ in 0..padding {
                bytes.push(0);
            }
        }

        // 第四遍：写入流数据（4 字节对齐）。
        for (_offset, _size, _name, data) in &stream_entries {
            bytes.extend_from_slice(data);
            let padding = align_up(data.len() as u32, 4) - data.len() as u32;
            for _ in 0..padding {
                bytes.push(0);
            }
        }

        bytes
    }
}

/// `PE` 二进制写入器辅助。
struct PeBinaryWriter {
    data: Vec<u8>,
}

impl PeBinaryWriter {
    /// 创建新的写入器。
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// 返回已写入字节数。
    fn len(&self) -> usize {
        self.data.len()
    }

    /// 写入字节。
    fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    /// 写入 `u8`。
    fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }

    /// 写入 `u16`（小端）。
    fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// 写入 `u32`（小端）。
    fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    /// 转为字节数组。
    fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

impl PeWriter {
    /// 构建用户类型定义和字段表行信息。
    ///
    /// 遍历 `module.types`，将类型名、字段名、字段签名添加到元数据堆中，
    /// 并构建 `UserTypeDefRow` 和 `FieldRow` 列表。
    ///
    /// `FieldList` 按 1-indexed 行号指向 `Field` 表中该类型的第一个字段。
    /// `MethodList` 按 1-indexed 行号指向 `MethodDef` 表中该类型的第一个方法
    /// （全局方法数量 + 1 起算）。
    fn build_type_and_field_info(&self, module: &MsilModule, metadata: &mut ClrMetadataBuilder) -> (Vec<UserTypeDefRow>, Vec<FieldRow>) {
        let mut user_type_defs = Vec::new();
        let mut field_rows = Vec::new();
        let mut type_token_map = BTreeMap::new();
        let mut field_row_index = 1u16;
        let mut method_row_index = (module.global_methods.len() as u16) + 1;

        // TypeDef 表行 1 是 `<Module>`，用户类型从行 2 开始。
        // TypeDef token = 0x02000000 | row。
        for (type_index, type_def) in module.types.iter().enumerate() {
            // TypeDef 表的 TypeName 和 TypeNamespace 分别存储简单名和命名空间。
            let type_name_offset = metadata.strings.add(&type_def.full_name) as u16;
            let type_namespace_offset = metadata.strings.add(&type_def.namespace) as u16;
            let field_list = field_row_index;
            let method_list = method_row_index;

            // 记录限定名到 TypeDef token 的映射，供签名编码使用。
            // 使用 qualified_name() 确保不同命名空间下的同名类型能分别解析。
            let type_def_row = (type_index + 2) as u32;
            let type_def_token = 0x0200_0000 | type_def_row;
            type_token_map.insert(type_def.qualified_name(), type_def_token);

            for field in &type_def.fields {
                let name_offset = metadata.strings.add(&field.name) as u16;
                let sig_offset = metadata.blob.add(&build_field_signature(&field.ty, &type_token_map)) as u16;
                // FieldAttributes: Public (0x06) | Instance (0x00)
                field_rows.push(FieldRow { flags: 0x0006, name_offset, signature_offset: sig_offset });
                field_row_index += 1;
            }

            method_row_index += type_def.methods.len() as u16;

            user_type_defs.push(UserTypeDefRow {
                type_name_offset,
                type_namespace_offset,
                field_list,
                method_list,
                is_value_type: type_def.is_value_type,
            });
        }

        (user_type_defs, field_rows)
    }
}

/// 对齐到指定边界。
fn align_up(value: u32, alignment: u32) -> u32 {
    if alignment == 0 {
        return value;
    }
    let remainder = value % alignment;
    if remainder == 0 {
        value
    }
    else {
        value + (alignment - remainder)
    }
}

/// 计算流名称占用空间（含 null 终止符，4 字节对齐）。
fn align_name_size(name_len_with_null: u32) -> u32 {
    align_up(name_len_with_null, 4)
}

fn build_local_method_token_map(module: &MsilModule) -> BTreeMap<String, u32> {
    let mut tokens = BTreeMap::new();
    let mut row = 1u32;

    for method in &module.global_methods {
        tokens.insert(method.method.name.clone(), METHOD_DEF_TOKEN_BASE | row);
        row += 1;
    }

    for type_def in &module.types {
        for method in &type_def.methods {
            // 使用命名空间限定名 `ns.TypeName.method_name` 避免不同命名空间下的同名类型方法冲突。
            // 与 lowering 阶段构造函数的 owner（限定名）保持一致。
            let qualified_name = format!("{}.{}", type_def.qualified_name(), method.method.name);
            tokens.insert(qualified_name, METHOD_DEF_TOKEN_BASE | row);
            row += 1;
        }
    }

    tokens
}
