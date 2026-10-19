//! Migrated from `src/binary/dex/merge.rs` (`tests`).

use super::*;

use acorn_jvm::class::JvmClassFile;

#[test]
fn merge_two_minimal_dex() {
    let a_bytes = JvmClassFile::new("com/foo/A").to_bytes().unwrap();
    let b_bytes = JvmClassFile::new("com/bar/B").to_bytes().unwrap();
    let merged = merge_class_sets(&[&[("com/foo/A".to_string(), a_bytes)], &[("com/bar/B".to_string(), b_bytes)]]).unwrap();
    assert!(merged.starts_with(b"dex\n035"));
    assert!(super::super::reader::dex_contains_strings(&merged, &["com/foo/A", "com/bar/B"]).unwrap());
}
