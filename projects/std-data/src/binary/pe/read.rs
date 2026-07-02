//! `PE/COFF` 与 `CLR` 元数据解析。
//!
//! 本模块提供两套解析入口：
//! - [`parse_pe64`]：轻量级 `PE32+` 头摘要，仅读取 `DOS`/`COFF`/可选头魔数，供宿主可执行格式探测使用。
//! - [`parse_pe`]：完整 `PE` 解析入口，返回包含节区、`CLI` 头与元数据根的 [`PeImage`]。
//!
//! 同时提供 `RVA` 到文件偏移的换算、`#Strings` / `#Blob` / `#US` 堆读取等基础能力。

use std::fmt;

/// `PE32+` 可执行文件解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pe64ParseError {
    /// 镜像过短。
    TooShort,
    /// `DOS` 头 `MZ` 魔数错误。
    BadDosMagic,
    /// `e_lfanew` 指向的 `PE` 签名缺失或无效。
    BadPeSignature,
    /// `COFF` 机器类型。
    Machine(u16),
    /// 可选头魔数。
    OptionalMagic(u16),
}

impl fmt::Display for Pe64ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "PE 镜像过短"),
            Self::BadDosMagic => write!(f, "PE 镜像缺少 DOS `MZ` 魔数"),
            Self::BadPeSignature => write!(f, "PE 镜像缺少 `PE\\0\\0` 签名"),
            Self::Machine(machine) => write!(f, "PE 镜像机器类型 0x{machine:04X}"),
            Self::OptionalMagic(magic) => write!(f, "PE 镜像可选头魔数 0x{magic:04X}"),
        }
    }
}

impl std::error::Error for Pe64ParseError {}

/// 解析后的 `PE32+` 头摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pe64Header {
    /// `PE` 头在文件中的偏移。
    pub pe_offset: usize,
    /// `COFF` 机器类型。
    pub machine: u16,
    /// 可选头魔数。
    pub optional_magic: u16,
}

const DOS_HEADER_SIZE: usize = 64;
const PE_SIGNATURE: &[u8; 4] = b"PE\0\0";

/// 解析 `PE32+` 可执行文件头。
///
/// 该函数仅做最小限度校验，用于宿主可执行格式探测，不解析节区与元数据。
pub fn parse_pe64(bytes: &[u8]) -> Result<Pe64Header, Pe64ParseError> {
    if bytes.len() < DOS_HEADER_SIZE {
        return Err(Pe64ParseError::TooShort);
    }
    if &bytes[0..2] != b"MZ" {
        return Err(Pe64ParseError::BadDosMagic);
    }

    let pe_offset = u32::from_le_bytes(bytes[0x3C..0x40].try_into().expect("slice")) as usize;
    if bytes.len() < pe_offset + 4 + 20 + 2 {
        return Err(Pe64ParseError::TooShort);
    }
    if bytes.get(pe_offset..pe_offset + 4) != Some(PE_SIGNATURE) {
        return Err(Pe64ParseError::BadPeSignature);
    }

    let machine = u16::from_le_bytes(bytes[pe_offset + 4..pe_offset + 6].try_into().expect("slice"));
    let optional_magic = u16::from_le_bytes(bytes[pe_offset + 24..pe_offset + 26].try_into().expect("slice"));
    Ok(Pe64Header { pe_offset, machine, optional_magic })
}

/// 从 `PE` 镜像中提取指定节内容。
///
/// 按节名（最长 8 字节，不足补 `0`）匹配，返回该节的原始数据副本。
pub fn extract_pe_section(pe: &[u8], section_name: &[u8]) -> Option<Vec<u8>> {
    let header = parse_pe64(pe).ok()?;
    let pe_offset = header.pe_offset;
    let section_count = u16::from_le_bytes(pe[pe_offset + 6..pe_offset + 8].try_into().ok()?) as usize;
    let optional_header_size = u16::from_le_bytes(pe[pe_offset + 20..pe_offset + 22].try_into().ok()?) as usize;
    let section_table = pe_offset + 24 + optional_header_size;
    for index in 0..section_count {
        let entry = section_table + index * 40;
        if pe.len() < entry + 40 {
            return None;
        }
        if &pe[entry..entry + 8][..section_name.len()] != section_name {
            continue;
        }
        let raw_size = u32::from_le_bytes(pe[entry + 16..entry + 20].try_into().ok()?) as usize;
        let raw_ptr = u32::from_le_bytes(pe[entry + 20..entry + 24].try_into().ok()?) as usize;
        if pe.len() < raw_ptr + raw_size {
            return None;
        }
        return Some(pe[raw_ptr..raw_ptr + raw_size].to_vec());
    }
    None
}

