//! `WASM` 段级解码层集成测试。
//!
//! 覆盖 `import` / `export` / `code` / `type` 四段解析，
//! 以及 `compute_section_offset` / `wasm_value_type_name` / `read_string` 等辅助 API。

use std_data::binary::wasm::{
    SECTION_CODE, SECTION_EXPORT, SECTION_IMPORT, SECTION_TYPE, WasmBinaryModule, WasmByteReader, WasmExternalKind, WasmHeapType, WasmSection,
    WasmTypeKind, WasmValueType, compute_section_offset, parse_code_section, parse_export_section, parse_import_section, parse_type_section,
    read_heap_type, read_value_type, section_name, skip_value_type, uleb128_size, wasm_heap_type_name, wasm_value_type_name,
};

/// 追加无符号 `LEB128` 编码到字节缓冲。
fn push_uleb128(buf: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// 追加长度前缀字符串到字节缓冲。
fn push_string(buf: &mut Vec<u8>, s: &str) {
    push_uleb128(buf, s.len() as u32);
    buf.extend_from_slice(s.as_bytes());
}

/// 构建包含 4 种 `kind` 的 `import` 段 `payload`。
fn build_import_payload_four_kinds() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 4);

    // kind 0 (func): env.log type_index=0
    push_string(&mut buf, "env");
    push_string(&mut buf, "log");
    buf.push(0x00);
    push_uleb128(&mut buf, 0);

    // kind 1 (table): env.tbl elem=0x70 funcref, flags=0, min=0
    push_string(&mut buf, "env");
    push_string(&mut buf, "tbl");
    buf.push(0x01);
    buf.push(0x70);
    buf.push(0x00);
    push_uleb128(&mut buf, 0);

    // kind 2 (memory): env.mem flags=1, min=1, max=10
    push_string(&mut buf, "env");
    push_string(&mut buf, "mem");
    buf.push(0x02);
    buf.push(0x01);
    push_uleb128(&mut buf, 1);
    push_uleb128(&mut buf, 10);

    // kind 3 (global): env.glb valtype=0x7F i32, mutable=1
    push_string(&mut buf, "env");
    push_string(&mut buf, "glb");
    buf.push(0x03);
    buf.push(0x7F);
    buf.push(0x01);

    buf
}

/// 构建仅含 1 个函数导入的 `import` 段 `payload`。
fn build_import_payload_one_func() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 1);
    push_string(&mut buf, "env");
    push_string(&mut buf, "log");
    buf.push(0x00);
    push_uleb128(&mut buf, 0);
    buf
}

/// 构建含 2 个导出的 `export` 段 `payload`。
fn build_export_payload_two() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 2);

    // export "add" kind=func index=1
    push_string(&mut buf, "add");
    buf.push(0x00);
    push_uleb128(&mut buf, 1);

    // export "memory" kind=memory index=0
    push_string(&mut buf, "memory");
    buf.push(0x02);
    push_uleb128(&mut buf, 0);

    buf
}

/// 构建含 1 个函数体的 `code` 段 `payload`。
///
/// 函数体：1 个局部变量组（1 个 `i32`），指令为 `end`。
fn build_code_payload_one_func() -> Vec<u8> {
    let mut body = Vec::new();
    push_uleb128(&mut body, 1);
    push_uleb128(&mut body, 1);
    body.push(0x7F);
    body.push(0x0B);

    let mut buf = Vec::new();
    push_uleb128(&mut buf, 1);
    push_uleb128(&mut buf, body.len() as u32);
    buf.extend_from_slice(&body);
    buf
}

/// 构建含 3 种 `form` 的 `type` 段 `payload`。
fn build_type_payload_three_forms() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 3);

    // entry 0: func 0x60, 1 param i32, 0 results
    buf.push(0x60);
    push_uleb128(&mut buf, 1);
    buf.push(0x7F);
    push_uleb128(&mut buf, 0);

    // entry 1: struct 0x5F, 1 field (i32, immutable)
    buf.push(0x5F);
    push_uleb128(&mut buf, 1);
    buf.push(0x7F);
    buf.push(0x00);

    // entry 2: array 0x61, element i32, mutable
    buf.push(0x61);
    buf.push(0x7F);
    buf.push(0x01);

    buf
}

/// 创建仅含指定段的模块。
fn module_with_section(id: u8, bytes: Vec<u8>) -> WasmBinaryModule {
    WasmBinaryModule { version: 1, sections: vec![WasmSection { id, name: None, bytes }] }
}

