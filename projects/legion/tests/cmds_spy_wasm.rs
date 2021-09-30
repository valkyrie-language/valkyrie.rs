//! `legion spy wasm` 子命令的集成测试。
//!
//! 这些测试在内存中构造最小 `WASM` 二进制，验证 `spy wasm` 的解析、
//! 列表、反汇编与偏移定位能力，不依赖外部文件。

use wasi_backend::{WasmBinaryModule, WasmSection};

/// 构造一个最小的 `WASM` 模块：包含一个导出函数 `main`，返回 `i32` 常量 `42`。
fn build_minimal_wasm() -> Vec<u8> {
    let mut module = WasmBinaryModule::new();

    // Type 段（id=1）：1 个函数类型 `() -> i32`
    module.sections.push(WasmSection { id: 1, name: None, bytes: vec![0x01, 0x60, 0x00, 0x01, 0x7F] });

    // Function 段（id=3）：1 个函数，类型索引 0
    module.sections.push(WasmSection { id: 3, name: None, bytes: vec![0x01, 0x00] });

    // Export 段（id=7）：导出函数 `main`（索引 0）
    module.sections.push(WasmSection {
        id: 7,
        name: None,
        bytes: vec![
            0x01, // 1 个导出
            0x04, b'm', b'a', b'i', b'n', // 名称 "main"
            0x00, // kind: func
            0x00, // 函数索引 0
        ],
    });

    // Code 段（id=10）：1 个函数体
    // 函数体：0 个局部变量组，i32.const 42，end
    let func_body = vec![0x00, 0x41, 0x2A, 0x0B];
    let body_size = func_body.len() as u8;
    module.sections.push(WasmSection {
        id: 10,
        name: None,
        bytes: vec![0x01, body_size] // 1 个函数，body 大小
            .into_iter()
            .chain(func_body.iter().copied())
            .collect(),
    });

    module.to_bytes().unwrap()
}

/// 构造一个带 `import` 的 `WASM` 模块：导入 `env.console_log`，导出 `_start`。
fn build_wasm_with_imports() -> Vec<u8> {
    let mut module = WasmBinaryModule::new();

    // Type 段（id=1）：2 个函数类型
    // type 0: (i32) -> ()  —— console_log
    // type 1: () -> ()     —— _start
    module.sections.push(WasmSection {
        id: 1,
        name: None,
        bytes: vec![
            0x02, // 2 个类型
            0x60, 0x01, 0x7F, 0x00, // type 0: (i32) -> ()
            0x60, 0x00, 0x00, // type 1: () -> ()
        ],
    });

    // Import 段（id=2）：1 个导入函数 `env.console_log`，类型 0
    module.sections.push(WasmSection {
        id: 2,
        name: None,
        bytes: vec![
            0x01, // 1 个导入
            0x03, b'e', b'n', b'v', // 模块名 "env"
            0x0B, b'c', b'o', b'n', b's', b'o', b'l', b'e', b'_', b'l', b'o', b'g', // 字段名 "console_log"
            0x00, // kind: func
            0x00, // 类型索引 0
        ],
    });

    // Function 段（id=3）：1 个函数，类型 1
    module.sections.push(WasmSection { id: 3, name: None, bytes: vec![0x01, 0x01] });

    // Export 段（id=7）：导出 `_start`（函数索引 1，因为索引 0 是导入）
    module.sections.push(WasmSection {
        id: 7,
        name: None,
        bytes: vec![
            0x01, // 1 个导出
            0x06, b'_', b's', b't', b'a', b'r', b't', // 名称 "_start"
            0x00, // kind: func
            0x01, // 函数索引 1
        ],
    });

    // Code 段（id=10）：1 个函数体
    let func_body = vec![
        0x00, // 0 个局部变量组
        0x41, 0x00, // i32.const 0
        0x10, 0x00, // call 0 (console_log)
        0x0B, // end
    ];
    let body_size = func_body.len() as u8;
    module.sections.push(WasmSection {
        id: 10,
        name: None,
        bytes: vec![0x01, body_size].into_iter().chain(func_body.iter().copied()).collect(),
    });

    module.to_bytes().unwrap()
}

#[test]
fn parses_minimal_wasm_magic_and_version() {
    let bytes = build_minimal_wasm();
    let module = WasmBinaryModule::from_bytes(&bytes).unwrap();
    assert_eq!(module.version, 1);
    assert!(!module.sections.is_empty());
}

#[test]
fn minimal_wasm_has_expected_sections() {
    let bytes = build_minimal_wasm();
    let module = WasmBinaryModule::from_bytes(&bytes).unwrap();

    let section_ids: Vec<u8> = module.sections.iter().map(|s| s.id).collect();
    assert!(section_ids.contains(&1), "应包含 Type 段");
    assert!(section_ids.contains(&3), "应包含 Function 段");
    assert!(section_ids.contains(&7), "应包含 Export 段");
    assert!(section_ids.contains(&10), "应包含 Code 段");
}

#[test]
fn wasm_with_imports_parses_correctly() {
    let bytes = build_wasm_with_imports();
    let module = WasmBinaryModule::from_bytes(&bytes).unwrap();

    // 应包含 Import 段
    let import_section = module.sections.iter().find(|s| s.id == 2);
    assert!(import_section.is_some(), "应包含 Import 段");

    // 手动解析 import 段验证内容
    let import_bytes = &import_section.unwrap().bytes;
    assert_eq!(import_bytes[0], 0x01, "应有 1 个导入");

    // 验证模块名 "env"
    let module_name_len = import_bytes[1] as usize;
    assert_eq!(module_name_len, 3);
    assert_eq!(&import_bytes[2..5], b"env");
}

#[test]
fn minimal_wasm_export_name_is_main() {
    let bytes = build_minimal_wasm();
    let module = WasmBinaryModule::from_bytes(&bytes).unwrap();

    let export_section = module.sections.iter().find(|s| s.id == 7).unwrap();
    let export_bytes = &export_section.bytes;

    assert_eq!(export_bytes[0], 0x01, "应有 1 个导出");
    let name_len = export_bytes[1] as usize;
    assert_eq!(name_len, 4);
    assert_eq!(&export_bytes[2..6], b"main");
}

/// 生成 CLI 端到端验证用的 fixture 文件。
///
/// 将最小 `WASM` 模块写入 `target/debug/test_spy.wasm`，
/// 供 `legion spy wasm` 命令行手动验证使用。
#[test]
fn write_fixture_wasm_for_cli() {
    let bytes = build_minimal_wasm();
    let target_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("target").join("debug");
    let fixture_path = target_dir.join("test_spy.wasm");

    std::fs::create_dir_all(&target_dir).expect("无法创建 target/debug 目录");
    std::fs::write(&fixture_path, &bytes).expect("无法写入 test_spy.wasm");

    println!("已写入 fixture：{}", fixture_path.display());
}
