//! `PE/CLR` 瑙ｆ瀽灞傛祴璇曘€?//!
//! 瑕嗙洊 [`std_data::binary::pe`] 鐨?`PE` 瑙ｆ瀽銆佸厓鏁版嵁琛ㄨ澶у皬璁＄畻銆?//! `CIL` 鎿嶄綔鐮佽〃鏌ユ壘涓庢柟娉曚綋瑙ｇ爜锛坄Tiny/Fat` 鍙屾牸寮忥級銆?
use std_data::{
    binary::pe::{
        CodedIndex, MetadataRoot, PeImage, PeParseError, PeWriter, PeWriterOptions, TableKind, UserStringsHeap, coded_index_size,
        heap_index_size, parse_method_body_il, parse_pe, read_blob, read_compressed_uint, read_strings_string, read_user_string,
        table_data_offset, table_row_size,
    },
    text::msil::{
        IlOperand, MsilAssembly, MsilInstruction, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule, MsilOpcode, MsilType,
        OP_CODES, OperandType, get_operand_type, lookup_opcode,
    },
};

/// 鏋勯€犱竴涓粎鍚?`Module` 琛紙1 琛岋級鐨?`MetadataRoot`锛屽爢鍧囦笉瓒?`2^16` 鏁呯储寮曚负 2 瀛楄妭銆?
fn sample_metadata_root() -> MetadataRoot {
    let mut row_counts = [0u32; 64];
    row_counts[TableKind::Module as usize] = 1;
    let valid_tables = 1u64 << TableKind::Module as u8;
    MetadataRoot {
        major_version: 1,
        minor_version: 1,
        version: "v4.0.30319".to_string(),
        streams: Vec::new(),
        strings: Vec::new(),
        user_strings: Vec::new(),
        guid: Vec::new(),
        blob: Vec::new(),
        tables: Vec::new(),
        row_counts,
        valid_tables,
        sorted_tables: 0,
    }
}

#[test]
fn test_opcode_table_lookup() {
    // nop (0x00)
    let info = lookup_opcode(0x00).expect("nop should exist");
    assert_eq!(info.opcode, "nop");
    assert!(!info.has_operand);
    assert_eq!(info.operand_size, 0);

    // ret (0x2A)
    let info = lookup_opcode(0x2A).expect("ret should exist");
    assert_eq!(info.opcode, "ret");
    assert!(!info.has_operand);

    // ldc.i4.1 (0x17) 鏃犳搷浣滄暟
    let info = lookup_opcode(0x17).expect("ldc.i4.1 should exist");
    assert_eq!(info.opcode, "ldc.i4.1");
    assert!(!info.has_operand);

    // call (0x28) 鎼哄甫 MethodToken
    let info = lookup_opcode(0x28).expect("call should exist");
    assert_eq!(info.opcode, "call");
    assert!(info.has_operand);
    assert_eq!(info.operand_size, 4);
}

#[test]
fn test_op_codes_static_map_accessible() {
    // OP_CODES 涓?LazyLock锛岃闂‘璁ゅ垵濮嬪寲涓?panic銆?
    let info = OP_CODES.get(&0x00);
    assert!(info.is_some());
    // 琛ㄨ妯″悎鐞嗭紙ECMA-335 鏍囧噯鎿嶄綔鐮佺害 158 鏉＄洰锛夈€?
    assert!(OP_CODES.len() >= 150);
}

#[test]
fn test_get_operand_type_single_byte() {
    // call (0x28) -> MethodToken
    assert!(matches!(get_operand_type(0x28), OperandType::MethodToken));
    // ldstr (0x72) -> StringToken锛堜慨澶嶄簡鍘熷疄鐜拌褰?FieldToken 鐨?bug锛?
    assert!(matches!(get_operand_type(0x72), OperandType::StringToken));
    // ldsfld (0x7E) -> FieldToken
    assert!(matches!(get_operand_type(0x7E), OperandType::FieldToken));
    // br.s (0x2B) -> ShortBrTarget
    assert!(matches!(get_operand_type(0x2B), OperandType::ShortBrTarget));
    // br (0x38) -> BrTarget
    assert!(matches!(get_operand_type(0x38), OperandType::BrTarget));
    // nop (0x00) -> None
    assert!(matches!(get_operand_type(0x00), OperandType::None));
}

