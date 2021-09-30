#![doc = include_str!("readme.md")]

use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use serde::{Deserialize, Serialize};

const JVM_CLASS_MAGIC: u32 = 0xCAFEBABE;
const ACC_PUBLIC: u16 = 0x0001;
const ACC_STATIC: u16 = 0x0008;
const ACC_SUPER: u16 = 0x0020;
const ACC_NATIVE: u16 = 0x0100;
const ACC_ABSTRACT: u16 = 0x0400;

/// `JVM class` 编解码错误。
#[derive(Debug)]
pub enum JvmClassError {
    /// 输入数据在中途结束。
    UnexpectedEof,
    /// 魔数不正确。
    InvalidMagic(u32),
    /// 常量池标签暂不支持。
    UnsupportedConstantTag(u8),
    /// 常量池引用越界或类型不匹配。
    InvalidConstantReference(u16),
    /// 输入格式不合法。
    InvalidFormat(String),
}

impl Display for JvmClassError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "class 文件在读取过程中意外结束"),
            Self::InvalidMagic(value) => write!(f, "无效的 class 魔数：0x{value:08X}"),
            Self::UnsupportedConstantTag(tag) => write!(f, "暂不支持的常量池标签：{tag}"),
            Self::InvalidConstantReference(index) => write!(f, "无效的常量池引用：{index}"),
            Self::InvalidFormat(message) => write!(f, "无效的 class 格式：{message}"),
        }
    }
}

impl std::error::Error for JvmClassError {}

/// `JVM` 类型描述符。
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum JvmTypeDescriptor {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Void,
    Object(String),
    Array(Box<JvmTypeDescriptor>),
}

impl JvmTypeDescriptor {
    /// 构造数组类型描述符。
    pub fn array(item: JvmTypeDescriptor) -> Self {
        Self::Array(Box::new(item))
    }

    fn parse(source: &str, cursor: &mut usize, allow_void: bool) -> Result<Self, JvmClassError> {
        let Some(ch) = source.as_bytes().get(*cursor).map(|byte| *byte as char)
        else {
            return Err(JvmClassError::InvalidFormat("类型描述符意外结束".to_string()));
        };
        *cursor += 1;
        Ok(match ch {
            'B' => Self::Byte,
            'C' => Self::Char,
            'D' => Self::Double,
            'F' => Self::Float,
            'I' => Self::Int,
            'J' => Self::Long,
            'S' => Self::Short,
            'Z' => Self::Boolean,
            'V' if allow_void => Self::Void,
            'L' => {
                let start = *cursor;
                while *cursor < source.len() && source.as_bytes()[*cursor] as char != ';' {
                    *cursor += 1;
                }
                if *cursor >= source.len() {
                    return Err(JvmClassError::InvalidFormat("对象类型描述符缺少 ';'".to_string()));
                }
                let name = source[start..*cursor].to_string();
                *cursor += 1;
                Self::Object(name)
            }
            '[' => Self::array(Self::parse(source, cursor, false)?),
            _ => {
                return Err(JvmClassError::InvalidFormat(format!("不支持的类型描述符片段：{ch}")));
            }
        })
    }

    fn value_kind(&self) -> JvmValueKind {
        match self {
            Self::Long => JvmValueKind::Long,
            Self::Float => JvmValueKind::Float,
            Self::Double => JvmValueKind::Double,
            Self::Object(_) | Self::Array(_) => JvmValueKind::Reference,
            Self::Void => JvmValueKind::Void,
            Self::Byte | Self::Char | Self::Int | Self::Short | Self::Boolean => JvmValueKind::IntLike,
        }
    }

    /// 返回该类型在局部变量表中占用的槽位数。
    ///
    /// `long` 和 `double` 占 2 槽，`void` 占 0 槽，其余占 1 槽。
    pub fn slot_count(&self) -> u16 {
        match self {
            Self::Long | Self::Double => 2,
            Self::Void => 0,
            _ => 1,
        }
    }
}

impl Display for JvmTypeDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Byte => write!(f, "B"),
            Self::Char => write!(f, "C"),
            Self::Double => write!(f, "D"),
            Self::Float => write!(f, "F"),
            Self::Int => write!(f, "I"),
            Self::Long => write!(f, "J"),
            Self::Short => write!(f, "S"),
            Self::Boolean => write!(f, "Z"),
            Self::Void => write!(f, "V"),
            Self::Object(name) => write!(f, "L{name};"),
            Self::Array(item) => write!(f, "[{item}"),
        }
    }
}

/// `JVM` 方法描述符。
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JvmMethodDescriptor {
    /// 参数类型列表。
    pub parameter_types: Vec<JvmTypeDescriptor>,
    /// 返回类型。
    pub return_type: JvmTypeDescriptor,
}

impl JvmMethodDescriptor {
    /// 创建一个新的方法描述符。
    pub fn new(parameter_types: Vec<JvmTypeDescriptor>, return_type: JvmTypeDescriptor) -> Self {
        Self { parameter_types, return_type }
    }