// ============================================================================
// 完整 PE / CLR 解析模型
// ============================================================================

/// `PE` / `CLR` 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeParseError {
    /// 文件过短。
    TooShort,
    /// `MZ` 魔数错误。
    BadMzMagic,
    /// `PE` 签名错误。
    BadPeSignature,
    /// 不是 PE32 或 PE32+。
    NotPe32,
    /// 不是 `CLR` 镜像（无 `CLI` 目录）。
    NotClrImage,
    /// `CLI` 头魔数错误。
    BadCliHeader,
    /// 元数据根签名错误（非 `BSJB`）。
    BadMetadataSignature,
    /// 流头越界。
    StreamHeaderOutOfBounds,
    /// 流数据越界。
    StreamDataOutOfBounds,
    /// 表行越界。
    TableRowOutOfBounds,
    /// 不支持的表种类。
    UnsupportedTable(u8),
}

impl fmt::Display for PeParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "文件过短"),
            Self::BadMzMagic => write!(f, "MZ 魔数错误"),
            Self::BadPeSignature => write!(f, "PE 签名错误"),
            Self::NotPe32 => write!(f, "不是 PE32 或 PE32+"),
            Self::NotClrImage => write!(f, "不是 CLR 镜像（无 CLI 目录）"),
            Self::BadCliHeader => write!(f, "CLI 头魔数错误"),
            Self::BadMetadataSignature => write!(f, "元数据根签名错误（非 BSJB）"),
            Self::StreamHeaderOutOfBounds => write!(f, "流头越界"),
            Self::StreamDataOutOfBounds => write!(f, "流数据越界"),
            Self::TableRowOutOfBounds => write!(f, "表行越界"),
            Self::UnsupportedTable(id) => write!(f, "不支持的表种类 0x{:02X}", id),
        }
    }
}

impl std::error::Error for PeParseError {}

/// `PE` 解析结果。
#[derive(Debug, Clone)]
pub struct PeImage {
    /// `DOS` 头。
    pub dos: DosHeader,
    /// `COFF` 头。
    pub coff: CoffHeader,
    /// 可选头。
    pub optional: OptionalHeader,
    /// 节头列表。
    pub sections: Vec<SectionHeader>,
    /// `CLI` 头（若存在）。
    pub cli: Option<CliHeader>,
    /// 元数据（若存在）。
    pub metadata: Option<MetadataRoot>,
}

/// `DOS` 头（只关心 `MZ` 魔数与 `PE` 偏移）。
#[derive(Debug, Clone, Copy)]
pub struct DosHeader {
    /// `PE` 头偏移（位于 `0x3C`）。
    pub pe_offset: u32,
}

/// `COFF` 头。
#[derive(Debug, Clone, Copy)]
pub struct CoffHeader {
    /// 机器类型。
    pub machine: u16,
    /// 节数量。
    pub number_of_sections: u16,
    /// 时间戳。
    pub time_date_stamp: u32,
    /// 符号表偏移。
    pub pointer_to_symbol_table: u32,
    /// 符号数量。
    pub number_of_symbols: u32,
    /// 可选头大小。
    pub size_of_optional_header: u16,
    /// 特征位。
    pub characteristics: u16,
}

