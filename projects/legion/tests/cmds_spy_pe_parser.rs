//! `PE` 解析入口的行为测试。
//!
//! `PE/CLR` 解析逻辑已下沉至 `std-data`，本测试直接消费 `std_data::binary::pe`，
//! 验证 `parse_pe` 在边界输入下的错误归类。

use std_data::binary::pe::{PeParseError, parse_pe};

#[test]
fn rejects_empty_data() {
    let result = parse_pe(&[]);
    assert_eq!(result.unwrap_err(), PeParseError::TooShort);
}

#[test]
fn rejects_bad_mz() {
    let mut data = vec![0u8; 128];
    data[0] = b'X';
    data[1] = b'Y';
    let result = parse_pe(&data);
    assert_eq!(result.unwrap_err(), PeParseError::BadMzMagic);
}