    /// 从文本描述符解析结构化表示。
    pub fn parse(source: &str) -> Result<Self, JvmClassError> {
        let mut cursor = 0usize;
        if source.as_bytes().get(cursor).copied() != Some(b'(') {
            return Err(JvmClassError::InvalidFormat("方法描述符缺少 '('".to_string()));
        }
        cursor += 1;

        let mut parameter_types = Vec::new();
        while source.as_bytes().get(cursor).copied() != Some(b')') {
            if cursor >= source.len() {
                return Err(JvmClassError::InvalidFormat("方法描述符缺少 ')'".to_string()));
            }
            parameter_types.push(JvmTypeDescriptor::parse(source, &mut cursor, false)?);
        }
        cursor += 1;

        let return_type = JvmTypeDescriptor::parse(source, &mut cursor, true)?;
        if cursor != source.len() {
            return Err(JvmClassError::InvalidFormat("方法描述符包含多余内容".to_string()));
        }
        Ok(Self::new(parameter_types, return_type))
    }

    /// 计算参数总槽位数。
    pub fn parameter_slot_count(&self) -> u16 {
        self.parameter_types.iter().map(JvmTypeDescriptor::slot_count).sum()
    }

    /// 返回类型对应的返回指令。
    pub fn return_instruction(&self) -> JvmInstruction {
        match self.return_type {
            JvmTypeDescriptor::Void => JvmInstruction::Return,
            JvmTypeDescriptor::Long => JvmInstruction::LReturn,
            JvmTypeDescriptor::Double => JvmInstruction::DReturn,
            JvmTypeDescriptor::Float => JvmInstruction::FReturn,
            JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::AReturn,
            _ => JvmInstruction::IReturn,
        }
    }
}

impl Default for JvmMethodDescriptor {
    fn default() -> Self {
        Self::new(Vec::new(), JvmTypeDescriptor::Void)
    }
}

impl Display for JvmMethodDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for parameter in &self.parameter_types {
            write!(f, "{parameter}")?;
        }
        write!(f, "){}", self.return_type)
    }
}

/// `JVM class` 方法签名。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmMethodSignature {
    /// 方法名。
    pub name: String,
    /// 方法描述符。
    pub descriptor: JvmMethodDescriptor,
    /// 方法访问标志。
    pub access_flags: u16,
    /// 方法体；为空时按返回类型生成默认桩代码。
    pub code: Option<JvmCodeBody>,
}

impl Default for JvmMethodSignature {
    fn default() -> Self {
        Self { name: String::new(), descriptor: JvmMethodDescriptor::default(), access_flags: ACC_PUBLIC | ACC_STATIC, code: None }
    }
}

/// `JVM` 方法引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmMethodRef {
    /// 内部类名，例如 `demo/Main`。
    pub owner: String,
    /// 方法名。
    pub name: String,
    /// 方法描述符。
    pub descriptor: JvmMethodDescriptor,
}

/// `JVM` 方法体模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmCodeBody {
    /// 栈深度上限。
    pub max_stack: u16,
    /// 局部变量槽位数量。
    pub max_locals: u16,
    /// 字节码指令序列。
    pub instructions: Vec<JvmInstruction>,
}

impl JvmCodeBody {
    /// 识别静态自尾递归并改写为参数重写 + 回跳。
    ///
    /// 当前仅处理：
    /// - `static` 方法
    /// - 单槽位整型/布尔型参数
    /// - `invokestatic self(...)` 后紧跟匹配的返回指令
    pub fn optimize_static_self_tail_recursion(
        &mut self,
        owner: &str,
        method_name: &str,
        descriptor: &JvmMethodDescriptor,
        access_flags: u16,
    ) -> Result<bool, JvmClassError> {
        if access_flags & ACC_STATIC == 0 {
            return Ok(false);
        }

        let return_instruction = descriptor.return_instruction();
        let parameter_kinds = descriptor.parameter_types.iter().map(JvmTypeDescriptor::value_kind).collect::<Vec<_>>();
        if parameter_kinds.iter().any(|kind| !matches!(kind, JvmValueKind::IntLike)) {
            return Ok(false);
        }

        let entry_label = "__tailcall_entry".to_string();
        if !matches!(self.instructions.first(), Some(JvmInstruction::Label(label)) if label == &entry_label) {
            self.instructions.insert(0, JvmInstruction::Label(entry_label.clone()));
        }

        let mut slot = 0u16;
        let parameter_slots = parameter_kinds
            .iter()
            .map(|_| {
                let current = slot;
                slot += 1;
                current
            })
            .collect::<Vec<_>>();

        let mut rewritten = false;
        let mut index = 0usize;
        while index + 1 < self.instructions.len() {
            let is_self_tail_call = matches!(
                (&self.instructions[index], &self.instructions[index + 1]),
                (JvmInstruction::InvokeStatic(method_ref), ret)
                    if method_ref.owner == owner
                        && method_ref.name == method_name
                        && method_ref.descriptor == *descriptor
                        && *ret == return_instruction
            );
            if is_self_tail_call {
                let mut replacement = Vec::with_capacity(parameter_slots.len() + 1);
                for slot in parameter_slots.iter().rev() {
                    replacement.push(JvmInstruction::IStore(*slot));
                }
                replacement.push(JvmInstruction::Goto(entry_label.clone()));
                self.instructions.splice(index..=index + 1, replacement);
                rewritten = true;
                index += parameter_slots.len() + 1;
                continue;
            }
            index += 1;
        }

        Ok(rewritten)
    }
}