/// 可选头（PE32 或 PE32+）。
#[derive(Debug, Clone, Copy)]
pub struct OptionalHeader {
    /// 魔数（`0x10B` = PE32, `0x20B` = PE32+）。
    pub magic: u16,
    /// 主链接器版本。
    pub major_linker_version: u8,
    /// 次链接器版本。
    pub minor_linker_version: u8,
    /// 代码大小。
    pub size_of_code: u32,
    /// 初始化数据大小。
    pub size_of_initialized_data: u32,
    /// 未初始化数据大小。
    pub size_of_uninitialized_data: u32,
    /// 入口点 `RVA`。
    pub address_of_entry_point: u32,
    /// 代码基址 `RVA`。
    pub base_of_code: u32,
    /// 镜像基址（PE32+: u64，PE32: u32 扩展为 u64）。
    pub image_base: u64,
    /// 节对齐。
    pub section_alignment: u32,
    /// 文件对齐。
    pub file_alignment: u32,
    /// 镜像大小。
    pub size_of_image: u32,
    /// 头大小。
    pub size_of_headers: u32,
    /// 子系统。
    pub subsystem: u16,
    /// `DLL` 特征。
    pub dll_characteristics: u16,
    /// 数据目录（16 个）。
    pub data_directories: [DataDirectory; 16],
}

/// 数据目录。
#[derive(Debug, Clone, Copy, Default)]
pub struct DataDirectory {
    /// `RVA`。
    pub rva: u32,
    /// 大小。
    pub size: u32,
}

/// 节头。
#[derive(Debug, Clone)]
pub struct SectionHeader {
    /// 节名（最长 8 字节）。
    pub name: String,
    /// 虚拟大小。
    pub virtual_size: u32,
    /// 虚拟地址。
    pub virtual_address: u32,
    /// 原始数据大小。
    pub size_of_raw_data: u32,
    /// 原始数据偏移。
    pub pointer_to_raw_data: u32,
    /// 特征位。
    pub characteristics: u32,
}

/// `CLI` 头。
#[derive(Debug, Clone, Copy)]
pub struct CliHeader {
    /// 头大小。
    pub cb: u32,
    /// 主运行时版本。
    pub major_runtime_version: u16,
    /// 次运行时版本。
    pub minor_runtime_version: u16,
    /// 元数据 `RVA`。
    pub metadata_rva: u32,
    /// 元数据大小。
    pub metadata_size: u32,
    /// `COM Image Flags`。
    pub flags: u32,
    /// 入口点 `token`。
    pub entry_point_token: u32,
}

/// 元数据根。
#[derive(Debug, Clone)]
pub struct MetadataRoot {
    /// 主版本。
    pub major_version: u16,
    /// 次版本。
    pub minor_version: u16,
    /// 版本字符串。
    pub version: String,
    /// 流列表。
    pub streams: Vec<StreamHeader>,
    /// `#Strings` 数据。
    pub strings: Vec<u8>,
    /// `#US` 数据。
    pub user_strings: Vec<u8>,
    /// `#GUID` 数据。
    pub guid: Vec<u8>,
    /// `#Blob` 数据。
    pub blob: Vec<u8>,
    /// `#~` 数据（元数据表）。
    pub tables: Vec<u8>,
    /// 表行数（按表种类索引，0..64）。
    pub row_counts: [u32; 64],
    /// 表存在位图。
    pub valid_tables: u64,
    /// 已排序表位图。
    pub sorted_tables: u64,
}

/// 流头。
#[derive(Debug, Clone)]
pub struct StreamHeader {
    /// 偏移（相对于元数据根起始）。
    pub offset: u32,
    /// 大小。
    pub size: u32,
    /// 名称。
    pub name: String,
}

/// 元数据表种类。
///
/// 取解析侧与写入侧（`meta_builder`）两份定义的并集，
/// 同时覆盖写入侧需要的 `StandAloneSig` (0x11)。解析侧与写入侧共用此枚举，
/// 避免重复定义导致编号漂移。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// `Module`（0x00）。
    Module = 0x00,
    /// `TypeRef`（0x01）。
    TypeRef = 0x01,
    /// `TypeDef`（0x02）。
    TypeDef = 0x02,
    /// `Field`（0x04）。
    Field = 0x04,
    /// `MethodDef`（0x06）。
    MethodDef = 0x06,
    /// `Param`（0x08）。
    Param = 0x08,
    /// `MemberRef`（0x0A）。
    MemberRef = 0x0A,
    /// `StandAloneSig`（0x11），写入侧用于局部变量签名。
    StandAloneSig = 0x11,
    /// `TypeSpec`（0x1B），写入侧用于 `T[]` / 泛型实例等类型操作数。
    TypeSpec = 0x1B,
    /// `Assembly`（0x20）。
    Assembly = 0x20,
    /// `AssemblyRef`（0x23）。
    AssemblyRef = 0x23,
}