#[test]
fn read_string_读取长度前缀字符串() {
    let mut bytes = Vec::new();
    push_string(&mut bytes, "hello");
    bytes.push(0xFF);

    let mut reader = WasmByteReader::new(&bytes);
    let s = reader.read_string().expect("读取字符串");
    assert_eq!(s, "hello");
    assert_eq!(reader.offset(), 6);
    assert_eq!(reader.read_u8().unwrap(), 0xFF);
}

#[test]
fn read_string_无效utf8返回错误() {
    let mut bytes = Vec::new();
    push_uleb128(&mut bytes, 2);
    bytes.push(0xFF);
    bytes.push(0xFE);

    let mut reader = WasmByteReader::new(&bytes);
    let result = reader.read_string();
    assert!(result.is_err());
}

#[test]
fn section_name_覆盖全部已知id() {
    assert_eq!(section_name(0), "custom");
    assert_eq!(section_name(1), "type");
    assert_eq!(section_name(2), "import");
    assert_eq!(section_name(3), "function");
    assert_eq!(section_name(4), "table");
    assert_eq!(section_name(5), "memory");
    assert_eq!(section_name(6), "global");
    assert_eq!(section_name(7), "export");
    assert_eq!(section_name(8), "start");
    assert_eq!(section_name(9), "element");
    assert_eq!(section_name(10), "code");
    assert_eq!(section_name(11), "data");
    assert_eq!(section_name(12), "data_count");
    assert_eq!(section_name(99), "unknown");
}

#[test]
fn uleb128_size_典型值() {
    assert_eq!(uleb128_size(0), 1);
    assert_eq!(uleb128_size(1), 1);
    assert_eq!(uleb128_size(127), 1);
    assert_eq!(uleb128_size(128), 2);
    assert_eq!(uleb128_size(16383), 2);
    assert_eq!(uleb128_size(16384), 3);
}

#[test]
fn skip_value_type_普通类型占一字节() {
    let mut reader = WasmByteReader::new(&[0x7F, 0x42]);
    skip_value_type(&mut reader).expect("跳过值类型");
    assert_eq!(reader.offset(), 1);
    assert_eq!(reader.read_u8().unwrap(), 0x42);
}

#[test]
fn skip_value_type_0x64前缀消耗额外sleb128() {
    let bytes = [0x64, 0x01, 0x42];
    let mut reader = WasmByteReader::new(&bytes);
    skip_value_type(&mut reader).expect("跳过值类型");
    assert_eq!(reader.read_u8().unwrap(), 0x42);
}

#[test]
fn skip_value_type_0x63前缀消耗额外sleb128() {
    // 0x63 (ref null ht) + heap type 0x70 (单字节 s33 = -16 = func) + 后续字节 0x42
    let bytes = [0x63, 0x70, 0x42];
    let mut reader = WasmByteReader::new(&bytes);
    skip_value_type(&mut reader).expect("跳过值类型");
    assert_eq!(reader.read_u8().unwrap(), 0x42);
}

#[test]
fn read_value_type_单字节数值类型() {
    let mut reader = WasmByteReader::new(&[0x7F, 0x7E, 0x7D, 0x7C, 0x7B]);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::I32);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::I64);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::F32);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::F64);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::V128);
}

#[test]
fn read_value_type_单字节引用类型() {
    let mut reader = WasmByteReader::new(&[0x70, 0x6F]);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::FuncRef);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::ExternRef);
}

#[test]
fn read_value_type_多字节ref_type() {
    // 0x64 + 0x70 = (ref func)
    let mut reader = WasmByteReader::new(&[0x64, 0x70]);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::Ref(WasmHeapType::Func));
    // 0x63 + 0x6B = (ref null struct)
    let mut reader = WasmByteReader::new(&[0x63, 0x6B]);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::RefNull(WasmHeapType::Struct));
    // 0x64 + 0x05 = (ref typeidx:5)
    let mut reader = WasmByteReader::new(&[0x64, 0x05]);
    assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::Ref(WasmHeapType::Index(5)));
}