/// `JVM` 指令。
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JvmInstruction {
    /// 标签，仅用于编码前的跳转解析。
    Label(String),
    ALoad0,
    ILoad(u16),
    IStore(u16),
    /// 加载对象引用类型的局部变量到操作数栈。
    ALoad(u16),
    /// 将对象引用从操作数栈存储到局部变量。
    AStore(u16),
    AConstNull,
    IConst(i32),
    /// 加载字符串常量到操作数栈。
    LdcString(String),
    LConst0,
    FConst0,
    DConst0,
    Pop,
    /// 复制栈顶值，对应 `dup` 指令。
    Dup,
    AALoad,
    /// 数组元素存储，对应 `aastore` 指令。
    AAStore,
    /// 获取数组长度，对应 `arraylength` 指令。
    ArrayLength,
    /// 创建对象类型数组，对应 `anewarray` 指令，参数为内部类名。
    ANewArray(String),
    /// 类型检查转换，对应 `checkcast` 指令，参数为内部类名。
    CheckCast(String),
    IAdd,
    ISub,
    IMul,
    IDiv,
    IRem,
    INeg,
    Goto(String),
    IfEq(String),
    IfNe(String),
    IfICmpEq(String),
    IfICmpNe(String),
    IfICmpLt(String),
    IfICmpLe(String),
    IfICmpGt(String),
    IfICmpGe(String),
    InvokeStatic(JvmMethodRef),
    InvokeSpecial(JvmMethodRef),
    /// 虚方法调用，对应 `invokevirtual` 指令。
    InvokeVirtual(JvmMethodRef),
    IReturn,
    LReturn,
    FReturn,
    DReturn,
    AReturn,
    Return,
}

/// `JVM class` 文件模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmClassFile {
    /// 次版本号。
    pub minor_version: u16,
    /// 主版本号。
    pub major_version: u16,
    /// 类访问标志。
    pub access_flags: u16,
    /// 内部类名，例如 `demo/Main`。
    pub internal_name: String,
    /// 父类内部名，默认使用 `java/lang/Object`。
    pub super_name: String,
    /// 实现的接口内部名列表。
    pub interfaces: Vec<String>,
    /// 方法签名列表。
    pub methods: Vec<JvmMethodSignature>,
}

impl Default for JvmClassFile {
    fn default() -> Self {
        Self {
            minor_version: 0,
            major_version: 50,
            access_flags: ACC_PUBLIC | ACC_SUPER,
            internal_name: String::new(),
            super_name: "java/lang/Object".to_string(),
            interfaces: Vec::new(),
            methods: Vec::new(),
        }
    }
}

impl JvmClassFile {
    /// 创建一个新的 `JVM class` 文件模型。
    pub fn new(internal_name: impl Into<String>) -> Self {
        Self { internal_name: internal_name.into(), ..Self::default() }
    }

    /// 追加一个方法签名。
    pub fn push_method(&mut self, name: impl Into<String>, descriptor: JvmMethodDescriptor) {
        self.methods.push(JvmMethodSignature { name: name.into(), descriptor, access_flags: ACC_PUBLIC | ACC_STATIC, code: None });
    }

    /// 对类中的静态自尾递归方法执行循环化改写。
    pub fn optimize_static_self_tail_recursion(&mut self) -> Result<usize, JvmClassError> {
        let mut optimized = 0usize;
        for method in &mut self.methods {
            let Some(code) = &mut method.code
            else {
                continue;
            };
            if code.optimize_static_self_tail_recursion(&self.internal_name, &method.name, &method.descriptor, method.access_flags)? {
                optimized += 1;
            }
        }
        Ok(optimized)
    }

    /// 将 `class` 模型编码为二进制 `ClassFile`。
    pub fn to_bytes(&self) -> Result<Vec<u8>, JvmClassError> {
        let mut pool = ConstantPoolBuilder::default();
        let this_class = pool.class(&self.internal_name);
        let super_class = if self.super_name.is_empty() { 0 } else { pool.class(&self.super_name) };
        let interface_indexes: Vec<u16> = self.interfaces.iter().map(|name| pool.class(name)).collect();
        let code_name = pool.utf8("Code");

        // 第一遍：收集方法名、描述符索引，并预先编码所有方法体以填充常量池。
        // 必须在常量池写入前完成所有 methodref/string 等条目的注册，
        // 否则字节码引用的索引会超出已写入的常量池范围。
        let mut method_records = Vec::with_capacity(self.methods.len());
        for method in &self.methods {
            let name_index = pool.utf8(&method.name);
            let descriptor_index = pool.utf8(&method.descriptor.to_string());
            let code = if method.access_flags & (ACC_NATIVE | ACC_ABSTRACT) == 0 {
                let super_init_methodref = if method.name == "<init>" {
                    let owner = if self.super_name.is_empty() { "java/lang/Object" } else { &self.super_name };
                    Some(pool.methodref(owner, "<init>", &JvmMethodDescriptor::default()))
                }
                else {
                    None
                };
                Some(method.code.clone().unwrap_or(build_default_method_code(method, &self.super_name, super_init_methodref)?))
            }
            else {
                None
            };
            method_records.push((method, name_index, descriptor_index, code));
        }

        // 预编码所有方法体，将 methodref/string 等条目注册到常量池中。
        let mut method_payloads = Vec::with_capacity(method_records.len());
        for (_, _, _, code) in &method_records {
            if let Some(code) = code {
                let payload = build_code_attribute_payload(code, &mut pool)?;
                method_payloads.push(Some(payload));
            }
            else {
                method_payloads.push(None);
            }
        }

        // 常量池现在已包含所有条目，可以安全写入。
        let mut writer = ClassWriter::default();
        writer.write_u32(JVM_CLASS_MAGIC);
        writer.write_u16(self.minor_version);
        writer.write_u16(self.major_version);
        pool.write_into(&mut writer)?;
        writer.write_u16(self.access_flags);
        writer.write_u16(this_class);
        writer.write_u16(super_class);
        writer.write_u16(interface_indexes.len() as u16);
        for interface_index in interface_indexes {
            writer.write_u16(interface_index);
        }

        writer.write_u16(0);
        writer.write_u16(self.methods.len() as u16);
        for ((method, name_index, descriptor_index, _), payload) in method_records.into_iter().zip(method_payloads.into_iter()) {
            writer.write_u16(method.access_flags);
            writer.write_u16(name_index);
            writer.write_u16(descriptor_index);
            match payload {
                Some(payload) => {
                    writer.write_u16(1);
                    writer.write_u16(code_name);
                    writer.write_u32(payload.len() as u32);
                    writer.write_bytes(&payload);
                }
                None => {
                    writer.write_u16(0);
                }
            }
        }

        writer.write_u16(0);
        Ok(writer.into_bytes())
    }