impl TableKind {
    /// 将原始表种类转为枚举，未支持的种类返回 `None`。
    pub fn from_raw(raw: u8) -> Option<Self> {
        match raw {
            0x00 => Some(Self::Module),
            0x01 => Some(Self::TypeRef),
            0x02 => Some(Self::TypeDef),
            0x04 => Some(Self::Field),
            0x06 => Some(Self::MethodDef),
            0x08 => Some(Self::Param),
            0x0A => Some(Self::MemberRef),
            0x11 => Some(Self::StandAloneSig),
            0x1B => Some(Self::TypeSpec),
            0x20 => Some(Self::Assembly),
            0x23 => Some(Self::AssemblyRef),
            _ => None,
        }
    }
}

/// 解析 `PE` 二进制，返回完整的 [`PeImage`]。
///
/// 依次解析 `DOS` 头、`PE` 签名、`COFF` 头、可选头、节头，并在存在 `CLI` 目录时
/// 进一步解析 `CLI` 头与元数据根。该入口是 std-data 侧 `PE/CLR` 解析的唯一完整入口。
pub fn parse_pe(data: &[u8]) -> Result<PeImage, PeParseError> {
    if data.len() < 64 {
        return Err(PeParseError::TooShort);
    }

    // 1. DOS 头。
    if &data[0..2] != b"MZ" {
        return Err(PeParseError::BadMzMagic);
    }
    let pe_offset = read_u32(data, 0x3C)?;
    let dos = DosHeader { pe_offset };

    // 2. PE 签名。
    let pe_sig_offset = pe_offset as usize;
    if data.len() < pe_sig_offset + 4 {
        return Err(PeParseError::TooShort);
    }
    if &data[pe_sig_offset..pe_sig_offset + 4] != b"PE\0\0" {
        return Err(PeParseError::BadPeSignature);
    }

    // 3. COFF 头（20 字节）。
    let coff_offset = pe_sig_offset + 4;
    if data.len() < coff_offset + 20 {
        return Err(PeParseError::TooShort);
    }
    let coff = CoffHeader {
        machine: read_u16(data, coff_offset)?,
        number_of_sections: read_u16(data, coff_offset + 2)?,
        time_date_stamp: read_u32(data, coff_offset + 4)?,
        pointer_to_symbol_table: read_u32(data, coff_offset + 8)?,
        number_of_symbols: read_u32(data, coff_offset + 12)?,
        size_of_optional_header: read_u16(data, coff_offset + 16)?,
        characteristics: read_u16(data, coff_offset + 18)?,
    };

    // 4. 可选头。
    let opt_offset = coff_offset + 20;
    let optional = parse_optional_header(data, opt_offset)?;

    // 5. 节头。
    let sections_offset = opt_offset + coff.size_of_optional_header as usize;
    let mut sections = Vec::with_capacity(coff.number_of_sections as usize);
    for i in 0..coff.number_of_sections as usize {
        let sec_offset = sections_offset + i * 40;
        if data.len() < sec_offset + 40 {
            return Err(PeParseError::TooShort);
        }
        let name_bytes = &data[sec_offset..sec_offset + 8];
        let name_end = name_bytes.iter().position(|&b| b == 0).unwrap_or(8);
        let name = String::from_utf8_lossy(&name_bytes[..name_end]).into_owned();
        sections.push(SectionHeader {
            name,
            virtual_size: read_u32(data, sec_offset + 8)?,
            virtual_address: read_u32(data, sec_offset + 12)?,
            size_of_raw_data: read_u32(data, sec_offset + 16)?,
            pointer_to_raw_data: read_u32(data, sec_offset + 20)?,
            characteristics: read_u32(data, sec_offset + 36)?,
        });
    }

    // 6. CLI 头（数据目录 14）。
    let cli_dir = optional.data_directories[14];
    let mut cli = None;
    let mut metadata = None;
    if cli_dir.rva != 0 && cli_dir.size != 0 {
        let cli_offset = rva_to_offset(&sections, cli_dir.rva)? as usize;
        if data.len() < cli_offset + 72 {
            return Err(PeParseError::TooShort);
        }
        let cli_header = CliHeader {
            cb: read_u32(data, cli_offset)?,
            major_runtime_version: read_u16(data, cli_offset + 4)?,
            minor_runtime_version: read_u16(data, cli_offset + 6)?,
            metadata_rva: read_u32(data, cli_offset + 8)?,
            metadata_size: read_u32(data, cli_offset + 12)?,
            flags: read_u32(data, cli_offset + 16)?,
            entry_point_token: read_u32(data, cli_offset + 20)?,
        };
        cli = Some(cli_header);

        // 7. 元数据根。
        let md_offset = rva_to_offset(&sections, cli_header.metadata_rva)? as usize;
        metadata = Some(parse_metadata(data, md_offset, cli_header.metadata_size as usize)?);
    }

    Ok(PeImage { dos, coff, optional, sections, cli, metadata })
}

