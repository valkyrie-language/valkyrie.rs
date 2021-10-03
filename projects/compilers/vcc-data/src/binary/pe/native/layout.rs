use crate::binary::pe::writer::{FILE_ALIGNMENT, SECTION_ALIGNMENT, align_up};

/// 节在镜像中的布局描述。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionPlacement {
    /// 虚拟地址（RVA）。
    pub rva: u32,
    /// 内容真实长度（写入 `VirtualSize`）。
    pub virtual_size: u32,
    /// 文件中顶点对齐后的字节数。
    pub raw_size: u32,
    /// 文件指针。
    pub raw_ptr: u32,
    /// 下一节起点 RVA（section-aligned）。
    pub next_rva: u32,
    /// 下一节文件指针。
    pub next_raw_ptr: u32,
}

/// 原生 PE 节表布局（空节被省略）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeSectionLayout {
    /// `.text`
    pub text: SectionPlacement,
    /// `.rdata`，缺省时为 `None`。
    pub rdata: Option<SectionPlacement>,
    /// `.idata`，缺省时为 `None`。
    pub idata: Option<SectionPlacement>,
    /// `SizeOfHeaders`（file-aligned）。
    pub size_of_headers: u32,
    /// `SizeOfImage`。
    pub size_of_image: u32,
    /// 节数量。
    pub section_count: u16,
    /// 初始化数据节原始大小之和。
    pub size_of_initialized_data: u32,
}

impl NativeSectionLayout {
    /// 根据节内容长度计算布局。
    pub fn compute(text_len: u32, rdata_len: u32, idata_len: u32) -> Self {
        let has_rdata = rdata_len > 0;
        let has_idata = idata_len > 0;
        let section_count = 1 + u16::from(has_rdata) + u16::from(has_idata);
        let optional_header_size = 240u32;
        let headers_size = 64 + 4 + 20 + optional_header_size + u32::from(section_count) * 40;
        let size_of_headers = align_up(headers_size, FILE_ALIGNMENT);

        let text = place_section(SECTION_ALIGNMENT, size_of_headers, text_len);
        let mut next_rva = text.next_rva;
        let mut next_raw_ptr = text.next_raw_ptr;
        let mut size_of_initialized_data = 0u32;

        let rdata = if has_rdata {
            let placement = place_section(next_rva, next_raw_ptr, rdata_len);
            next_rva = placement.next_rva;
            next_raw_ptr = placement.next_raw_ptr;
            size_of_initialized_data += placement.raw_size;
            Some(placement)
        }
        else {
            None
        };

        let idata = if has_idata {
            let placement = place_section(next_rva, next_raw_ptr, idata_len);
            next_rva = placement.next_rva;
            size_of_initialized_data += placement.raw_size;
            Some(placement)
        }
        else {
            None
        };

        Self { text, rdata, idata, size_of_headers, size_of_image: next_rva, section_count, size_of_initialized_data }
    }
}

fn place_section(rva: u32, raw_ptr: u32, content_len: u32) -> SectionPlacement {
    let virtual_size = content_len;
    let raw_size = align_up(content_len, FILE_ALIGNMENT);
    let next_rva = rva + align_up(content_len.max(1), SECTION_ALIGNMENT);
    let next_raw_ptr = raw_ptr + raw_size;
    SectionPlacement { rva, virtual_size, raw_size, raw_ptr, next_rva, next_raw_ptr }
}