    /// 从二进制 `ClassFile` 解码出模型。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, JvmClassError> {
        let mut reader = ClassReader::new(bytes);
        let magic = reader.read_u32()?;
        if magic != JVM_CLASS_MAGIC {
            return Err(JvmClassError::InvalidMagic(magic));
        }

        let minor_version = reader.read_u16()?;
        let major_version = reader.read_u16()?;
        let constants = ConstantPool::read(&mut reader)?;
        let access_flags = reader.read_u16()?;
        let this_class = reader.read_u16()?;
        let super_class = reader.read_u16()?;

        let interfaces_count = reader.read_u16()? as usize;
        let mut interfaces = Vec::with_capacity(interfaces_count);
        for _ in 0..interfaces_count {
            interfaces.push(constants.class_name(reader.read_u16()?)?);
        }

        let fields_count = reader.read_u16()? as usize;
        for _ in 0..fields_count {
            skip_member(&mut reader)?;
        }

        let methods_count = reader.read_u16()? as usize;
        let mut methods = Vec::with_capacity(methods_count);
        for _ in 0..methods_count {
            let method_access_flags = reader.read_u16()?;
            let name_index = reader.read_u16()?;
            let descriptor_index = reader.read_u16()?;
            let attributes_count = reader.read_u16()? as usize;
            for _ in 0..attributes_count {
                skip_attribute(&mut reader)?;
            }
            methods.push(JvmMethodSignature {
                name: constants.utf8(name_index)?,
                descriptor: JvmMethodDescriptor::parse(&constants.utf8(descriptor_index)?)?,
                access_flags: method_access_flags,
                code: None,
            });
        }

        let attributes_count = reader.read_u16()? as usize;
        for _ in 0..attributes_count {
            skip_attribute(&mut reader)?;
        }

        Ok(Self {
            minor_version,
            major_version,
            access_flags,
            internal_name: constants.class_name(this_class)?,
            super_name: if super_class == 0 { String::new() } else { constants.class_name(super_class)? },
            interfaces,
            methods,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ConstantPoolEntry {
    Utf8(String),
    Class(u16),
    Fieldref { class_index: u16, name_and_type_index: u16 },
    Methodref { class_index: u16, name_and_type_index: u16 },
    InterfaceMethodref { class_index: u16, name_and_type_index: u16 },
    String(u16),
    Integer(u32),
    Float(u32),
    Long(u64),
    Double(u64),
    NameAndType { name_index: u16, descriptor_index: u16 },
    MethodHandle { reference_kind: u8, reference_index: u16 },
    MethodType(u16),
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    Module(u16),
    Package(u16),
    Padding,
}

#[derive(Debug)]
struct ConstantPool {
    entries: Vec<ConstantPoolEntry>,
}

impl ConstantPool {
    fn read(reader: &mut ClassReader<'_>) -> Result<Self, JvmClassError> {
        let count = reader.read_u16()? as usize;
        let mut entries = vec![ConstantPoolEntry::Padding];
        while entries.len() < count {
            let tag = reader.read_u8()?;
            let entry = match tag {
                1 => {
                    let length = reader.read_u16()? as usize;
                    let bytes = reader.read_bytes(length)?;
                    let value = String::from_utf8(bytes.to_vec()).map_err(|_| JvmClassError::InvalidFormat("UTF-8 常量无效".to_string()))?;
                    ConstantPoolEntry::Utf8(value)
                }
                3 => ConstantPoolEntry::Integer(reader.read_u32()?),
                4 => ConstantPoolEntry::Float(reader.read_u32()?),
                5 => {
                    let value = reader.read_u64()?;
                    entries.push(ConstantPoolEntry::Long(value));
                    entries.push(ConstantPoolEntry::Padding);
                    continue;
                }
                6 => {
                    let value = reader.read_u64()?;
                    entries.push(ConstantPoolEntry::Double(value));
                    entries.push(ConstantPoolEntry::Padding);
                    continue;
                }
                7 => ConstantPoolEntry::Class(reader.read_u16()?),
                8 => ConstantPoolEntry::String(reader.read_u16()?),
                9 => ConstantPoolEntry::Fieldref { class_index: reader.read_u16()?, name_and_type_index: reader.read_u16()? },
                10 => ConstantPoolEntry::Methodref { class_index: reader.read_u16()?, name_and_type_index: reader.read_u16()? },
                11 => ConstantPoolEntry::InterfaceMethodref { class_index: reader.read_u16()?, name_and_type_index: reader.read_u16()? },
                12 => ConstantPoolEntry::NameAndType { name_index: reader.read_u16()?, descriptor_index: reader.read_u16()? },
                15 => ConstantPoolEntry::MethodHandle { reference_kind: reader.read_u8()?, reference_index: reader.read_u16()? },
                16 => ConstantPoolEntry::MethodType(reader.read_u16()?),
                18 => ConstantPoolEntry::InvokeDynamic {
                    bootstrap_method_attr_index: reader.read_u16()?,
                    name_and_type_index: reader.read_u16()?,
                },
                19 => ConstantPoolEntry::Module(reader.read_u16()?),
                20 => ConstantPoolEntry::Package(reader.read_u16()?),
                _ => return Err(JvmClassError::UnsupportedConstantTag(tag)),
            };
            entries.push(entry);
        }
        Ok(Self { entries })
    }

    fn utf8(&self, index: u16) -> Result<String, JvmClassError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Utf8(value)) => Ok(value.clone()),
            _ => Err(JvmClassError::InvalidConstantReference(index)),
        }
    }