/// 解析可选头（支持 PE32 和 PE32+）。
fn parse_optional_header(data: &[u8], offset: usize) -> Result<OptionalHeader, PeParseError> {
    if data.len() < offset + 2 {
        return Err(PeParseError::TooShort);
    }
    let magic = read_u16(data, offset)?;
    let is_pe32plus = magic == 0x20B;
    if magic != 0x10B && !is_pe32plus {
        return Err(PeParseError::NotPe32);
    }

    // PE32+ 可选头比 PE32 多 16 字节（image_base 是 8 字节，base_of_data 不存在）
    // PE32:  标准字段 28 字节 + Windows 特定字段 68 字节 + 数据目录 128 字节 = 224 字节
    // PE32+: 标准字段 24 字节 + Windows 特定字段 88 字节 + 数据目录 128 字节 = 240 字节
    let dd_offset = if is_pe32plus { offset + 112 } else { offset + 96 };
    let min_size = dd_offset + 128;
    if data.len() < min_size {
        return Err(PeParseError::TooShort);
    }

    let mut data_directories = [DataDirectory::default(); 16];
    for i in 0..16 {
        data_directories[i] = DataDirectory { rva: read_u32(data, dd_offset + i * 8)?, size: read_u32(data, dd_offset + i * 8 + 4)? };
    }

    // PE32:  base_of_data 在 offset+24 (u32，已忽略), image_base 在 offset+28 (u32，扩展为 u64)
    // PE32+: base_of_data 不存在, image_base 在 offset+24 (u64)
    let image_base: u64 = if is_pe32plus { read_u64(data, offset + 24)? } else { read_u32(data, offset + 28)? as u64 };

    Ok(OptionalHeader {
        magic,
        major_linker_version: data[offset + 2],
        minor_linker_version: data[offset + 3],
        size_of_code: read_u32(data, offset + 4)?,
        size_of_initialized_data: read_u32(data, offset + 8)?,
        size_of_uninitialized_data: read_u32(data, offset + 12)?,
        address_of_entry_point: read_u32(data, offset + 16)?,
        base_of_code: read_u32(data, offset + 20)?,
        image_base,
        section_alignment: read_u32(data, offset + 32)?,
        file_alignment: read_u32(data, offset + 36)?,
        size_of_image: read_u32(data, offset + 56)?,
        size_of_headers: read_u32(data, offset + 60)?,
        subsystem: read_u16(data, offset + 68)?,
        dll_characteristics: read_u16(data, offset + 70)?,
        data_directories,
    })
}

