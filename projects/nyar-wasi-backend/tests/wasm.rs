use nyar_wasi_backend::{WasmBinaryModule, WasmSection};

#[test]
fn round_trips_custom_and_standard_sections() {
    let mut module = WasmBinaryModule::new();
    module.push_custom_section("name", b"demo".to_vec());
    module.sections.push(WasmSection { id: 1, name: None, bytes: vec![0x01, 0x60, 0x00, 0x00] });

    let bytes = module.to_bytes().unwrap();
    let decoded = WasmBinaryModule::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.sections.len(), 2);
    assert_eq!(decoded.custom_sections()[0].name, "name");
    assert_eq!(decoded.sections[1].id, 1);
}