    fn class_name(&self, index: u16) -> Result<String, JvmClassError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Class(name_index)) => self.utf8(*name_index),
            _ => Err(JvmClassError::InvalidConstantReference(index)),
        }
    }
}

/// `JVM` 常量池构建器，负责在编码阶段收集并去重常量池条目。
#[derive(Default)]
pub struct ConstantPoolBuilder {
    entries: Vec<BuilderEntry>,
    utf8_entries: BTreeMap<String, u16>,
    class_entries: BTreeMap<String, u16>,
    string_entries: BTreeMap<String, u16>,
    name_and_type_entries: BTreeMap<(String, String), u16>,
    methodref_entries: BTreeMap<(String, String, String), u16>,
    integer_entries: BTreeMap<i32, u16>,
}

impl ConstantPoolBuilder {
    /// 创建一个空的常量池构建器。
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            utf8_entries: BTreeMap::new(),
            class_entries: BTreeMap::new(),
            string_entries: BTreeMap::new(),
            name_and_type_entries: BTreeMap::new(),
            methodref_entries: BTreeMap::new(),
            integer_entries: BTreeMap::new(),
        }
    }

    fn utf8(&mut self, value: &str) -> u16 {
        if let Some(index) = self.utf8_entries.get(value) {
            return *index;
        }
        let index = self.push(BuilderEntry::Utf8(value.to_string()));
        self.utf8_entries.insert(value.to_string(), index);
        index
    }

    fn class(&mut self, class_name: &str) -> u16 {
        if let Some(index) = self.class_entries.get(class_name) {
            return *index;
        }
        let name_index = self.utf8(class_name);
        let index = self.push(BuilderEntry::Class(name_index));
        self.class_entries.insert(class_name.to_string(), index);
        index
    }

    /// 将字符串字面量加入常量池，返回 `String` 常量的索引。
    fn string(&mut self, value: &str) -> u16 {
        if let Some(index) = self.string_entries.get(value) {
            return *index;
        }
        let utf8_index = self.utf8(value);
        let index = self.push(BuilderEntry::String(utf8_index));
        self.string_entries.insert(value.to_string(), index);
        index
    }

    fn name_and_type(&mut self, name: &str, descriptor: &JvmMethodDescriptor) -> u16 {
        let descriptor_text = descriptor.to_string();
        let key = (name.to_string(), descriptor_text.clone());
        if let Some(index) = self.name_and_type_entries.get(&key) {
            return *index;
        }
        let name_index = self.utf8(name);
        let descriptor_index = self.utf8(&descriptor_text);
        let index = self.push(BuilderEntry::NameAndType { name_index, descriptor_index });
        self.name_and_type_entries.insert(key, index);
        index
    }

    fn methodref(&mut self, owner: &str, name: &str, descriptor: &JvmMethodDescriptor) -> u16 {
        let key = (owner.to_string(), name.to_string(), descriptor.to_string());
        if let Some(index) = self.methodref_entries.get(&key) {
            return *index;
        }
        let class_index = self.class(owner);
        let name_and_type_index = self.name_and_type(name, descriptor);
        let index = self.push(BuilderEntry::Methodref { class_index, name_and_type_index });
        self.methodref_entries.insert(key, index);
        index
    }

    /// 将 `i32` 整型常量加入常量池，返回 `Integer` 常量的索引。
    ///
    /// 用于 `ldc` / `ldc_w` 加载超出 `sipush` 范围的整型常量。
    fn integer(&mut self, value: i32) -> u16 {
        if let Some(index) = self.integer_entries.get(&value) {
            return *index;
        }
        let index = self.push(BuilderEntry::Integer(value));
        self.integer_entries.insert(value, index);
        index
    }

    fn push(&mut self, entry: BuilderEntry) -> u16 {
        self.entries.push(entry);
        self.entries.len() as u16
    }

    fn write_into(&self, writer: &mut ClassWriter) -> Result<(), JvmClassError> {
        writer.write_u16((self.entries.len() + 1) as u16);
        for entry in &self.entries {
            match entry {
                BuilderEntry::Utf8(value) => {
                    let bytes = value.as_bytes();
                    let length = u16::try_from(bytes.len()).map_err(|_| JvmClassError::InvalidFormat("UTF-8 常量过长".to_string()))?;
                    writer.write_u8(1);
                    writer.write_u16(length);
                    writer.write_bytes(bytes);
                }
                BuilderEntry::Class(name_index) => {
                    writer.write_u8(7);
                    writer.write_u16(*name_index);
                }
                BuilderEntry::String(utf8_index) => {
                    writer.write_u8(8);
                    writer.write_u16(*utf8_index);
                }
                BuilderEntry::NameAndType { name_index, descriptor_index } => {
                    writer.write_u8(12);
                    writer.write_u16(*name_index);
                    writer.write_u16(*descriptor_index);
                }
                BuilderEntry::Methodref { class_index, name_and_type_index } => {
                    writer.write_u8(10);
                    writer.write_u16(*class_index);
                    writer.write_u16(*name_and_type_index);
                }
                BuilderEntry::Integer(value) => {
                    writer.write_u8(3);
                    writer.write_u32(*value as u32);
                }
            }
        }
        Ok(())
    }
}

