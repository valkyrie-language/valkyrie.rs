//! Migrated from `src/binary/dex/writer.rs` (`tests`).

use super::*;

use crate::binary::class::{JvmClassFile, JvmMethodDescriptor, JvmTypeDescriptor};

#[test]
fn dex_has_class_defs_and_magic() {
    let mut bridge = JvmClassFile::new("com/asgard/runtime/AsgardHostBridge");
    bridge.access_flags = 0x0031;
    bridge.push_method(
        "invokeExport",
        JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".into())], JvmTypeDescriptor::Void),
    );
    bridge.methods[0].access_flags = 0x0109;
    let bytes = bridge.to_bytes().expect("class");
    let mut builder = DexImageBuilder::new();
    builder.add_class("com/asgard/runtime/AsgardHostBridge", &bytes);
    let dex = builder.build().expect("dex");
    assert!(dex.starts_with(b"dex\n035"));
    assert!(dex.windows(b"AsgardHostBridge".len()).any(|w| w == b"AsgardHostBridge"));
}
