//! Migrated from `src/binary/aarch64/mod.rs` (`tests`).

use super::*;

#[test]
fn ret_opcode_is_four_bytes() {
    assert_eq!(ret_bytes(), [0xC0, 0x03, 0x5F, 0xD6]);
}