enum BuilderEntry {
    Utf8(String),
    Class(u16),
    String(u16),
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    /// `CONSTANT_Integer`，tag = 3。
    Integer(i32),
}

fn build_default_method_code(
    method: &JvmMethodSignature,
    super_name: &str,
    _super_init_methodref: Option<u16>,
) -> Result<JvmCodeBody, JvmClassError> {
    if method.name == "<init>" {
        return Ok(JvmCodeBody {
            max_stack: 1,
            max_locals: 1,
            instructions: vec![
                JvmInstruction::ALoad0,
                JvmInstruction::InvokeSpecial(JvmMethodRef {
                    owner: if super_name.is_empty() { "java/lang/Object".to_string() } else { super_name.to_string() },
                    name: "<init>".to_string(),
                    descriptor: JvmMethodDescriptor::default(),
                }),
                JvmInstruction::Return,
            ],
        });
    }

    match method.descriptor.return_type {
        JvmTypeDescriptor::Void => {
            Ok(JvmCodeBody { max_stack: 0, max_locals: compute_max_locals(method)?, instructions: vec![JvmInstruction::Return] })
        }
        JvmTypeDescriptor::Long => Ok(JvmCodeBody {
            max_stack: 2,
            max_locals: compute_max_locals(method)?,
            instructions: vec![JvmInstruction::LConst0, JvmInstruction::LReturn],
        }),
        JvmTypeDescriptor::Double => Ok(JvmCodeBody {
            max_stack: 2,
            max_locals: compute_max_locals(method)?,
            instructions: vec![JvmInstruction::DConst0, JvmInstruction::DReturn],
        }),
        JvmTypeDescriptor::Float => Ok(JvmCodeBody {
            max_stack: 1,
            max_locals: compute_max_locals(method)?,
            instructions: vec![JvmInstruction::FConst0, JvmInstruction::FReturn],
        }),
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => Ok(JvmCodeBody {
            max_stack: 1,
            max_locals: compute_max_locals(method)?,
            instructions: vec![JvmInstruction::AConstNull, JvmInstruction::AReturn],
        }),
        _ => Ok(JvmCodeBody {
            max_stack: 1,
            max_locals: compute_max_locals(method)?,
            instructions: vec![JvmInstruction::IConst(0), JvmInstruction::IReturn],
        }),
    }
}

fn build_code_attribute_payload(code: &JvmCodeBody, pool: &mut ConstantPoolBuilder) -> Result<Vec<u8>, JvmClassError> {
    let encoded = encode_instructions(&code.instructions, pool)?;
    let code_length = u32::try_from(encoded.len()).map_err(|_| JvmClassError::InvalidFormat("方法字节码过长".to_string()))?;
    let mut writer = ClassWriter::default();
    writer.write_u16(code.max_stack);
    writer.write_u16(code.max_locals);
    writer.write_u32(code_length);
    writer.write_bytes(&encoded);
    writer.write_u16(0);
    writer.write_u16(0);
    Ok(writer.into_bytes())
}

/// 将指令序列编码为 `JVM` 字节码，返回原始字节向量。
pub fn encode_instructions(instructions: &[JvmInstruction], pool: &mut ConstantPoolBuilder) -> Result<Vec<u8>, JvmClassError> {
    let mut label_offsets = BTreeMap::new();
    let mut instruction_offsets = Vec::with_capacity(instructions.len());
    let mut current_offset = 0usize;
    for instruction in instructions {
        instruction_offsets.push(current_offset);
        if let JvmInstruction::Label(label) = instruction {
            label_offsets.insert(label.clone(), current_offset);
        }
        current_offset += instruction_size(instruction);
    }

    let mut bytes = Vec::with_capacity(current_offset);
    for (index, instruction) in instructions.iter().enumerate() {
        encode_instruction(instruction, instruction_offsets[index], &label_offsets, pool, &mut bytes)?;
    }
    Ok(bytes)
}