/// 解析元数据根。
fn parse_metadata(data: &[u8], offset: usize, _size: usize) -> Result<MetadataRoot, PeParseError> {
    if data.len() < offset + 16 {
        return Err(PeParseError::TooShort);
    }
    let signature = read_u32(data, offset)?;
    if signature != 0x424A5342 {
        return Err(PeParseError::BadMetadataSignature);
    }
    let major_version = read_u16(data, offset + 4)?;
    let minor_version = read_u16(data, offset + 6)?;
    let _reserved = read_u32(data, offset + 8)?;
    let version_len = read_u32(data, offset + 12)? as usize;
    if data.len() < offset + 16 + version_len {
        return Err(PeParseError::TooShort);
    }
    let version_bytes = &data[offset + 16..offset + 16 + version_len];
    let version_end = version_bytes.iter().position(|&b| b == 0).unwrap_or(version_len);
    let version = String::from_utf8_lossy(&version_bytes[..version_end]).into_owned();

    let streams_count_offset = offset + 16 + version_len;
    // ECMA-335 II.24.2.1: Flags(2) + Streams(2)
    let _flags = read_u16(data, streams_count_offset)?;
    let streams_count = read_u16(data, streams_count_offset + 2)? as usize;

    // 解析流头。
    let mut stream_cursor = streams_count_offset + 4;
    let mut streams = Vec::with_capacity(streams_count);
    for _ in 0..streams_count {
        if data.len() < stream_cursor + 8 {
            return Err(PeParseError::StreamHeaderOutOfBounds);
        }
        let s_offset = read_u32(data, stream_cursor)?;
        let s_size = read_u32(data, stream_cursor + 4)?;
        stream_cursor += 8;
        // 名称以 0 结尾，且整体 4 字节对齐。
        let mut name = String::new();
        while stream_cursor < data.len() && data[stream_cursor] != 0 {
            name.push(data[stream_cursor] as char);
            stream_cursor += 1;
        }
        // 跳过 0 终止符。
        if stream_cursor < data.len() {
            stream_cursor += 1;
        }
        // 对齐到 4 字节。
        while stream_cursor % 4 != 0 {
            stream_cursor += 1;
        }
        streams.push(StreamHeader { offset: s_offset, size: s_size, name });
    }

    // 提取各流数据。
    let mut strings = Vec::new();
    let mut user_strings = Vec::new();
    let mut guid = Vec::new();
    let mut blob = Vec::new();
    let mut tables = Vec::new();
    let mut row_counts = [0u32; 64];
    let mut valid_tables = 0u64;
    let mut sorted_tables = 0u64;

    for s in &streams {
        let s_data_start = offset + s.offset as usize;
        let s_data_end = s_data_start + s.size as usize;
        if data.len() < s_data_end {
            return Err(PeParseError::StreamDataOutOfBounds);
        }
        let s_data = &data[s_data_start..s_data_end];
        match s.name.as_str() {
            "#Strings" => strings = s_data.to_vec(),
            "#US" => user_strings = s_data.to_vec(),
            "#GUID" => guid = s_data.to_vec(),
            "#Blob" => blob = s_data.to_vec(),
            "#~" => {
                // 解析表流头。
                if s_data.len() < 24 {
                    return Err(PeParseError::TooShort);
                }
                valid_tables = read_u64(s_data, 8)?;
                sorted_tables = read_u64(s_data, 16)?;
                // 行数数组从偏移 24 开始。
                let mut rc = 24usize;
                for i in 0..64 {
                    if (valid_tables >> i) & 1 == 1 {
                        if rc + 4 > s_data.len() {
                            return Err(PeParseError::TooShort);
                        }
                        row_counts[i] = read_u32(s_data, rc)?;
                        rc += 4;
                    }
                }
                tables = s_data[rc..].to_vec();
            }
            _ => {}
        }
    }

    Ok(MetadataRoot {
        major_version,
        minor_version,
        version,
        streams,
        strings,
        user_strings,
        guid,
        blob,
        tables,
        row_counts,
        valid_tables,
        sorted_tables,
    })
}

/// 将 `RVA` 转为文件偏移。
///
/// 遍历节头，定位 `rva` 落入哪个节的虚拟地址范围，返回对应的原始数据偏移。
pub fn rva_to_offset(sections: &[SectionHeader], rva: u32) -> Result<u32, PeParseError> {
    for s in sections {
        if rva >= s.virtual_address && rva < s.virtual_address + s.virtual_size {
            return Ok(s.pointer_to_raw_data + (rva - s.virtual_address));
        }
    }
    Err(PeParseError::StreamDataOutOfBounds)
}