#[test]
fn test_get_operand_type_double_byte_token() {
    // ldftn (0xFE06) -> MethodToken锛堝弻瀛楄妭锛屽師瀹炵幇鏈瘑鍒級
    assert!(matches!(get_operand_type(0xFE06), OperandType::MethodToken));
    // ldvirtftn (0xFE07) -> MethodToken
    assert!(matches!(get_operand_type(0xFE07), OperandType::MethodToken));
    // initobj (0xFE15) -> TypeToken
    assert!(matches!(get_operand_type(0xFE15), OperandType::TypeToken));
    // constrained. (0xFE16) -> TypeToken
    assert!(matches!(get_operand_type(0xFE16), OperandType::TypeToken));
    // sizeof (0xFE1C) -> TypeToken
    assert!(matches!(get_operand_type(0xFE1C), OperandType::TypeToken));
    // ldtoken (0xD0) -> Token
    assert!(matches!(get_operand_type(0xD0), OperandType::Token));
}

#[test]
fn test_parse_tiny_method_body() {
    // Tiny 鏂规硶浣擄紙宸插墺绂诲ご閮級锛歯op + ret
    let code = [0x00u8, 0x2A];
    let instrs = parse_method_body_il(&code, true);
    assert_eq!(instrs.len(), 2);
    assert_eq!(instrs[0].offset, 0);
    assert_eq!(instrs[0].opcode, "nop");
    assert!(matches!(instrs[0].operand, IlOperand::None));
    assert_eq!(instrs[1].offset, 1);
    assert_eq!(instrs[1].opcode, "ret");
}

#[test]
fn test_parse_tiny_with_ldc_and_branch() {
    // ldc.i4.1 + br.s +0 + ret锛氶獙璇佺煭鍒嗘敮鐩爣涓虹粷瀵瑰亸绉汇€?    // br.s 鐨勬搷浣滄暟涓?1 瀛楄妭鏈夌鍙峰閲忥紝鐩爣 = il_offset(2) + size(1) + delta(0) = 3
    let code = [0x17u8, 0x2B, 0x00, 0x2A];
    let instrs = parse_method_body_il(&code, true);
    assert_eq!(instrs.len(), 3);
    assert_eq!(instrs[0].opcode, "ldc.i4.1");
    assert_eq!(instrs[1].opcode, "br.s");
    match &instrs[1].operand {
        IlOperand::ShortBrTarget(target) => assert_eq!(*target, 3),
        other => panic!("expected ShortBrTarget, got {other:?}"),
    }
}

#[test]
fn test_parse_fat_method_body() {
    // Fat 鏂规硶浣擄細12 瀛楄妭澶?+ nop + ret
    // Flags: 0x0303 (Fat + 12 瀛楄妭澶?+ InitLocals)锛孧axStack: 0锛孋odeLen: 2锛孡ocalVarSigTok: 0
    let mut data = vec![0u8; 12];
    data[0] = 0x03;
    data[1] = 0x03;
    // MaxStack = 0
    data[2] = 0;
    data[3] = 0;
    // CodeLen = 2
    data[4] = 2;
    data[5] = 0;
    data[6] = 0;
    data[7] = 0;
    // LocalVarSigTok = 0
    data[8..12].copy_from_slice(&0u32.to_le_bytes());
    // IL: nop + ret
    data.push(0x00);
    data.push(0x2A);

    let instrs = parse_method_body_il(&data, false);
    assert_eq!(instrs.len(), 2);
    assert_eq!(instrs[0].opcode, "nop");
    assert_eq!(instrs[1].opcode, "ret");
    assert_eq!(instrs[0].offset, 0);
    assert_eq!(instrs[1].offset, 1);
}

#[test]
fn test_parse_fat_body_with_call() {
    // Fat 鏂规硶浣撳惈 call 鎸囦护锛氶獙璇?token 鎿嶄綔鏁拌В鐮併€?
    let mut data = vec![0u8; 12];
    data[0] = 0x03;
    data[1] = 0x03;
    // CodeLen = 6 (call 1 + 4 operand + ret 1)
    data[4] = 6;
    data[5] = 0;
    data[6] = 0;
    data[7] = 0;
    // IL: call <token 0x06000001> + ret
    data.push(0x28);
    data.extend_from_slice(&0x0600_0001u32.to_le_bytes());
    data.push(0x2A);

    let instrs = parse_method_body_il(&data, false);
    assert_eq!(instrs.len(), 2);
    assert_eq!(instrs[0].opcode, "call");
    match &instrs[0].operand {
        IlOperand::MethodToken(token) => assert_eq!(*token, 0x0600_0001),
        other => panic!("expected MethodToken, got {other:?}"),
    }
}

