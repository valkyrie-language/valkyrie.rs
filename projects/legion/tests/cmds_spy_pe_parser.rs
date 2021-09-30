use legion::cmds::spy::pe_parser::{parse_pe, PeParseError};

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