#[test]
fn read_heap_type_抽象类型与索引() {
    // 单字节 0x70 作为 s33 = -16 = func
    let mut reader = WasmByteReader::new(&[0x70]);
    assert_eq!(read_heap_type(&mut reader).unwrap(), WasmHeapType::Func);
    // 0x6F = -17 = extern
    let mut reader = WasmByteReader::new(&[0x6F]);
    assert_eq!(read_heap_type(&mut reader).unwrap(), WasmHeapType::Extern);
    // 0x69 = -23 = none (-0x17)
    let mut reader = WasmByteReader::new(&[0x69]);
    assert_eq!(read_heap_type(&mut reader).unwrap(), WasmHeapType::None);
    // 0x05 = 非负类型索引 5
    let mut reader = WasmByteReader::new(&[0x05]);
    assert_eq!(read_heap_type(&mut reader).unwrap(), WasmHeapType::Index(5));
}

#[test]
fn parse_import_section_覆盖四种kind() {
    let payload = build_import_payload_four_kinds();
    let module = module_with_section(SECTION_IMPORT, payload);
    let imports = parse_import_section(&module);

    assert_eq!(imports.len(), 4);

    let func = &imports[0];
    assert_eq!(func.module, "env");
    assert_eq!(func.field, "log");
    assert_eq!(func.kind, WasmExternalKind::Func);
    assert_eq!(func.type_index, 0);

    let table = &imports[1];
    assert_eq!(table.kind, WasmExternalKind::Table);
    assert_eq!(table.table_elem_type, WasmValueType::FuncRef);
    assert_eq!(table.memory_min, 0);
    assert_eq!(table.memory_max, None);

    let memory = &imports[2];
    assert_eq!(memory.kind, WasmExternalKind::Memory);
    assert_eq!(memory.memory_min, 1);
    assert_eq!(memory.memory_max, Some(10));

    let global = &imports[3];
    assert_eq!(global.kind, WasmExternalKind::Global);
    assert_eq!(global.global_value_type, WasmValueType::I32);
    assert!(global.global_mutable);
}

#[test]
fn parse_import_section_无导入段返回空() {
    let module = WasmBinaryModule::new();
    let imports = parse_import_section(&module);
    assert!(imports.is_empty());
}

#[test]
fn parse_export_section_基本导出() {
    let payload = build_export_payload_two();
    let module = module_with_section(SECTION_EXPORT, payload);
    let exports = parse_export_section(&module);

    assert_eq!(exports.len(), 2);

    let func_export = &exports[0];
    assert_eq!(func_export.name, "add");
    assert_eq!(func_export.kind, WasmExternalKind::Func);
    assert_eq!(func_export.index, 1);

    let mem_export = &exports[1];
    assert_eq!(mem_export.name, "memory");
    assert_eq!(mem_export.kind, WasmExternalKind::Memory);
    assert_eq!(mem_export.index, 0);
}

#[test]
fn parse_export_section_无导出段返回空() {
    let module = WasmBinaryModule::new();
    let exports = parse_export_section(&module);
    assert!(exports.is_empty());
}

#[test]
fn parse_code_section_函数索引与偏移() {
    let import_payload = build_import_payload_one_func();
    let code_payload = build_code_payload_one_func();
    let module = WasmBinaryModule {
        version: 1,
        sections: vec![
            WasmSection { id: SECTION_IMPORT, name: None, bytes: import_payload },
            WasmSection { id: SECTION_CODE, name: None, bytes: code_payload },
        ],
    };

    let raw_bytes = module.to_bytes().expect("序列化模块");
    let parsed_module = WasmBinaryModule::from_bytes(&raw_bytes).expect("解析模块");
    let functions = parse_code_section(&parsed_module);

    assert_eq!(functions.len(), 1);

    let func = &functions[0];
    assert_eq!(func.index, 1);
    assert_eq!(func.local_groups, 1);
    assert_eq!(func.body_len, 4);

    let expected_offset = compute_section_offset(&parsed_module, SECTION_CODE) + 2;
    assert_eq!(func.code_offset, expected_offset);

    let body = &raw_bytes[func.code_offset..func.code_offset + func.body_len];
    assert_eq!(body, &[0x01, 0x01, 0x7F, 0x0B]);
}

#[test]
fn parse_code_section_无code段返回空() {
    let module = WasmBinaryModule::new();
    let functions = parse_code_section(&module);
    assert!(functions.is_empty());
}

