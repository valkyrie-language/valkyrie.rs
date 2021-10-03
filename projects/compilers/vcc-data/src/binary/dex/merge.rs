//! 多 dex/035 合并：将多组 JVM class 编入单一 `classes.dex`。
//!
//! `merge_dex_images` 从已有 dex 提取 **class 名** 并重建最小 stub（lossy，非 ID remap 合并）。
//! 生产路径优先 `merge_class_sets` 或维护者提供的 vendor dex。

use miette::{Result, miette};

use super::writer::DexImageBuilder;

/// 合并多组 `(internal_name, class_bytes)`；后者覆盖同名 class。
pub fn merge_class_sets(sets: &[&[(String, Vec<u8>)]]) -> Result<Vec<u8>> {
    if sets.is_empty() {
        return Err(miette!("merge_class_sets: 输入为空"));
    }
    let mut builder = DexImageBuilder::new();
    let mut order = Vec::new();
    let mut map = std::collections::BTreeMap::<String, Vec<u8>>::new();
    for set in sets {
        for (name, bytes) in *set {
            if !map.contains_key(name) {
                order.push(name.clone());
            }
            map.insert(name.clone(), bytes.clone());
        }
    }
    for name in order {
        if let Some(bytes) = map.get(&name) {
            builder.add_class(&name, bytes);
        }
    }
    builder.build()
}

/// 合并多个已写出的 dex 镜像（剥离尾段后按 class 名去重；依赖镜像由本工具链写出）。
pub fn merge_dex_images(images: &[&[u8]]) -> Result<Vec<u8>> {
    let mut sets = Vec::new();
    for dex in images {
        sets.push(extract_class_pairs(dex)?);
    }
    let refs: Vec<&[(String, Vec<u8>)]> = sets.iter().map(Vec::as_slice).collect();
    merge_class_sets(&refs)
}

/// 合并后在末尾附加尾段（ASGARDNT / ASGARDUI）。
pub fn merge_dex_images_with_tail(images: &[&[u8]], tail: &[u8]) -> Result<Vec<u8>> {
    let stripped: Vec<Vec<u8>> = images.iter().map(|d| strip_known_tail(d)).collect();
    let refs: Vec<&[u8]> = stripped.iter().map(Vec::as_slice).collect();
    let mut out = merge_dex_images(&refs)?;
    out.extend_from_slice(tail);
    Ok(out)
}

fn strip_known_tail(dex: &[u8]) -> Vec<u8> {
    for magic in [b"ASGARDNT", b"ASGARDUI"] {
        if let Some(pos) = find_subslice(dex, magic) {
            if pos >= 8 {
                let len_off = pos - 8;
                if let Some(len_bytes) = dex.get(len_off..len_off + 4) {
                    let len = u32::from_le_bytes(len_bytes.try_into().unwrap()) as usize;
                    let total = len_off.saturating_add(8).saturating_add(len);
                    if total <= dex.len() {
                        return dex[..len_off].to_vec();
                    }
                }
            }
        }
    }
    dex.to_vec()
}

fn find_subslice(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

fn extract_class_pairs(dex: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    use super::reader::dex_core_bytes;
    use crate::binary::class::JvmClassFile;
    let stripped = strip_known_tail(dex);
    let core = dex_core_bytes(&stripped);
    if core.len() < 8 || &core[..4] != b"dex\n" {
        return Err(miette!("不是 dex 镜像"));
    }
    let strings = super::reader::dex_string_ids(core)?;
    let type_ids_size = read_u32(core, 0x40)? as usize;
    let type_ids_off = read_u32(core, 0x44)? as usize;
    let class_defs_size = read_u32(core, 0x60)? as usize;
    let class_defs_off = read_u32(core, 0x64)? as usize;
    let mut out = Vec::new();
    for i in 0..class_defs_size {
        let off = class_defs_off + i * 32;
        let type_idx = read_u32(core, off)? as usize;
        if type_idx >= type_ids_size {
            continue;
        }
        let string_idx = read_u32(core, type_ids_off + type_idx * 4)? as usize;
        let descriptor = strings.get(string_idx).cloned().unwrap_or_default();
        let internal = descriptor.trim_start_matches('L').trim_end_matches(';');
        if internal.is_empty() {
            continue;
        }
        let mut class = JvmClassFile::new(internal);
        class.access_flags = 0x0021;
        out.push((internal.to_string(), class.to_bytes().map_err(|e| miette!("{e}"))?));
    }
    Ok(out)
}

fn read_u32(buf: &[u8], off: usize) -> Result<u32> {
    let bytes = buf.get(off..off + 4).ok_or_else(|| miette!("u32 越界"))?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::class::JvmClassFile;

    #[test]
    fn merge_two_minimal_dex() {
        let a_bytes = JvmClassFile::new("com/foo/A").to_bytes().unwrap();
        let b_bytes = JvmClassFile::new("com/bar/B").to_bytes().unwrap();
        let merged = merge_class_sets(&[&[("com/foo/A".to_string(), a_bytes)], &[("com/bar/B".to_string(), b_bytes)]]).unwrap();
        assert!(merged.starts_with(b"dex\n035"));
        assert!(super::super::reader::dex_contains_strings(&merged, &["com/foo/A", "com/bar/B"]).unwrap());
    }
}