/// 从 `#Strings` 堆读取以 0 结尾的字符串。
///
/// `offset` 为堆内偏移；越界或读取失败返回空串。
pub fn read_strings_string(strings: &[u8], offset: u32) -> String {
    let offset = offset as usize;
    if offset >= strings.len() {
        return String::new();
    }
    let end = strings[offset..].iter().position(|&b| b == 0).map(|e| offset + e).unwrap_or(strings.len());
    String::from_utf8_lossy(&strings[offset..end]).into_owned()
}

/// 从 `#Blob` 堆读取 blob（压缩长度前缀）。
///
/// `offset` 为堆内偏移；返回的数据不含长度前缀。越界返回空 `Vec`。
pub fn read_blob(blob: &[u8], offset: u32) -> Vec<u8> {
    let offset = offset as usize;
    if offset >= blob.len() {
        return Vec::new();
    }
    let (len, len_bytes) = read_compressed_uint(blob, offset);
    let start = offset + len_bytes;
    let end = start + len as usize;
    if end > blob.len() {
        return Vec::new();
    }
    blob[start..end].to_vec()
}

/// 从 `#US` 堆读取用户字符串。
///
/// `offset` 为堆内偏移。用户字符串以压缩长度前缀编码，内容为 `UTF-16LE`，
/// 末尾一字节为标志位（跳过）。越界或长度为 0 返回空串。
pub fn read_user_string(us: &[u8], offset: u32) -> String {
    let offset = offset as usize;
    if offset >= us.len() {
        return String::new();
    }
    let (len, len_bytes) = read_compressed_uint(us, offset);
    let start = offset + len_bytes;
    let end = start + len as usize;
    if end > us.len() || len == 0 {
        return String::new();
    }
    // 最后一个字节是标志位，跳过。
    let utf16_end = end.saturating_sub(1);
    let bytes = &us[start..utf16_end];
    let u16s: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    String::from_utf16_lossy(&u16s)
}

/// 读取压缩无符号整数（ECMA-335 II.23.2），返回 `(值, 占用字节数)`。
///
/// 统一供 `#Blob` / `#US` 堆读取与元数据签名解码使用，替代旧实现中重复的
/// `read_compressed_uint` / `read_compressed_uint_at`。
pub fn read_compressed_uint(data: &[u8], offset: usize) -> (u32, usize) {
    if offset >= data.len() {
        return (0, 0);
    }
    let b0 = data[offset];
    if b0 & 0x80 == 0 {
        (b0 as u32, 1)
    }
    else if b0 & 0xC0 == 0x80 {
        if offset + 1 >= data.len() {
            return (0, 0);
        }
        let b1 = data[offset + 1];
        (((b0 as u32 & 0x3F) << 8) | b1 as u32, 2)
    }
    else if b0 & 0xE0 == 0xC0 {
        if offset + 3 >= data.len() {
            return (0, 0);
        }
        let b1 = data[offset + 1];
        let b2 = data[offset + 2];
        let b3 = data[offset + 3];
        (((b0 as u32 & 0x1F) << 24) | ((b1 as u32) << 16) | ((b2 as u32) << 8) | b3 as u32, 4)
    }
    else {
        (0, 0)
    }
}

fn read_u16(data: &[u8], offset: usize) -> Result<u16, PeParseError> {
    if data.len() < offset + 2 {
        return Err(PeParseError::TooShort);
    }
    Ok(u16::from_le_bytes([data[offset], data[offset + 1]]))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32, PeParseError> {
    if data.len() < offset + 4 {
        return Err(PeParseError::TooShort);
    }
    Ok(u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]))
}

fn read_u64(data: &[u8], offset: usize) -> Result<u64, PeParseError> {
    if data.len() < offset + 8 {
        return Err(PeParseError::TooShort);
    }
    Ok(u64::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::pe::{NativePeImage, NativePeWriter};

    #[test]
    fn parses_native_pe_writer_output() {
        let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
        let bytes = NativePeWriter::write_executable(&image).expect("write pe");
        let header = parse_pe64(&bytes).expect("parse pe");
        assert_eq!(header.machine, 0x8664);
        assert_eq!(header.optional_magic, 0x020B);
    }

    #[test]
    fn rejects_mz_only_stub() {
        assert_eq!(parse_pe64(b"MZ"), Err(Pe64ParseError::TooShort));
    }
}