#[test]
fn test_parse_method_body_empty() {
    let instrs = parse_method_body_il(&[], true);
    assert!(instrs.is_empty());
}

#[test]
fn test_read_strings_string() {
    // #Strings 鍫嗭細浠?null 缁堟鐨?UTF-8 瀛楃涓层€?
    let heap = b"\0Main\0Exit\0";
    assert_eq!(read_strings_string(heap, 0), "");
    assert_eq!(read_strings_string(heap, 1), "Main");
    assert_eq!(read_strings_string(heap, 6), "Exit");
}

#[test]
fn test_read_blob() {
    // #Blob 鍫嗭細鍘嬬缉闀垮害鍓嶇紑 + 鏁版嵁銆?    // [0x03, 0xAA, 0xBB, 0xCC] 琛ㄧず 3 瀛楄妭 blob
    let heap = [0x03u8, 0xAA, 0xBB, 0xCC];
    let blob = read_blob(&heap, 0);
    assert_eq!(blob, vec![0xAA, 0xBB, 0xCC]);
}

#[test]
fn test_read_user_string() {
    // #US 堆：压缩长度前缀 + UTF-16LE + 1 字节尾标志。
    // "Hi" = [0x48, 0x00, 0x69, 0x00]，长度 = 4 + 1(尾) = 5，压缩为 0x05
    let heap = [0x05u8, 0x48, 0x00, 0x69, 0x00, 0x00];
    let s = read_user_string(&heap, 0);
    assert_eq!(s, "Hi");
}

#[test]
fn test_user_strings_heap_supplementary_plane_flag() {
    let mut heap = UserStringsHeap::new();
    // BMP ASCII：尾字节应为 0
    let off_hi = heap.add("Hi");
    let data = heap.data();
    let (len, len_bytes) = read_compressed_uint(data, off_hi as usize);
    let flag = data[off_hi as usize + len_bytes + len as usize - 1];
    assert_eq!(flag, 0, "BMP ASCII must clear #US trailing flag");
    assert_eq!(read_user_string(data, off_hi), "Hi");

    // 补充平面（U+1F600 😀 → UTF-16 代理对 ≥ 0x8000）：尾字节应为 1
    let mut heap2 = UserStringsHeap::new();
    let off_emoji = heap2.add("😀");
    let data2 = heap2.data();
    let (len2, len_bytes2) = read_compressed_uint(data2, off_emoji as usize);
    let flag2 = data2[off_emoji as usize + len_bytes2 + len2 as usize - 1];
    assert_eq!(flag2, 1, "supplementary-plane chars must set #US trailing flag");
    assert_eq!(read_user_string(data2, off_emoji), "😀");
}

#[test]
fn test_read_compressed_uint() {
    // 鍗曞瓧鑺傦細0x00-0x7F -> 鍊煎嵆鏈韩
    assert_eq!(read_compressed_uint(&[0x05], 0), (5, 1));
    assert_eq!(read_compressed_uint(&[0x7F], 0), (0x7F, 1));
    // 鍙屽瓧鑺傦細0x80-0xBF -> (b0 & 0x3F) << 8 | b1
    assert_eq!(read_compressed_uint(&[0x80, 0x01], 0), (1, 2));
    // 鍥涘瓧鑺傦細0xC0-0xFF -> (b0 & 0x1F) << 24 | b1 << 16 | b2 << 8 | b3
    assert_eq!(read_compressed_uint(&[0xC0, 0x00, 0x01, 0x00], 0), (0x100, 4));
}

#[test]
fn test_heap_index_size() {
    assert_eq!(heap_index_size(0), 2);
    assert_eq!(heap_index_size(0xFFFF), 2);
    assert_eq!(heap_index_size(0x1_0000), 4);
}

