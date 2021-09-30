//! legion spy jvm 子模式：JVM class 文件解析与字节码反汇编。
//!
//! 支持 `.class` 和 `.jar` 文件，可列出方法签名、反汇编指定方法体。
//! 用于定位 JVM 字节码验证错误。

use std::{collections::BTreeMap, fs, path::Path, process::ExitCode};

use miette::{miette, IntoDiagnostic, Result};

use super::{SpyOptions, SpyTargetOptions};

/// 执行 JVM 字节码 dump。
///
/// 支持的文件扩展名：`.class` / `.jar`。
pub fn run(options: &SpyOptions) -> Result<ExitCode> {
    let (_, options) = options.split();
    let Some(target) = &options.input
    else {
        return Err(miette!(
            "用法：legion spy jvm <file> [--method <name>] [--list] [--json]\n  file               目标文件（.class / .jar）\n  --method <name>    输出包含指定名称的方法体\n  --list             列出所有方法签名\n  --json             以 JSON 格式输出"
        ));
    };

    if !Path::exists(Path::new(target)) {
        return Err(miette!("文件不存在：{}", target));
    }

    let extension = Path::new(target).extension().and_then(|ext| ext.to_str()).map(|ext| ext.to_ascii_lowercase()).unwrap_or_default();

    match extension.as_str() {
        "class" => run_class_file(target, options),
        "jar" => run_jar_file(target, options),
        other => Err(miette!("不支持的文件扩展名 '{}'，支持 .class / .jar", other)),
    }
}

/// 分析单个 `.class` 文件。
fn run_class_file(target: &str, options: &SpyTargetOptions) -> Result<ExitCode> {
    let data = fs::read(target).into_diagnostic().map_err(|error| error.wrap_err(format!("无法读取文件 {}", target)))?;
    let class = ClassFileParser::parse(&data)?;
    output_class(&class, options);
    Ok(ExitCode::SUCCESS)
}

