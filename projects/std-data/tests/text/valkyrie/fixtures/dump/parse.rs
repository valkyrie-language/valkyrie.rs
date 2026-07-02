use std_data::text::valkyrie::parse_dump::{dump_parse_tree, dump_parse_vx_tree};

/// 将源码格式化为稳定的 `*.parse` 树形文本快照。
pub fn dump_parse_snapshot(source: &str) -> String {
    dump_parse_tree(source)
}

/// 将 `.vx` 源码格式化为稳定的 `*.parse` 树形文本快照。
pub fn dump_parse_vx_snapshot(source: &str) -> String {
    dump_parse_vx_tree(source)
}