#[test]
fn test_coded_index_size() {
    let md = sample_metadata_root();
    // TypeDefOrRef 寮曠敤 TypeDef(1 琛?銆乀ypeRef(0 琛?銆乀ypeSpec(0 琛?锛宮ax=1锛? 瀛楄妭瓒冲銆?
    let size = coded_index_size(&md, CodedIndex::TypeDefOrRef);
    assert_eq!(size, 2);
    // MemberRefParent 寮曠敤 TypeDef(1)銆乀ypeRef(0)銆丮oduleRef(0)銆丮ethodDef(0)銆乀ypeSpec(0)锛宮ax=1銆?
    let size = coded_index_size(&md, CodedIndex::MemberRefParent);
    assert_eq!(size, 2);
}

#[test]
fn test_table_row_size_module() {
    let md = sample_metadata_root();
    // Module 琛ㄨ锛欸eneration(2) + Name(strings 2) + Mvid(guid 2) + EncId(guid 2) + EncBaseId(guid 2) = 10
    let size = table_row_size(&md, TableKind::Module as u8);
    assert_eq!(size, Some(10));
}

#[test]
fn test_table_row_size_typeref_formula() {
    let md = sample_metadata_root();
    // table_row_size 浠呰繑鍥炲崟琛屽叕寮忥紝涓嶆鏌ヨ〃鏄惁瀛樺湪銆?    // TypeRef 琛?= ResolutionScope(coded 2) + Name(2) + Namespace(2) = 6
    let size = table_row_size(&md, TableKind::TypeRef as u8);
    assert_eq!(size, Some(6));
}

#[test]
fn test_table_data_offset_absent_table() {
    let md = sample_metadata_root();
    // TypeRef 琛ㄤ笉瀛樺湪浜?valid_tables锛宼able_data_offset 搴旇繑鍥?None銆?
    let offset = table_data_offset(&md, TableKind::TypeRef as u8);
    assert_eq!(offset, None);
}

#[test]
fn test_table_data_offset_module() {
    let md = sample_metadata_root();
    // 浠?Module 琛ㄥ瓨鍦ㄤ笖涓虹涓€寮犺〃锛屽亸绉诲簲涓?0銆?
    let offset = table_data_offset(&md, TableKind::Module as u8);
    assert_eq!(offset, Some(0));
}

#[test]
fn test_parse_pe_too_short() {
    assert!(matches!(parse_pe(&[]), Err(PeParseError::TooShort)));
    assert!(matches!(parse_pe(&[0u8; 10]), Err(PeParseError::TooShort)));
}

#[test]
fn test_parse_pe_bad_mz() {
    // 瓒冲闀夸絾鏃?MZ 榄旀暟銆?
    let data = [0xFFu8; 64];
    assert!(matches!(parse_pe(&data), Err(PeParseError::BadMzMagic)));
}

#[test]
fn test_parse_pe_bad_signature() {
    // 鏈?MZ 浣?PE 绛惧悕閿欒銆?
    let mut data = vec![0u8; 128];
    data[0] = b'M';
    data[1] = b'Z';
    // pe_offset = 0x40
    data[0x3C] = 0x40;
    data[0x3D] = 0x00;
    data[0x3E] = 0x00;
    data[0x3F] = 0x00;
    // 0x40 澶勯潪 "PE\0\0"
    assert!(matches!(parse_pe(&data), Err(PeParseError::BadPeSignature)));
}