#[test]
fn parse_code_section_函数名来自export映射() {
    let import_payload = build_import_payload_one_func();
    let code_payload = build_code_payload_one_func();
    let export_payload = build_export_payload_two();
    let module = WasmBinaryModule {
        version: 1,
        sections: vec![
            WasmSection { id: SECTION_IMPORT, name: None, bytes: import_payload },
            WasmSection { id: SECTION_EXPORT, name: None, bytes: export_payload },
            WasmSection { id: SECTION_CODE, name: None, bytes: code_payload },
        ],
    };

    let functions = parse_code_section(&module);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].index, 1);
    assert_eq!(functions[0].name.as_deref(), Some("add"));
}

#[test]
fn parse_type_section_三种form() {
    let payload = build_type_payload_three_forms();
    let module = module_with_section(SECTION_TYPE, payload);

    let raw_bytes = module.to_bytes().expect("序列化模块");
    let parsed_module = WasmBinaryModule::from_bytes(&raw_bytes).expect("解析模块");
    let entries = parse_type_section(&parsed_module);

    assert_eq!(entries.len(), 3);

    let func_entry = &entries[0];
    assert_eq!(func_entry.index, 0);
    assert_eq!(func_entry.kind, WasmTypeKind::Func { params: vec![WasmValueType::I32], results: vec![] });
    assert_eq!(raw_bytes[func_entry.file_offset], 0x60);

    let struct_entry = &entries[1];
    assert_eq!(struct_entry.index, 1);
    assert_eq!(struct_entry.kind, WasmTypeKind::Struct { fields: vec![(WasmValueType::I32, false)] });
    assert_eq!(raw_bytes[struct_entry.file_offset], 0x5F);

    let array_entry = &entries[2];
    assert_eq!(array_entry.index, 2);
    assert_eq!(array_entry.kind, WasmTypeKind::Array { element: WasmValueType::I32, mutable: true });
    assert_eq!(raw_bytes[array_entry.file_offset], 0x61);
}

#[test]
fn parse_type_section_无type段返回空() {
    let module = WasmBinaryModule::new();
    let entries = parse_type_section(&module);
    assert!(entries.is_empty());
}

#[test]
fn parse_type_section_未知form归为unknown() {
    let mut payload = Vec::new();
    push_uleb128(&mut payload, 1);
    payload.push(0xAB);
    let module = module_with_section(SECTION_TYPE, payload);
    let entries = parse_type_section(&module);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, WasmTypeKind::Unknown { form: 0xAB });
}

#[test]
fn compute_section_offset_单段模块() {
    let payload = vec![0x01, 0x60, 0x00, 0x00];
    let module = module_with_section(SECTION_TYPE, payload);
    let raw_bytes = module.to_bytes().expect("序列化模块");

    let offset = compute_section_offset(&module, SECTION_TYPE);
    assert_eq!(offset, 10);
    assert_eq!(raw_bytes[offset], 0x01);
}

#[test]
fn compute_section_offset_多段模块() {
    let import_payload = build_import_payload_one_func();
    let code_payload = build_code_payload_one_func();
    let module = WasmBinaryModule {
        version: 1,
        sections: vec![
            WasmSection { id: SECTION_IMPORT, name: None, bytes: import_payload },
            WasmSection { id: SECTION_CODE, name: None, bytes: code_payload },
        ],
    };
    let raw_bytes = module.to_bytes().expect("序列化模块");

    let import_offset = compute_section_offset(&module, SECTION_IMPORT);
    assert_eq!(raw_bytes[import_offset], 0x01);

    let code_offset = compute_section_offset(&module, SECTION_CODE);
    assert_eq!(raw_bytes[code_offset], 0x01);
}

#[test]
fn compute_section_offset_含自定义段() {
    let module = WasmBinaryModule {
        version: 1,
        sections: vec![
            WasmSection { id: 0, name: Some("name".to_string()), bytes: vec![0x01, 0x02, 0x03] },
            WasmSection { id: SECTION_TYPE, name: None, bytes: vec![0x01, 0x60, 0x00, 0x00] },
        ],
    };
    let raw_bytes = module.to_bytes().expect("序列化模块");

    let type_offset = compute_section_offset(&module, SECTION_TYPE);
    assert_eq!(raw_bytes[type_offset], 0x01);
}