fn instruction_size(instruction: &JvmInstruction) -> usize {
    match instruction {
        JvmInstruction::Label(_) => 0,
        JvmInstruction::ALoad0
        | JvmInstruction::AConstNull
        | JvmInstruction::LConst0
        | JvmInstruction::FConst0
        | JvmInstruction::DConst0
        | JvmInstruction::Pop
        | JvmInstruction::Dup
        | JvmInstruction::AALoad
        | JvmInstruction::AAStore
        | JvmInstruction::ArrayLength
        | JvmInstruction::IAdd
        | JvmInstruction::ISub
        | JvmInstruction::IMul
        | JvmInstruction::IDiv
        | JvmInstruction::IRem
        | JvmInstruction::INeg
        | JvmInstruction::IReturn
        | JvmInstruction::LReturn
        | JvmInstruction::FReturn
        | JvmInstruction::DReturn
        | JvmInstruction::AReturn
        | JvmInstruction::Return => 1,
        JvmInstruction::ILoad(index) | JvmInstruction::IStore(index) => {
            if *index <= 3 {
                1
            }
            else {
                2
            }
        }
        JvmInstruction::ALoad(index) | JvmInstruction::AStore(index) => {
            if *index <= 3 {
                1
            }
            else {
                2
            }
        }
        JvmInstruction::IConst(value) => match *value {
            -1..=5 => 1,
            value if (i8::MIN as i32..=i8::MAX as i32).contains(&value) => 2,
            value if (i16::MIN as i32..=i16::MAX as i32).contains(&value) => 3,
            _ => 3,
        },
        JvmInstruction::LdcString(_) => 3,
        JvmInstruction::Goto(_)
        | JvmInstruction::IfEq(_)
        | JvmInstruction::IfNe(_)
        | JvmInstruction::IfICmpEq(_)
        | JvmInstruction::IfICmpNe(_)
        | JvmInstruction::IfICmpLt(_)
        | JvmInstruction::IfICmpLe(_)
        | JvmInstruction::IfICmpGt(_)
        | JvmInstruction::IfICmpGe(_) => 3,
        JvmInstruction::InvokeStatic(_) | JvmInstruction::InvokeSpecial(_) | JvmInstruction::InvokeVirtual(_) => 3,
        JvmInstruction::ANewArray(_) => 3,
        JvmInstruction::CheckCast(_) => 3,
    }
}