#[test]
fn test_parse_pe_roundtrip_with_writer() {
    // 鏋勯€犳渶灏?MsilModule锛? 涓叏灞€ Main 鏂规硶锛屼粎 ret銆?
    let module = MsilModule {
        assembly: MsilAssembly { name: "TestAsm".to_string(), externs: Vec::new() },
        types: Vec::new(),
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "Main".to_string(), signature: MsilMethodSignature::new(MsilType::Void, Vec::new()) },
            locals: Vec::new(),
            instructions: vec![MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None }],
            max_stack: 0,
            is_entry_point: true,
            is_async: false,
        }],
    };

    let options = PeWriterOptions {
        assembly_name: "TestAsm".to_string(),
        module_name: "TestModule".to_string(),
        image_kind: nyar::backends::clr::ClrImageKind::Executable,
    };
    let bytes = PeWriter::new(options).write_module(&module).expect("PE write should succeed");

    // 瑙ｆ瀽鐢熸垚鐨?PE銆?
    let image: PeImage = parse_pe(&bytes).expect("PE parse should succeed");
    assert_eq!(image.dos.pe_offset, 0x40);
    assert_eq!(image.coff.number_of_sections, 2);
    // CLI 澶翠笌鍏冩暟鎹簲瀛樺湪銆?
    let cli = image.cli.expect("should have CLI header");
    assert!(cli.metadata_rva != 0);
    assert!(cli.entry_point_token != 0);
    let metadata = image.metadata.expect("should have metadata root");
    // 娴佸簲鍖呭惈 #Strings / #US / #GUID / #Blob / #~銆?
    let stream_names: Vec<&str> = metadata.streams.iter().map(|s| s.name.as_str()).collect();
    assert!(stream_names.contains(&"#Strings"), "should contain #Strings stream, actual: {stream_names:?}");
    assert!(stream_names.contains(&"#~"), "should contain #~ stream, actual: {stream_names:?}");

    // Module 琛ㄥ簲瀛樺湪涓旇鏁颁负 1銆?
    assert_eq!(metadata.row_counts[TableKind::Module as usize], 1);
    // MethodDef 琛ㄥ簲瀛樺湪涓旇鏁颁负 1銆?
    assert_eq!(metadata.row_counts[TableKind::MethodDef as usize], 1);
    // Assembly 琛ㄥ簲瀛樺湪涓旇鏁颁负 1銆?
    assert_eq!(metadata.row_counts[TableKind::Assembly as usize], 1);

    // table_row_size 鍦ㄧ湡瀹炲厓鏁版嵁涓婂簲杩斿洖鍚堢悊鍊笺€?
    let module_row = table_row_size(&metadata, TableKind::Module as u8);
    assert!(module_row.is_some(), "Module 琛ㄨ澶у皬搴斿彲璁＄畻");
    let methoddef_row = table_row_size(&metadata, TableKind::MethodDef as u8);
    assert!(methoddef_row.is_some(), "MethodDef 琛ㄨ澶у皬搴斿彲璁＄畻");

    // table_data_offset 瀵?Module 琛ㄥ簲杩斿洖 0锛堢涓€寮犺〃锛夈€?
    let module_offset = table_data_offset(&metadata, TableKind::Module as u8);
    assert_eq!(module_offset, Some(0));
}

#[test]
fn write_scaled_main_exe_for_entrypoint_smoke() {
    use nyar::backends::clr::ClrImageKind;
    use std_data::{
        binary::pe::{PeWriter, PeWriterOptions},
        text::msil::*,
    };

    let mut global_methods = Vec::new();
    global_methods.push(MsilMethodBody {
        method: MsilMethodRef {
            owner: None,
            name: "Main".to_string(),
            signature: MsilMethodSignature::new(MsilType::Int32 { signed: true }, vec![MsilType::sz_array(MsilType::String)]),
        },
        locals: Vec::new(),
        instructions: vec![
            MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
            MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
        ],
        max_stack: 8,
        is_entry_point: true,
        is_async: false,
    });
    for index in 0..440 {
        global_methods.push(MsilMethodBody {
            method: MsilMethodRef {
                owner: None,
                name: format!("stub_{index}"),
                signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
            },
            locals: Vec::new(),
            instructions: vec![MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None }],
            max_stack: 0,
            is_entry_point: false,
            is_async: false,
        });
    }

    let module =
        MsilModule { assembly: MsilAssembly { name: "ScaledAsm".to_string(), externs: Vec::new() }, types: Vec::new(), global_methods };

    let options = PeWriterOptions {
        assembly_name: "ScaledAsm".to_string(),
        module_name: "ScaledModule".to_string(),
        image_kind: ClrImageKind::Executable,
    };
    let bytes = PeWriter::new(options).write_module(&module).expect("PE write should succeed");
    let image = parse_pe(&bytes).expect("PE parse should succeed");
    assert_eq!(image.coff.number_of_sections, 2);
    let cli = image.cli.expect("should have CLI header");
    assert!(cli.metadata_rva != 0);
    assert!(cli.entry_point_token != 0);
    let metadata = image.metadata.expect("should have metadata root");
    assert_eq!(metadata.row_counts[TableKind::MethodDef as usize], 441);
}