/// 分析 `.jar` 文件，遍历其中所有 `.class` 条目。
fn run_jar_file(target: &str, options: &SpyTargetOptions) -> Result<ExitCode> {
    let data = fs::read(target).into_diagnostic().map_err(|error| error.wrap_err(format!("无法读取文件 {}", target)))?;
    let entries = parse_jar_entries(&data)?;
    for (name, class_data) in entries {
        if options.method.is_some() {
            // 指定了方法名时，只输出包含该方法名的类。
            if let Ok(class) = ClassFileParser::parse(&class_data) {
                if class.methods.iter().any(|m| m.name.contains(options.method.as_deref().unwrap_or(""))) {
                    println!("=== {} ===", name);
                    output_class(&class, options);
                }
            }
        }
        else if options.list {
            if let Ok(class) = ClassFileParser::parse(&class_data) {
                println!("=== {} ===", name);
                output_class(&class, options);
            }
        }
        else {
            // 默认只输出主类的方法列表。
            if let Ok(class) = ClassFileParser::parse(&class_data) {
                println!("=== {} ===", name);
                output_class(&class, options);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// 输出 class 文件信息。
fn output_class(class: &ClassFile, options: &SpyTargetOptions) {
    println!("class: {}", class.this_class);
    println!("super: {}", class.super_class);
    println!("version: {}.{}", class.major_version, class.minor_version);
    println!("methods: {}", class.methods.len());

    if options.list {
        println!("\n--- 方法列表 ---");
        for method in &class.methods {
            println!("  {}{}", method.name, method.descriptor);
        }
        return;
    }

    if let Some(method_name) = &options.method {
        for method in &class.methods {
            if method.name.contains(method_name) {
                println!("\n--- {}{} ---", method.name, method.descriptor);
                if let Some(code) = &method.code {
                    println!("max_stack: {}, max_locals: {}", code.max_stack, code.max_locals);
                    println!("code length: {} bytes", code.code.len());
                    disassemble(&code.code, &class.constant_pool);
                }
                else {
                    println!("(无方法体 - native 或 abstract)");
                }
            }
        }
        return;
    }

    // 默认输出所有方法签名。
    println!("\n--- 方法签名 ---");
    for method in &class.methods {
        let flags = format_access_flags(method.access_flags);
        println!("  {} {}{}", flags, method.name, method.descriptor);
    }
}

/// 格式化访问标志。
fn format_access_flags(flags: u16) -> String {
    let mut parts = Vec::new();
    if flags & 0x0001 != 0 {
        parts.push("public");
    }
    if flags & 0x0002 != 0 {
        parts.push("private");
    }
    if flags & 0x0004 != 0 {
        parts.push("protected");
    }
    if flags & 0x0008 != 0 {
        parts.push("static");
    }
    if flags & 0x0010 != 0 {
        parts.push("final");
    }
    if flags & 0x0400 != 0 {
        parts.push("abstract");
    }
    if flags & 0x0100 != 0 {
        parts.push("native");
    }
    parts.join(" ")
}

// ============ Class 文件解析 ============

/// 解析后的 class 文件。
struct ClassFile {
    minor_version: u16,
    major_version: u16,
    access_flags: u16,
    this_class: String,
    super_class: String,
    constant_pool: ConstantPool,
    methods: Vec<MethodInfo>,
}

/// 方法信息。
struct MethodInfo {
    access_flags: u16,
    name: String,
    descriptor: String,
    code: Option<CodeAttribute>,
}

/// Code 属性。
struct CodeAttribute {
    max_stack: u16,
    max_locals: u16,
    code: Vec<u8>,
}

/// 常量池。
struct ConstantPool {
    entries: Vec<ConstantEntry>,
}

/// 常量池条目。
#[allow(dead_code)]
enum ConstantEntry {
    Utf8(String),
    Class(u16),
    Methodref { class_index: u16, name_and_type_index: u16 },
    String(u16),
    Integer(i32),
    NameAndType { name_index: u16, descriptor_index: u16 },
    Padding,
}

impl ConstantPool {
    /// 获取 UTF8 常量。
    fn utf8(&self, index: u16) -> Option<&str> {
        self.entries.get(index as usize).and_then(|e| match e {
            ConstantEntry::Utf8(s) => Some(s.as_str()),
            _ => None,
        })
    }

    /// 解析 Methodref 为 "owner.name descriptor" 格式。
    fn methodref(&self, index: u16) -> String {
        match self.entries.get(index as usize) {
            Some(ConstantEntry::Methodref { class_index, name_and_type_index }) => {
                let class_name = self.class_name(*class_index).unwrap_or_else(|| "?".to_string());
                let nat = match self.entries.get(*name_and_type_index as usize) {
                    Some(ConstantEntry::NameAndType { name_index, descriptor_index }) => {
                        let name = self.utf8(*name_index).unwrap_or("?");
                        let desc = self.utf8(*descriptor_index).unwrap_or("?");
                        format!("{}.{}{}", class_name, name, desc)
                    }
                    _ => "?".to_string(),
                };
                nat
            }
            _ => "?".to_string(),
        }
    }

    /// 获取类名。
    fn class_name(&self, index: u16) -> Option<String> {
        match self.entries.get(index as usize) {
            Some(ConstantEntry::Class(name_index)) => self.utf8(*name_index).map(|s| s.to_string()),
            _ => None,
        }
    }

    /// 获取 String 常量的值。
    fn string_value(&self, index: u16) -> Option<String> {
        match self.entries.get(index as usize) {
            Some(ConstantEntry::String(utf8_index)) => self.utf8(*utf8_index).map(|s| s.to_string()),
            _ => None,
        }
    }
}

/// Class 文件解析器。
struct ClassFileParser<'a> {
    data: &'a [u8],
    offset: usize,
}

impl<'a> ClassFileParser<'a> {
    fn parse(data: &'a [u8]) -> Result<ClassFile> {
        let mut parser = Self { data, offset: 0 };
        parser.parse_class()
    }

    fn parse_class(mut self) -> Result<ClassFile> {
        let magic = self.read_u32()?;
        if magic != 0xCAFEBABE {
            return Err(miette!("无效的 class 文件魔数：0x{:08X}", magic));
        }

        let minor_version = self.read_u16()?;
        let major_version = self.read_u16()?;
        let constant_pool = self.parse_constant_pool()?;
        let access_flags = self.read_u16()?;
        let this_class_index = self.read_u16()?;
        let super_class_index = self.read_u16()?;

        let this_class = constant_pool.class_name(this_class_index).unwrap_or_default();
        let super_class = constant_pool.class_name(super_class_index).unwrap_or_default();

        // 跳过接口。
        let interfaces_count = self.read_u16()?;
        for _ in 0..interfaces_count {
            self.read_u16()?;
        }

        // 跳过字段。
        let fields_count = self.read_u16()?;
        for _ in 0..fields_count {
            self.skip_member()?;
        }

        // 解析方法。
        let methods_count = self.read_u16()?;
        let mut methods = Vec::with_capacity(methods_count as usize);
        for _ in 0..methods_count {
            methods.push(self.parse_method(&constant_pool)?);
        }

        Ok(ClassFile { minor_version, major_version, access_flags, this_class, super_class, constant_pool, methods })
    }

    fn parse_constant_pool(&mut self) -> Result<ConstantPool> {
        let count = self.read_u16()? as usize;
        let mut entries = vec![ConstantEntry::Padding];

        while entries.len() < count {
            let tag = self.read_u8()?;
            let entry = match tag {
                1 => {
                    // UTF-8
                    let length = self.read_u16()? as usize;
                    let bytes = self.read_bytes(length)?;
                    let value = String::from_utf8(bytes.to_vec()).map_err(|_| miette!("无效的 UTF-8 常量"))?;
                    ConstantEntry::Utf8(value)
                }
                3 => {
                    // Integer
                    let value = self.read_u32()? as i32;
                    ConstantEntry::Integer(value)
                }
                4 => {
                    // Float
                    let _ = self.read_u32()?;
                    ConstantEntry::Padding
                }
                5 => {
                    // Long (占两个槽位)
                    let _ = self.read_u64()?;
                    entries.push(ConstantEntry::Padding);
                    ConstantEntry::Padding
                }
                6 => {
                    // Double (占两个槽位)
                    let _ = self.read_u64()?;
                    entries.push(ConstantEntry::Padding);
                    ConstantEntry::Padding
                }
                7 => {
                    // Class
                    let name_index = self.read_u16()?;
                    ConstantEntry::Class(name_index)
                }
                8 => {
                    // String
                    let utf8_index = self.read_u16()?;
                    ConstantEntry::String(utf8_index)
                }
                9 => {
                    // Fieldref
                    let class_index = self.read_u16()?;
                    let name_and_type_index = self.read_u16()?;
                    ConstantEntry::Methodref { class_index, name_and_type_index }
                }
                10 => {
                    // Methodref
                    let class_index = self.read_u16()?;
                    let name_and_type_index = self.read_u16()?;
                    ConstantEntry::Methodref { class_index, name_and_type_index }
                }
                11 => {
                    // InterfaceMethodref
                    let class_index = self.read_u16()?;
                    let name_and_type_index = self.read_u16()?;
                    ConstantEntry::Methodref { class_index, name_and_type_index }
                }
                12 => {
                    // NameAndType
                    let name_index = self.read_u16()?;
                    let descriptor_index = self.read_u16()?;
                    ConstantEntry::NameAndType { name_index, descriptor_index }
                }
                _ => {
                    return Err(miette!("不支持的常量池标签：{}", tag));
                }
            };
            entries.push(entry);
        }

        Ok(ConstantPool { entries })
    }

    fn parse_method(&mut self, constant_pool: &ConstantPool) -> Result<MethodInfo> {
        let access_flags = self.read_u16()?;
        let name_index = self.read_u16()?;
        let descriptor_index = self.read_u16()?;
        let name = constant_pool.utf8(name_index).unwrap_or("?").to_string();
        let descriptor = constant_pool.utf8(descriptor_index).unwrap_or("?").to_string();

        let attributes_count = self.read_u16()? as usize;
        let mut code = None;
        for _ in 0..attributes_count {
            let attr_name_index = self.read_u16()?;
            let attr_length = self.read_u32()? as usize;
            let attr_name = constant_pool.utf8(attr_name_index).unwrap_or("");
            if attr_name == "Code" {
                code = Some(self.parse_code_attribute()?);
            }
            else {
                self.skip(attr_length)?;
            }
        }

        Ok(MethodInfo { access_flags, name, descriptor, code })
    }

    fn parse_code_attribute(&mut self) -> Result<CodeAttribute> {
        let max_stack = self.read_u16()?;
        let max_locals = self.read_u16()?;
        let code_length = self.read_u32()? as usize;
        let code = self.read_bytes(code_length)?.to_vec();
        // 跳过异常表。
        let exception_table_length = self.read_u16()?;
        for _ in 0..exception_table_length {
            self.read_u16()?; // start_pc
            self.read_u16()?; // end_pc
            self.read_u16()?; // handler_pc
            self.read_u16()?; // catch_type
        }
        // 跳过 Code 属性的属性。
        let attributes_count = self.read_u16()?;
        for _ in 0..attributes_count {
            self.skip_member()?;
        }
        Ok(CodeAttribute { max_stack, max_locals, code })
    }

    fn skip_member(&mut self) -> Result<()> {
        self.read_u16()?;
        self.read_u16()?;
        self.read_u16()?;
        let attributes_count = self.read_u16()?;
        for _ in 0..attributes_count {
            self.read_u16()?;
            let length = self.read_u32()? as usize;
            self.skip(length)?;
        }
        Ok(())
    }

    fn read_u8(&mut self) -> Result<u8> {
        let value = *self.data.get(self.offset).ok_or_else(|| miette!("意外的文件结束"))?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64> {
        let bytes = self.read_bytes(8)?;
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(u64::from_be_bytes(arr))
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self.offset.checked_add(length).ok_or_else(|| miette!("意外的文件结束"))?;
        if end > self.data.len() {
            return Err(miette!("意外的文件结束"));
        }
        let bytes = &self.data[self.offset..end];
        self.offset = end;
        Ok(bytes)
    }

    fn skip(&mut self, length: usize) -> Result<()> {
        self.read_bytes(length).map(|_| ())
    }
}

// ============ 字节码反汇编 ============

/// 反汇编字节码并输出。
fn disassemble(code: &[u8], constant_pool: &ConstantPool) {
    let mut offset = 0usize;
    while offset < code.len() {
        let opcode = code[offset];
        let (mnemonic, size, operand_str) = decode_instruction(opcode, &code[offset..], constant_pool);
        println!("  {:4}: {:<12} {}", offset, mnemonic, operand_str);
        offset += size;
    }
}

/// 解码单条指令，返回 (助记符, 指令长度, 操作数字符串)。
fn decode_instruction(opcode: u8, rest: &[u8], constant_pool: &ConstantPool) -> (&'static str, usize, String) {
    match opcode {
        0x00 => ("nop", 1, String::new()),
        0x01 => ("aconst_null", 1, String::new()),
        0x02 => ("iconst_m1", 1, String::new()),
        0x03 => ("iconst_0", 1, String::new()),
        0x04 => ("iconst_1", 1, String::new()),
        0x05 => ("iconst_2", 1, String::new()),
        0x06 => ("iconst_3", 1, String::new()),
        0x07 => ("iconst_4", 1, String::new()),
        0x08 => ("iconst_5", 1, String::new()),
        0x09 => ("lconst_0", 1, String::new()),
        0x0A => ("lconst_1", 1, String::new()),
        0x0B => ("fconst_0", 1, String::new()),
        0x0C => ("fconst_1", 1, String::new()),
        0x0D => ("fconst_2", 1, String::new()),
        0x0E => ("dconst_0", 1, String::new()),
        0x0F => ("dconst_1", 1, String::new()),
        0x10 => ("bipush", 2, format!("{}", rest.get(1).copied().unwrap_or(0) as i8)),
        0x11 => ("sipush", 3, format!("{}", i16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]))),
        0x12 => ("ldc", 2, format_index(rest.get(1).copied().unwrap_or(0) as u16, constant_pool)),
        0x13 => (
            "ldc_w",
            3,
            format_index(u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]), constant_pool),
        ),
        0x14 => (
            "ldc2_w",
            3,
            format_index(u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]), constant_pool),
        ),
        // 加载指令
        0x15 => ("iload", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x16 => ("lload", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x17 => ("fload", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x18 => ("dload", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x19 => ("aload", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x1A => ("iload_0", 1, String::new()),
        0x1B => ("iload_1", 1, String::new()),
        0x1C => ("iload_2", 1, String::new()),
        0x1D => ("iload_3", 1, String::new()),
        0x1E => ("lload_0", 1, String::new()),
        0x1F => ("lload_1", 1, String::new()),
        0x20 => ("lload_2", 1, String::new()),
        0x21 => ("lload_3", 1, String::new()),
        0x22 => ("fload_0", 1, String::new()),
        0x23 => ("fload_1", 1, String::new()),
        0x24 => ("fload_2", 1, String::new()),
        0x25 => ("fload_3", 1, String::new()),
        0x26 => ("dload_0", 1, String::new()),
        0x27 => ("dload_1", 1, String::new()),
        0x28 => ("dload_2", 1, String::new()),
        0x29 => ("dload_3", 1, String::new()),
        0x2A => ("aload_0", 1, String::new()),
        0x2B => ("aload_1", 1, String::new()),
        0x2C => ("aload_2", 1, String::new()),
        0x2D => ("aload_3", 1, String::new()),
        // 数组加载
        0x2E => ("iaload", 1, String::new()),
        0x2F => ("laload", 1, String::new()),
        0x30 => ("faload", 1, String::new()),
        0x31 => ("daload", 1, String::new()),
        0x32 => ("aaload", 1, String::new()),
        0x33 => ("baload", 1, String::new()),
        0x34 => ("caload", 1, String::new()),
        0x35 => ("saload", 1, String::new()),
        // 存储指令
        0x36 => ("istore", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x37 => ("lstore", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x38 => ("fstore", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x39 => ("dstore", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x3A => ("astore", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0x3B => ("istore_0", 1, String::new()),
        0x3C => ("istore_1", 1, String::new()),
        0x3D => ("istore_2", 1, String::new()),
        0x3E => ("istore_3", 1, String::new()),
        0x3F => ("lstore_0", 1, String::new()),
        0x40 => ("lstore_1", 1, String::new()),
        0x41 => ("lstore_2", 1, String::new()),
        0x42 => ("lstore_3", 1, String::new()),
        0x43 => ("fstore_0", 1, String::new()),
        0x44 => ("fstore_1", 1, String::new()),
        0x45 => ("fstore_2", 1, String::new()),
        0x46 => ("fstore_3", 1, String::new()),
        0x47 => ("dstore_0", 1, String::new()),
        0x48 => ("dstore_1", 1, String::new()),
        0x49 => ("dstore_2", 1, String::new()),
        0x4A => ("dstore_3", 1, String::new()),
        0x4B => ("astore_0", 1, String::new()),
        0x4C => ("astore_1", 1, String::new()),
        0x4D => ("astore_2", 1, String::new()),
        0x4E => ("astore_3", 1, String::new()),
        // 数组存储
        0x4F => ("iastore", 1, String::new()),
        0x50 => ("lastore", 1, String::new()),
        0x51 => ("fastore", 1, String::new()),
        0x52 => ("dastore", 1, String::new()),
        0x53 => ("aastore", 1, String::new()),
        0x54 => ("bastore", 1, String::new()),
        0x55 => ("castore", 1, String::new()),
        0x56 => ("sastore", 1, String::new()),
        0x57 => ("pop", 1, String::new()),
        0x58 => ("pop2", 1, String::new()),
        0x59 => ("dup", 1, String::new()),
        0x5A => ("dup_x1", 1, String::new()),
        0x5B => ("dup_x2", 1, String::new()),
        0x5C => ("dup2", 1, String::new()),
        0x5D => ("dup2_x1", 1, String::new()),
        0x5E => ("dup2_x2", 1, String::new()),
        0x5F => ("swap", 1, String::new()),
        // 算术
        0x60 => ("iadd", 1, String::new()),
        0x61 => ("ladd", 1, String::new()),
        0x62 => ("fadd", 1, String::new()),
        0x63 => ("dadd", 1, String::new()),
        0x64 => ("isub", 1, String::new()),
        0x65 => ("lsub", 1, String::new()),
        0x66 => ("fsub", 1, String::new()),
        0x67 => ("dsub", 1, String::new()),
        0x68 => ("imul", 1, String::new()),
        0x69 => ("lmul", 1, String::new()),
        0x6A => ("fmul", 1, String::new()),
        0x6B => ("dmul", 1, String::new()),
        0x6C => ("idiv", 1, String::new()),
        0x6D => ("ldiv", 1, String::new()),
        0x6E => ("fdiv", 1, String::new()),
        0x6F => ("ddiv", 1, String::new()),
        0x70 => ("irem", 1, String::new()),
        0x71 => ("lrem", 1, String::new()),
        0x72 => ("frem", 1, String::new()),
        0x73 => ("drem", 1, String::new()),
        0x74 => ("ineg", 1, String::new()),
        0x75 => ("lneg", 1, String::new()),
        0x76 => ("fneg", 1, String::new()),
        0x77 => ("dneg", 1, String::new()),
        0x78 => ("ishl", 1, String::new()),
        0x79 => ("lshl", 1, String::new()),
        0x7A => ("ishr", 1, String::new()),
        0x7B => ("lshr", 1, String::new()),
        0x7C => ("iushr", 1, String::new()),
        0x7D => ("lushr", 1, String::new()),
        0x7E => ("iand", 1, String::new()),
        0x7F => ("land", 1, String::new()),
        0x80 => ("ior", 1, String::new()),
        0x81 => ("lor", 1, String::new()),
        0x82 => ("ixor", 1, String::new()),
        0x83 => ("lxor", 1, String::new()),
        // 转换
        0x85 => ("i2l", 1, String::new()),
        0x86 => ("i2f", 1, String::new()),
        0x87 => ("i2d", 1, String::new()),
        0x88 => ("l2i", 1, String::new()),
        0x89 => ("l2f", 1, String::new()),
        0x8A => ("l2d", 1, String::new()),
        0x8B => ("f2i", 1, String::new()),
        0x8C => ("f2l", 1, String::new()),
        0x8D => ("f2d", 1, String::new()),
        0x8E => ("d2i", 1, String::new()),
        0x8F => ("d2l", 1, String::new()),
        0x90 => ("d2f", 1, String::new()),
        0x91 => ("i2b", 1, String::new()),
        0x92 => ("i2c", 1, String::new()),
        0x93 => ("i2s", 1, String::new()),
        // 比较
        0x94 => ("lcmp", 1, String::new()),
        0x95 => ("fcmpl", 1, String::new()),
        0x96 => ("fcmpg", 1, String::new()),
        0x97 => ("dcmpl", 1, String::new()),
        0x98 => ("dcmpg", 1, String::new()),
        // 分支
        0x99 => ("ifeq", 3, format_branch(rest, 1)),
        0x9A => ("ifne", 3, format_branch(rest, 1)),
        0x9B => ("iflt", 3, format_branch(rest, 1)),
        0x9C => ("ifge", 3, format_branch(rest, 1)),
        0x9D => ("ifgt", 3, format_branch(rest, 1)),
        0x9E => ("ifle", 3, format_branch(rest, 1)),
        0x9F => ("if_icmpeq", 3, format_branch(rest, 1)),
        0xA0 => ("if_icmpne", 3, format_branch(rest, 1)),
        0xA1 => ("if_icmplt", 3, format_branch(rest, 1)),
        0xA2 => ("if_icmpge", 3, format_branch(rest, 1)),
        0xA3 => ("if_icmpgt", 3, format_branch(rest, 1)),
        0xA4 => ("if_icmple", 3, format_branch(rest, 1)),
        0xA5 => ("if_acmpeq", 3, format_branch(rest, 1)),
        0xA6 => ("if_acmpne", 3, format_branch(rest, 1)),
        0xA7 => ("goto", 3, format_branch(rest, 1)),
        0xA8 => ("jsr", 3, format_branch(rest, 1)),
        0xA9 => ("ret", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0xAA => ("tableswitch", 1, String::new()),
        0xAB => ("lookupswitch", 1, String::new()),
        // 返回
        0xAC => ("ireturn", 1, String::new()),
        0xAD => ("lreturn", 1, String::new()),
        0xAE => ("freturn", 1, String::new()),
        0xAF => ("dreturn", 1, String::new()),
        0xB0 => ("areturn", 1, String::new()),
        0xB1 => ("return", 1, String::new()),
        // 字段
        0xB2 => ("getstatic", 3, format_field(rest, constant_pool)),
        0xB3 => ("putstatic", 3, format_field(rest, constant_pool)),
        0xB4 => ("getfield", 3, format_field(rest, constant_pool)),
        0xB5 => ("putfield", 3, format_field(rest, constant_pool)),
        // 方法调用
        0xB6 => ("invokevirtual", 3, format_method(rest, constant_pool)),
        0xB7 => ("invokespecial", 3, format_method(rest, constant_pool)),
        0xB8 => ("invokestatic", 3, format_method(rest, constant_pool)),
        0xB9 => ("invokeinterface", 5, format_method(rest, constant_pool)),
        0xBA => (
            "invokedynamic",
            5,
            format_index(u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]), constant_pool),
        ),
        // 对象
        0xBB => ("new", 3, format_class_ref(rest, constant_pool)),
        0xBC => ("newarray", 2, format!("{}", rest.get(1).copied().unwrap_or(0))),
        0xBD => ("anewarray", 3, format_class_ref(rest, constant_pool)),
        0xBE => ("arraylength", 1, String::new()),
        0xBF => ("athrow", 1, String::new()),
        0xC0 => ("checkcast", 3, format_class_ref(rest, constant_pool)),
        0xC1 => ("instanceof", 3, format_class_ref(rest, constant_pool)),
        0xC2 => ("monitorenter", 1, String::new()),
        0xC3 => ("monitorexit", 1, String::new()),
        // wide
        0xC4 => ("wide", 4, String::new()),
        // multidim array
        0xC5 => ("multianewarray", 4, format_class_ref(rest, constant_pool)),
        0xC6 => ("ifnull", 3, format_branch(rest, 1)),
        0xC7 => ("ifnonnull", 3, format_branch(rest, 1)),
        0xC8 => ("goto_w", 5, format_branch_w(rest, 1)),
        0xC9 => ("jsr_w", 5, format_branch_w(rest, 1)),
        _ => ("unknown", 1, format!("0x{:02X}", opcode)),
    }
}

/// 格式化分支目标偏移。
fn format_branch(rest: &[u8], operand_offset: usize) -> String {
    let offset = i16::from_be_bytes([rest.get(operand_offset).copied().unwrap_or(0), rest.get(operand_offset + 1).copied().unwrap_or(0)]);
    format!("-> {}", offset)
}

/// 格式化宽分支目标偏移。
fn format_branch_w(rest: &[u8], operand_offset: usize) -> String {
    let bytes = [
        rest.get(operand_offset).copied().unwrap_or(0),
        rest.get(operand_offset + 1).copied().unwrap_or(0),
        rest.get(operand_offset + 2).copied().unwrap_or(0),
        rest.get(operand_offset + 3).copied().unwrap_or(0),
    ];
    let offset = i32::from_be_bytes(bytes);
    format!("-> {}", offset)
}

/// 格式化常量池索引引用。
fn format_index(index: u16, constant_pool: &ConstantPool) -> String {
    let value = constant_pool.string_value(index).unwrap_or_else(|| format!("#{}", index));
    format!("#{} ({})", index, value)
}

/// 格式化方法引用。
fn format_method(rest: &[u8], constant_pool: &ConstantPool) -> String {
    let index = u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]);
    let method = constant_pool.methodref(index);
    format!("#{} {}", index, method)
}

/// 格式化字段引用。
fn format_field(rest: &[u8], constant_pool: &ConstantPool) -> String {
    let index = u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]);
    format!("#{}", index)
}

/// 格式化类引用。
fn format_class_ref(rest: &[u8], constant_pool: &ConstantPool) -> String {
    let index = u16::from_be_bytes([rest.get(1).copied().unwrap_or(0), rest.get(2).copied().unwrap_or(0)]);
    let class_name = constant_pool.class_name(index).unwrap_or_else(|| format!("#{}", index));
    format!("#{} {}", index, class_name)
}

// ============ JAR 文件解析 ============

/// 解析 JAR 文件中的 .class 条目。
///
/// 使用 `zip` crate 解压 JAR（ZIP）文件，提取所有 `.class` 条目。
fn parse_jar_entries(data: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    use std::io::{Read, Seek};
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|error| miette!("无法读取 JAR 文件：{}", error))?;

    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|error| miette!("无法读取 JAR 条目 {}：{}", index, error))?;
        let name = file.name().to_string();
        if name.ends_with(".class") {
            let mut buffer = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buffer).map_err(|error| miette!("无法解压 JAR 条目 {}：{}", index, error))?;
            entries.push((name, buffer));
        }
    }

    Ok(entries)
}
