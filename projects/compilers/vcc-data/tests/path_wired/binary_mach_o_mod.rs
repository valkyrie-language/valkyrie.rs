//! Migrated from `src/binary/mach_o/mod.rs` (`tests`).

use super::*;

#[test]
fn mach_o_magic() {
    let exe = MachOImageBuilder::new().build_executable().unwrap();
    assert_eq!(&exe[..4], &MH_MAGIC_64.to_le_bytes());
}