fn encode_instruction(
    instruction: &JvmInstruction,
    offset: usize,
    label_offsets: &BTreeMap<String, usize>,
    pool: &mut ConstantPoolBuilder,
    bytes: &mut Vec<u8>,
) -> Result<(), JvmClassError> {
    match instruction {
        JvmInstruction::Label(_) => {}
        JvmInstruction::ALoad0 => bytes.push(0x2A),
        JvmInstruction::ILoad(index) => encode_iload(*index, bytes)?,
        JvmInstruction::IStore(index) => encode_istore(*index, bytes)?,
        JvmInstruction::ALoad(index) => encode_aload(*index, bytes)?,
        JvmInstruction::AStore(index) => encode_astore(*index, bytes)?,
        JvmInstruction::AConstNull => bytes.push(0x01),
        JvmInstruction::IConst(value) => encode_iconst(*value, bytes, pool)?,
        JvmInstruction::LdcString(value) => {
            let index = pool.string(value);
            bytes.push(0x13);
            bytes.extend_from_slice(&index.to_be_bytes());
        }
        JvmInstruction::LConst0 => bytes.push(0x09),
        JvmInstruction::FConst0 => bytes.push(0x0B),
        JvmInstruction::DConst0 => bytes.push(0x0E),
        JvmInstruction::Pop => bytes.push(0x57),
        JvmInstruction::Dup => bytes.push(0x59),
        JvmInstruction::AALoad => bytes.push(0x32),
        JvmInstruction::AAStore => bytes.push(0x53),
        JvmInstruction::ArrayLength => bytes.push(0xBE),
        JvmInstruction::ANewArray(class_name) => {
            bytes.push(0xBD);
            let class_index = pool.class(class_name);
            bytes.extend_from_slice(&class_index.to_be_bytes());
        }
        JvmInstruction::CheckCast(class_name) => {
            bytes.push(0xC0);
            let class_index = pool.class(class_name);
            bytes.extend_from_slice(&class_index.to_be_bytes());
        }
        JvmInstruction::IAdd => bytes.push(0x60),
        JvmInstruction::ISub => bytes.push(0x64),
        JvmInstruction::IMul => bytes.push(0x68),
        JvmInstruction::IDiv => bytes.push(0x6C),
        JvmInstruction::IRem => bytes.push(0x70),
        JvmInstruction::INeg => bytes.push(0x74),
        JvmInstruction::Goto(label) => {
            bytes.push(0xA7);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfEq(label) => {
            bytes.push(0x99);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfNe(label) => {
            bytes.push(0x9A);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpEq(label) => {
            bytes.push(0x9F);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpNe(label) => {
            bytes.push(0xA0);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpLt(label) => {
            bytes.push(0xA1);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpGe(label) => {
            bytes.push(0xA2);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpGt(label) => {
            bytes.push(0xA3);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpLe(label) => {
            bytes.push(0xA4);
            encode_branch_target(label, offset, label_offsets, bytes)?;
        }
        JvmInstruction::InvokeStatic(method_ref) => {
            bytes.push(0xB8);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::InvokeSpecial(method_ref) => {
            bytes.push(0xB7);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::InvokeVirtual(method_ref) => {
            bytes.push(0xB6);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::IReturn => bytes.push(0xAC),
        JvmInstruction::LReturn => bytes.push(0xAD),
        JvmInstruction::FReturn => bytes.push(0xAE),
        JvmInstruction::DReturn => bytes.push(0xAF),
        JvmInstruction::AReturn => bytes.push(0xB0),
        JvmInstruction::Return => bytes.push(0xB1),
    }
    Ok(())
}

fn encode_branch_target(label: &str, offset: usize, label_offsets: &BTreeMap<String, usize>, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    let target = label_offsets.get(label).copied().ok_or_else(|| JvmClassError::InvalidFormat(format!("找不到跳转标签 {label}")))?;
    // JVM 规范要求分支偏移相对于分支指令 opcode 起始地址，而非下一条指令地址。
    let relative = target as isize - offset as isize;
    bytes.extend_from_slice(&(relative as i16).to_be_bytes());
    Ok(())
}

fn encode_iload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    match index {
        0 => bytes.push(0x1A),
        1 => bytes.push(0x1B),
        2 => bytes.push(0x1C),
        3 => bytes.push(0x1D),
        value if value <= u8::MAX as u16 => {
            bytes.push(0x15);
            bytes.push(value as u8);
        }
        _ => return Err(JvmClassError::InvalidFormat(format!("iload 槽位过大：{index}"))),
    }
    Ok(())
}

fn encode_istore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    match index {
        0 => bytes.push(0x3B),
        1 => bytes.push(0x3C),
        2 => bytes.push(0x3D),
        3 => bytes.push(0x3E),
        value if value <= u8::MAX as u16 => {
            bytes.push(0x36);
            bytes.push(value as u8);
        }
        _ => return Err(JvmClassError::InvalidFormat(format!("istore 槽位过大：{index}"))),
    }
    Ok(())
}

fn encode_aload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    match index {
        0 => bytes.push(0x2A),
        1 => bytes.push(0x2B),
        2 => bytes.push(0x2C),
        3 => bytes.push(0x2D),
        value if value <= u8::MAX as u16 => {
            bytes.push(0x19);
            bytes.push(value as u8);
        }
        _ => return Err(JvmClassError::InvalidFormat(format!("aload 槽位过大：{index}"))),
    }
    Ok(())
}

fn encode_astore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    match index {
        0 => bytes.push(0x4B),
        1 => bytes.push(0x4C),
        2 => bytes.push(0x4D),
        3 => bytes.push(0x4E),
        value if value <= u8::MAX as u16 => {
            bytes.push(0x3A);
            bytes.push(value as u8);
        }
        _ => return Err(JvmClassError::InvalidFormat(format!("astore 槽位过大：{index}"))),
    }
    Ok(())
}

fn encode_iconst(value: i32, bytes: &mut Vec<u8>, pool: &mut ConstantPoolBuilder) -> Result<(), JvmClassError> {
    match value {
        -1 => bytes.push(0x02),
        0 => bytes.push(0x03),
        1 => bytes.push(0x04),
        2 => bytes.push(0x05),
        3 => bytes.push(0x06),
        4 => bytes.push(0x07),
        5 => bytes.push(0x08),
        value if (i8::MIN as i32..=i8::MAX as i32).contains(&value) => {
            bytes.push(0x10);
            bytes.push(value as i8 as u8);
        }
        value if (i16::MIN as i32..=i16::MAX as i32).contains(&value) => {
            bytes.push(0x11);
            bytes.extend_from_slice(&(value as i16).to_be_bytes());
        }
        // 超出 sipush 范围的整型常量通过 ldc_w 从常量池加载。
        value => {
            let index = pool.integer(value);
            bytes.push(0x13);
            bytes.extend_from_slice(&index.to_be_bytes());
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JvmValueKind {
    IntLike,
    Long,
    Float,
    Double,
    Reference,
    Void,
}

fn compute_max_locals(method: &JvmMethodSignature) -> Result<u16, JvmClassError> {
    let parameters = method.descriptor.parameter_slot_count();
    Ok(if method.access_flags & ACC_STATIC == 0 { parameters + 1 } else { parameters })
}

fn skip_member(reader: &mut ClassReader<'_>) -> Result<(), JvmClassError> {
    reader.read_u16()?;
    reader.read_u16()?;
    reader.read_u16()?;
    let attributes_count = reader.read_u16()? as usize;
    for _ in 0..attributes_count {
        skip_attribute(reader)?;
    }
    Ok(())
}

fn skip_attribute(reader: &mut ClassReader<'_>) -> Result<(), JvmClassError> {
    reader.read_u16()?;
    let length = reader.read_u32()? as usize;
    reader.skip(length)
}

#[derive(Default)]
struct ClassWriter {
    bytes: Vec<u8>,
}

impl ClassWriter {
    fn write_u8(&mut self, value: u8) {
        self.bytes.push(value);
    }

    fn write_u16(&mut self, value: u16) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.bytes.extend_from_slice(&value.to_be_bytes());
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

struct ClassReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ClassReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<u8, JvmClassError> {
        let value = *self.bytes.get(self.offset).ok_or(JvmClassError::UnexpectedEof)?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u16(&mut self) -> Result<u16, JvmClassError> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }

    fn read_u32(&mut self) -> Result<u32, JvmClassError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u64(&mut self) -> Result<u64, JvmClassError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], JvmClassError> {
        let end = self.offset.checked_add(length).ok_or(JvmClassError::UnexpectedEof)?;
        let bytes = self.bytes.get(self.offset..end).ok_or(JvmClassError::UnexpectedEof)?;
        self.offset = end;
        Ok(bytes)
    }

    fn skip(&mut self, length: usize) -> Result<(), JvmClassError> {
        self.read_bytes(length).map(|_| ())
    }
}
