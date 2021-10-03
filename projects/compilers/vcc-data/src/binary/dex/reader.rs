//! 最小 dex/035 读取：string_ids、class_defs 与类型名扫描。

use miette::{Result, miette};

const DEX_MAGIC: &[u8] = b"dex\n035\0";

/// 按 header `file_size` 截取 dex 本体（忽略 ASGARD 尾段）。
pub fn dex_core_bytes(dex: &[u8]) -> &[u8] {
    if dex.len() >= 0x24 {
        let file_size = u32::from_le_bytes(dex[0x20..0x24].try_into().unwrap()) as usize;
        if file_size >= 0x70 && file_size <= dex.len() {
            return &dex[..file_size];
        }
    }
    dex
}

/// 解析 dex `string_ids` 表中的 UTF-8 字符串。
pub fn dex_string_ids(dex: &[u8]) -> Result<Vec<String>> {
    let dex = dex_core_bytes(dex);
    if dex.len() < 0x70 || dex.get(..8) != Some(DEX_MAGIC) {
        return Err(miette!("不是 dex/035 镜像"));
    }
    let string_ids_size = read_u32(dex, 0x38)? as usize;
    let string_ids_off = read_u32(dex, 0x3c)? as usize;
    if string_ids_off + string_ids_size.saturating_mul(4) > dex.len() {
        return Err(miette!("dex string_ids 越界"));
    }
    let mut out = Vec::with_capacity(string_ids_size);
    for i in 0..string_ids_size {
        let data_off = read_u32(dex, string_ids_off + i * 4)? as usize;
        if data_off >= dex.len() {
            continue;
        }
        let (len, cursor) = read_uleb128(dex, data_off)?;
        let end = cursor.saturating_add(len as usize);
        if end > dex.len() {
            continue;
        }
        let s = std::str::from_utf8(&dex[cursor..end]).unwrap_or("");
        out.push(s.to_string());
    }
    Ok(out)
}

/// 解析 `class_defs` 中的类型描述符（`L...;`）。
pub fn dex_class_descriptors(dex: &[u8]) -> Result<Vec<String>> {
    let dex = dex_core_bytes(dex);
    if dex.len() < 0x70 || dex.get(..8) != Some(DEX_MAGIC) {
        return Err(miette!("不是 dex/035 镜像"));
    }
    let strings = dex_string_ids(dex)?;
    let type_ids_size = read_u32(dex, 0x40)? as usize;
    let type_ids_off = read_u32(dex, 0x44)? as usize;
    let class_defs_size = read_u32(dex, 0x60)? as usize;
    let class_defs_off = read_u32(dex, 0x64)? as usize;
    if type_ids_off + type_ids_size.saturating_mul(4) > dex.len() {
        return Err(miette!("dex type_ids 越界"));
    }
    if class_defs_off + class_defs_size.saturating_mul(32) > dex.len() {
        return Err(miette!("dex class_defs 越界"));
    }
    let mut out = Vec::with_capacity(class_defs_size);
    for i in 0..class_defs_size {
        let off = class_defs_off + i * 32;
        let type_idx = read_u32(dex, off)? as usize;
        if type_idx >= type_ids_size {
            continue;
        }
        let string_idx = read_u32(dex, type_ids_off + type_idx * 4)? as usize;
        if let Some(descriptor) = strings.get(string_idx) {
            out.push(descriptor.clone());
        }
    }
    Ok(out)
}

/// `class_defs` 表项数量（dex 本体，忽略 ASGARD 尾段）。
pub fn dex_class_defs_count(dex: &[u8]) -> Result<usize> {
    let dex = dex_core_bytes(dex);
    if dex.len() < 0x70 || dex.get(..8) != Some(DEX_MAGIC) {
        return Err(miette!("不是 dex/035 镜像"));
    }
    Ok(read_u32(dex, 0x60)? as usize)
}

/// dex 字符串表是否包含全部 `needles`（子串匹配）。
pub fn dex_contains_strings(dex: &[u8], needles: &[&str]) -> Result<bool> {
    let strings = dex_string_ids(dex)?;
    let haystack = strings.join("\0");
    if needles.iter().all(|n| haystack.contains(n)) {
        return Ok(true);
    }
    let lossy = String::from_utf8_lossy(dex_core_bytes(dex));
    Ok(needles.iter().all(|n| lossy.contains(n)))
}

fn read_u32(dex: &[u8], off: usize) -> Result<u32> {
    let bytes = dex.get(off..off + 4).ok_or_else(|| miette!("dex 读取 u32 越界"))?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_uleb128(dex: &[u8], mut off: usize) -> Result<(u32, usize)> {
    let mut result = 0u32;
    let mut shift = 0;
    loop {
        let b = *dex.get(off).ok_or_else(|| miette!("uleb128 越界"))?;
        off += 1;
        result |= u32::from(b & 0x7f) << shift;
        if b & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift > 35 {
            return Err(miette!("uleb128 过长"));
        }
    }
    Ok((result, off))
}