#[test]
fn wasm_value_type_name_典型变体() {
    assert_eq!(wasm_value_type_name(&WasmValueType::I32), "i32");
    assert_eq!(wasm_value_type_name(&WasmValueType::I64), "i64");
    assert_eq!(wasm_value_type_name(&WasmValueType::F32), "f32");
    assert_eq!(wasm_value_type_name(&WasmValueType::F64), "f64");
    assert_eq!(wasm_value_type_name(&WasmValueType::V128), "v128");
    assert_eq!(wasm_value_type_name(&WasmValueType::FuncRef), "funcref");
    assert_eq!(wasm_value_type_name(&WasmValueType::ExternRef), "externref");
    assert_eq!(wasm_value_type_name(&WasmValueType::Ref(WasmHeapType::Func)), "ref func");
    assert_eq!(wasm_value_type_name(&WasmValueType::RefNull(WasmHeapType::Struct)), "ref null struct");
    assert_eq!(wasm_value_type_name(&WasmValueType::Ref(WasmHeapType::Index(5))), "ref typeidx:5");
    assert_eq!(wasm_value_type_name(&WasmValueType::Unknown(0x42)), "unknown(0x42)");
}

#[test]
fn wasm_heap_type_name_典型变体() {
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Func), "func");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Extern), "extern");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Any), "any");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Eq), "eq");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::I31), "i31");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Struct), "struct");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Array), "array");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::None), "none");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::NoFunc), "nofunc");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::NoExtern), "noextern");
    assert_eq!(wasm_heap_type_name(&WasmHeapType::Index(7)), "typeidx:7");
}

#[test]
fn wasm_external_kind_from_u8_覆盖全部种类() {
    assert_eq!(WasmExternalKind::from_u8(0), WasmExternalKind::Func);
    assert_eq!(WasmExternalKind::from_u8(1), WasmExternalKind::Table);
    assert_eq!(WasmExternalKind::from_u8(2), WasmExternalKind::Memory);
    assert_eq!(WasmExternalKind::from_u8(3), WasmExternalKind::Global);
    assert_eq!(WasmExternalKind::from_u8(0xFF), WasmExternalKind::Unknown(0xFF));
}

#[test]
fn wasm_type_kind_name_可读名称() {
    assert_eq!(WasmTypeKind::Func { params: vec![], results: vec![] }.name(), "functype");
    assert_eq!(WasmTypeKind::Struct { fields: vec![] }.name(), "structtype");
    assert_eq!(WasmTypeKind::Array { element: WasmValueType::I32, mutable: false }.name(), "arraytype");
    assert_eq!(WasmTypeKind::Unknown { form: 0xAB }.name(), "unknown");
}

// ========== 切片 R：多字节 reftype 解码（GC proposal）==========

/// 构建 type 段 payload：含多字节 reftype 的 func / struct / array 条目，
/// 并在末尾追加一个普通 func 条目以验证游标未错位。
fn build_type_payload_with_reftypes() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 4);

    // entry 0: func 0x60, params=[i32, (ref func), (ref null struct)], results=[(ref typeidx:5)]
    //   0x7F            = i32
    //   0x64 0x70       = (ref func)
    //   0x63 0x6B       = (ref null struct)
    buf.push(0x60);
    push_uleb128(&mut buf, 3);
    buf.extend_from_slice(&[0x7F, 0x64, 0x70, 0x63, 0x6B]);
    push_uleb128(&mut buf, 1);
    buf.extend_from_slice(&[0x64, 0x05]);

    // entry 1: struct 0x5F, fields=[((ref extern), mutable), (i32, immutable)]
    //   0x64 0x6F       = (ref extern)
    buf.push(0x5F);
    push_uleb128(&mut buf, 2);
    buf.extend_from_slice(&[0x64, 0x6F, 0x01]);
    buf.extend_from_slice(&[0x7F, 0x00]);

    // entry 2: array 0x61, element=(ref null any), mutable=true
    //   0x63 0x6E       = (ref null any)
    buf.push(0x61);
    buf.extend_from_slice(&[0x63, 0x6E, 0x01]);

    // entry 3: 普通 func 0x60, params=[i32], results=[]（验证游标未错位）
    buf.push(0x60);
    push_uleb128(&mut buf, 1);
    buf.push(0x7F);
    push_uleb128(&mut buf, 0);

    buf
}

