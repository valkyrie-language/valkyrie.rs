//! Migrated from `src/binary/tvm/mod.rs` (`tests`).

use super::*;

#[test]
fn encode_decode_round_trip() {
    let header = TvmHeader {
        version: 42,
        encoding: 1,
        operation: TvmOperation::Find as u8,
        flags: TvmFlags::HAS_PREFIX | TvmFlags::HAS_CAPTURE,
        min_match_len: 7,
        literal_prefix_offset: 32,
        dfa_table_offset: 128,
        dfa_state_count: 25,
        total_size: 512,
    };

    let encoded = header.to_bytes();
    let decoded = TvmHeader::decode(&encoded).expect("decode");
    assert_eq!(decoded, header);
}

#[test]
fn bad_magic_is_rejected() {
    let mut buf = [0u8; TVM_HEADER_SIZE];
    buf[..4].copy_from_slice(b"BAD\0");
    assert!(matches!(TvmHeader::decode(&buf), Err(TvmHeaderError::InvalidMagic(_))));
}