#[test]
fn parse_type_section_多字节reftype的func() {
    let payload = build_type_payload_with_reftypes();
    let module = module_with_section(SECTION_TYPE, payload);
    let entries = parse_type_section(&module);

    assert_eq!(entries.len(), 4);

    let func_entry = &entries[0];
    match &func_entry.kind {
        WasmTypeKind::Func { params, results } => {
            assert_eq!(params.len(), 3);
            assert_eq!(params[0], WasmValueType::I32);
            assert_eq!(params[1], WasmValueType::Ref(WasmHeapType::Func));
            assert_eq!(params[2], WasmValueType::RefNull(WasmHeapType::Struct));
            assert_eq!(results.len(), 1);
            assert_eq!(results[0], WasmValueType::Ref(WasmHeapType::Index(5)));
        }
        other => panic!("entry 0 应为 Func，实际：{:?}", other),
    }
}

#[test]
fn parse_type_section_多字节reftype的struct() {
    let payload = build_type_payload_with_reftypes();
    let module = module_with_section(SECTION_TYPE, payload);
    let entries = parse_type_section(&module);

    let struct_entry = &entries[1];
    match &struct_entry.kind {
        WasmTypeKind::Struct { fields } => {
            assert_eq!(fields.len(), 2);
            assert_eq!(fields[0], (WasmValueType::Ref(WasmHeapType::Extern), true));
            assert_eq!(fields[1], (WasmValueType::I32, false));
        }
        other => panic!("entry 1 应为 Struct，实际：{:?}", other),
    }
}

#[test]
fn parse_type_section_多字节reftype的array() {
    let payload = build_type_payload_with_reftypes();
    let module = module_with_section(SECTION_TYPE, payload);
    let entries = parse_type_section(&module);

    let array_entry = &entries[2];
    match &array_entry.kind {
        WasmTypeKind::Array { element, mutable } => {
            assert_eq!(*element, WasmValueType::RefNull(WasmHeapType::Any));
            assert!(*mutable);
        }
        other => panic!("entry 2 应为 Array，实际：{:?}", other),
    }
}

#[test]
fn parse_type_section_多字节reftype后游标未错位() {
    let payload = build_type_payload_with_reftypes();
    let module = module_with_section(SECTION_TYPE, payload);
    let entries = parse_type_section(&module);

    // 最后一个条目应为普通 func(i32)->()，证明前面多字节 reftype 消费正确
    let trailing = &entries[3];
    match &trailing.kind {
        WasmTypeKind::Func { params, results } => {
            assert_eq!(*params, vec![WasmValueType::I32]);
            assert!(results.is_empty());
        }
        other => panic!("entry 3 应为 Func，实际：{:?}", other),
    }
}

/// 构建 import 段 payload：含 table import（elem 为 reftype）与 global import（value 为 reftype）。
fn build_import_payload_with_reftypes() -> Vec<u8> {
    let mut buf = Vec::new();
    push_uleb128(&mut buf, 2);

    // kind 1 (table): env.tbl elem=(ref func), flags=0, min=0
    //   0x64 0x70 = (ref func)
    push_string(&mut buf, "env");
    push_string(&mut buf, "tbl");
    buf.push(0x01);
    buf.extend_from_slice(&[0x64, 0x70]);
    buf.push(0x00);
    push_uleb128(&mut buf, 0);

    // kind 3 (global): env.glb valtype=(ref null struct), mutable=1
    //   0x63 0x6B = (ref null struct)
    push_string(&mut buf, "env");
    push_string(&mut buf, "glb");
    buf.push(0x03);
    buf.extend_from_slice(&[0x63, 0x6B]);
    buf.push(0x01);

    buf
}

#[test]
fn parse_import_section_table_elem为多字节reftype() {
    let payload = build_import_payload_with_reftypes();
    let module = module_with_section(SECTION_IMPORT, payload);
    let imports = parse_import_section(&module);

    assert_eq!(imports.len(), 2);
    let table = &imports[0];
    assert_eq!(table.kind, WasmExternalKind::Table);
    assert_eq!(table.table_elem_type, WasmValueType::Ref(WasmHeapType::Func));
    assert_eq!(table.memory_min, 0);
    assert_eq!(table.memory_max, None);
}

#[test]
fn parse_import_section_global_value为多字节reftype且游标未错位() {
    let payload = build_import_payload_with_reftypes();
    let module = module_with_section(SECTION_IMPORT, payload);
    let imports = parse_import_section(&module);

    // table 已正确消费多字节 elem type，global 才能解析出来
    let global = &imports[1];
    assert_eq!(global.kind, WasmExternalKind::Global);
    assert_eq!(global.global_value_type, WasmValueType::RefNull(WasmHeapType::Struct));
    assert!(global.global_mutable);
}
