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
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum JvmTypeDescriptor {
    /// `byte` 基本类型。
    Byte,
    /// `char` 基本类型。
    Char,
    /// `double` 基本类型。
    Double,
    /// `float` 基本类型。
    Float,
    /// `int` 基本类型。
    Int,
    /// `long` 基本类型。
    Long,
    /// `short` 基本类型。
    Short,
    /// `boolean` 基本类型。
    Boolean,
    /// `void` 返回类型，仅用于方法描述符。
    Void,
    /// 对象类型，内部存储类内部名（如 `java/lang/String`）。
    Object(String),
    /// 数组类型，内部存储元素类型。
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

    /// 计算该类型在局部变量表与操作数栈中占用的槽位数。
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

/// `JVM` 字段引用。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmFieldRef {
    /// 内部类名，例如 `java/lang/System`。
    pub owner: String,
    /// 字段名，例如 `out`。
    pub name: String,
    /// 字段类型描述符，例如 `Ljava/io/PrintStream;`。
    pub descriptor: JvmTypeDescriptor,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JvmInstruction {
    /// 标签，仅用于编码前的跳转解析，不产生字节码。
    Label(String),
    /// `aload_0` 指令，从局部变量 0 加载引用到操作数栈。
    ALoad0,
    /// `aload` 指令，从指定局部变量加载引用到操作数栈。
    ALoad(u16),
    /// `iload` 指令，从指定局部变量加载 `int` 到操作数栈。
    ILoad(u16),
    /// `lload` 指令，从指定局部变量加载 `long` 到操作数栈。
    LLoad(u16),
    /// `fload` 指令，从指定局部变量加载 `float` 到操作数栈。
    FLoad(u16),
    /// `dload` 指令，从指定局部变量加载 `double` 到操作数栈。
    DLoad(u16),
    /// `astore` 指令，将栈顶引用存入指定局部变量。
    AStore(u16),
    /// `istore` 指令，将栈顶 `int` 存入指定局部变量。
    IStore(u16),
    /// `lstore` 指令，将栈顶 `long` 存入指定局部变量。
    LStore(u16),
    /// `fstore` 指令，将栈顶 `float` 存入指定局部变量。
    FStore(u16),
    /// `dstore` 指令，将栈顶 `double` 存入指定局部变量。
    DStore(u16),
    /// `aconst_null` 指令，将 `null` 压入操作数栈。
    AConstNull,
    /// `iconst`/`bipush`/`sipush`/`ldc` 指令，将 `int` 常量压入操作数栈。
    IConst(i32),
    /// `lconst_0` 指令，将 `long` 值 0 压入操作数栈。
    LConst0,
    /// `lconst_1` 指令，将 `long` 值 1 压入操作数栈。
    LConst1,
    /// `ldc2_w` 指令，从常量池加载 `long` 常量。
    LdcLong(i64),
    /// `fconst_0` 指令，将 `float` 值 0 压入操作数栈。
    FConst0,
    /// `dconst_0` 指令，将 `double` 值 0 压入操作数栈。
    DConst0,
    /// `dconst_1` 指令，将 `double` 值 1 压入操作数栈。
    DConst1,
    /// `ldc2_w` 指令，从常量池加载 `double` 常量。
    LdcDouble(u64),
    /// `ldc_w` 指令，从常量池加载 `String` 常量。
    LdcString(String),
    /// `pop` 指令，弹出栈顶 1 个槽位的值。
    Pop,
    /// `dup` 指令，复制栈顶值并压入。
    Dup,
    /// `swap` 指令，交换栈顶两个单槽值（引用/`int`/`float`）。
    Swap,
    /// `iaload` 指令，从 `int` 数组加载元素。
    IALoad,
    /// `baload` 指令，从 `byte`/`boolean` 数组加载元素（压 `int`）。
    BALoad,
    /// `caload` 指令，从 `char` 数组加载元素（压 `int`）。
    CALoad,
    /// `saload` 指令，从 `short` 数组加载元素（压 `int`）。
    SALoad,
    /// `iastore` 指令，存入 `int` 数组元素。
    IAStore,
    /// `bastore` 指令，存入 `byte`/`boolean` 数组元素。
    BAStore,
    /// `castore` 指令，存入 `char` 数组元素。
    CAStore,
    /// `sastore` 指令，存入 `short` 数组元素。
    SAStore,
    /// `aaload` 指令，从引用数组加载元素。
    AALoad,
    /// `aastore` 指令，存入引用数组元素。
    AAStore,
    /// `arraylength` 指令，获取数组长度。
    ArrayLength,
    /// `iadd` 指令，`int` 加法。
    IAdd,
    /// `ladd` 指令，`long` 加法。
    LAdd,
    /// `fadd` 指令，`float` 加法。
    FAdd,
    /// `dadd` 指令，`double` 加法。
    DAdd,
    /// `isub` 指令，`int` 减法。
    ISub,
    /// `lsub` 指令，`long` 减法。
    LSub,
    /// `fsub` 指令，`float` 减法。
    FSub,
    /// `dsub` 指令，`double` 减法。
    DSub,
    /// `imul` 指令，`int` 乘法。
    IMul,
    /// `lmul` 指令，`long` 乘法。
    LMul,
    /// `fmul` 指令，`float` 乘法。
    FMul,
    /// `dmul` 指令，`double` 乘法。
    DMul,
    /// `idiv` 指令，`int` 除法。
    IDiv,
    /// `ldiv` 指令，`long` 除法。
    LDiv,
    /// `fdiv` 指令，`float` 除法。
    FDiv,
    /// `ddiv` 指令，`double` 除法。
    DDiv,
    /// `irem` 指令，`int` 取余。
    IRem,
    /// `lrem` 指令，`long` 取余。
    LRem,
    /// `frem` 指令，`float` 取余。
    FRem,
    /// `drem` 指令，`double` 取余。
    DRem,
    /// `iand` 指令，`int` 按位与。
    IAnd,
    IXor,
    LAnd,
    LOr,
    LXor,
    IShl,
    IShr,
    IUShr,
    LShl,
    LShr,
    LUShr,
    /// `ior` 指令，`int` 按位或。
    IOr,
    /// `i2l` 指令，`int` 转 `long` (0x85)。
    I2L,
    /// `l2i` 指令，`long` 转 `int` (0x88)。
    L2I,
    /// `i2d` 指令，`int` 转 `double` (0x87)。
    I2D,
    /// `l2d` 指令，`long` 转 `double` (0x8A)。
    L2D,
    /// `d2i` 指令，`double` 转 `int` (0x8B)。
    D2I,
    /// `ineg` 指令，`int` 取负。
    INeg,
    /// `lneg` 指令，`long` 取负。
    LNeg,
    /// `fneg` 指令，`float` 取负。
    FNeg,
    /// `dneg` 指令，`double` 取负。
    DNeg,
    /// `lcmp` 指令，`long` 比较。
    LCmp,
    /// `fcmpl` 指令，`float` 比较（遇 `NaN` 返回 -1）。
    FCmpL,
    /// `fcmpg` 指令，`float` 比较（遇 `NaN` 返回 1）。
    FCmpG,
    /// `dcmpl` 指令，`double` 比较（遇 `NaN` 返回 -1）。
    DCmpL,
    /// `dcmpg` 指令，`double` 比较（遇 `NaN` 返回 1）。
    DCmpG,
    /// `goto` 指令，无条件跳转到指定标签。
    Goto(String),
    /// `ifeq` 指令，栈顶 `int` 为 0 时跳转。
    IfEq(String),
    /// `ifne` 指令，栈顶 `int` 不为 0 时跳转。
    IfNe(String),
    /// `iflt` 指令，栈顶 `int` 小于 0 时跳转。
    IfLt(String),
    /// `ifle` 指令，栈顶 `int` 小于等于 0 时跳转。
    IfLe(String),
    /// `ifgt` 指令，栈顶 `int` 大于 0 时跳转。
    IfGt(String),
    /// `ifge` 指令，栈顶 `int` 大于等于 0 时跳转。
    IfGe(String),
    /// `ifnull` 指令，栈顶为 `null` 时跳转。
    IfNull(String),
    /// `ifnonnull` 指令，栈顶不为 `null` 时跳转。
    IfNonNull(String),
    /// `if_icmpeq` 指令，两个 `int` 相等时跳转。
    IfICmpEq(String),
    /// `if_icmpne` 指令，两个 `int` 不相等时跳转。
    IfICmpNe(String),
    /// `if_icmplt` 指令，第一个 `int` 小于第二个时跳转。
    IfICmpLt(String),
    /// `if_icmple` 指令，第一个 `int` 小于等于第二个时跳转。
    IfICmpLe(String),
    /// `if_icmpgt` 指令，第一个 `int` 大于第二个时跳转。
    IfICmpGt(String),
    /// `if_icmpge` 指令，第一个 `int` 大于等于第二个时跳转。
    IfICmpGe(String),
    /// `if_acmpeq` 指令，两个引用相等时跳转。
    IfACmpEq(String),
    /// `if_acmpne` 指令，两个引用不相等时跳转。
    IfACmpNe(String),
    /// `checkcast` 指令，类型检查转换。
    CheckCast(String),
    /// `newarray` 指令，创建原始类型数组。
    ///
    /// `atype` 为 JVM 规范表 6.5 `newarray` 的数组类型码：
    /// 4=boolean, 5=char, 6=float, 7=double, 8=byte, 9=short, 10=int, 11=long。
    NewArray(u8),
    /// `newarray` 创建 `int[]`（`atype=10`）的便捷别名。
    NewIntArray,
    /// `anewarray` 指令，创建引用类型数组。
    ANewArray(String),
    /// `getstatic` 指令，读取静态字段。
    GetStatic(JvmFieldRef),
    /// `putstatic` 指令，写入静态字段。
    PutStatic(JvmFieldRef),
    /// `getfield` 指令，读取实例字段。
    GetField(JvmFieldRef),
    /// `putfield` 指令，写入实例字段。
    PutField(JvmFieldRef),
    /// `new` 指令，创建对象实例。
    New(String),
    /// `invokestatic` 指令，调用静态方法。
    InvokeStatic(JvmMethodRef),
    /// `invokevirtual` 指令，调用虚方法。
    InvokeVirtual(JvmMethodRef),
    /// `invokespecial` 指令，调用特殊方法（构造器、`super`、私有方法）。
    InvokeSpecial(JvmMethodRef),
    /// `ireturn` 指令，返回 `int`。
    IReturn,
    /// `lreturn` 指令，返回 `long`。
    LReturn,
    /// `freturn` 指令，返回 `float`。
    FReturn,
    /// `dreturn` 指令，返回 `double`。
    DReturn,
    /// `areturn` 指令，返回引用。
    AReturn,
    /// `return` 指令，返回 `void`。
    Return,
}

/// JVM 字段签名。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JvmFieldSignature {
    /// 字段名。
    pub name: String,
    /// 字段描述符。
    pub descriptor: JvmTypeDescriptor,
    /// 访问标志。
    pub access_flags: u16,
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
    /// 字段列表。
    pub fields: Vec<JvmFieldSignature>,
    /// 方法签名列表。
    pub methods: Vec<JvmMethodSignature>,
    /// 解码时保留的常量池，供 [`decode_instructions`] 等反汇编 API 使用；
    /// 编码路径 [`JvmClassFile::to_bytes`] 不使用该字段。
    #[serde(skip)]
    pub constant_pool: ConstantPool,
    /// 解码时保留的各方法 `Code` 属性原始字节码，与 `methods` 按下标一一对应；
    /// 无 `Code` 属性的方法对应 `None`。编码路径不使用该字段。
    #[serde(skip)]
    pub raw_method_code: Vec<Option<Vec<u8>>>,
}

impl Default for JvmClassFile {
    fn default() -> Self {
        Self {
            minor_version: 0,
            major_version: 49,
            access_flags: ACC_PUBLIC | ACC_SUPER,
            internal_name: String::new(),
            super_name: "java/lang/Object".to_string(),
            interfaces: Vec::new(),
            fields: Vec::new(),
            methods: Vec::new(),
            constant_pool: ConstantPool::default(),
            raw_method_code: Vec::new(),
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
        let mut field_records = Vec::with_capacity(self.fields.len());
        for field in &self.fields {
            let name_index = pool.utf8(&field.name);
            let descriptor_index = pool.utf8(&field.descriptor.to_string());
            field_records.push((field, name_index, descriptor_index));
        }
        let mut method_records = Vec::with_capacity(self.methods.len());
        for method in &self.methods {
            let name_index = pool.utf8(&method.name);
            let descriptor_index = pool.utf8(&method.descriptor.to_string());
            let payload = if method.access_flags & (ACC_NATIVE | ACC_ABSTRACT) == 0 {
                let super_init_methodref = if method.name == "<init>" {
                    let owner = if self.super_name.is_empty() { "java/lang/Object" } else { &self.super_name };
                    Some(pool.methodref(owner, "<init>", &JvmMethodDescriptor::default()))
                }
                else {
                    None
                };
                let mut code = method.code.clone().unwrap_or(build_default_method_code(method, &self.super_name, super_init_methodref)?);
                // HotSpot rejects Code.max_locals < argument slots ("Arguments can't fit into
                // locals"). Emitters that count parameters instead of JVM width (J/D = 2) under-size
                // stubs such as Fine(JJ)I / infix^(IJJ)I — clamp at write time as the last line of defense.
                let min_locals = compute_max_locals(method)?;
                if code.max_locals < min_locals {
                    code.max_locals = min_locals;
                }
                Some(build_code_attribute_payload(&code, &mut pool)?)
            }
            else {
                None
            };
            method_records.push((method, name_index, descriptor_index, payload));
        }

        let this_class = pool.class(&self.internal_name);
        let super_class = if self.super_name.is_empty() { 0 } else { pool.class(&self.super_name) };
        let interface_indexes: Vec<u16> = self.interfaces.iter().map(|name| pool.class(name)).collect();
        let code_name = pool.utf8("Code");

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

        writer.write_u16(self.fields.len() as u16);
        for (field, name_index, descriptor_index) in field_records {
            writer.write_u16(field.access_flags);
            writer.write_u16(name_index);
            writer.write_u16(descriptor_index);
            writer.write_u16(0);
        }
        writer.write_u16(self.methods.len() as u16);
        for (method, name_index, descriptor_index, payload) in method_records {
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
        let mut fields = Vec::with_capacity(fields_count);
        for _ in 0..fields_count {
            let field_access_flags = reader.read_u16()?;
            let name_index = reader.read_u16()?;
            let descriptor_index = reader.read_u16()?;
            let field_attributes_count = reader.read_u16()? as usize;
            for _ in 0..field_attributes_count {
                skip_attribute(&mut reader)?;
            }
            fields.push(JvmFieldSignature {
                name: constants.utf8(name_index)?,
                descriptor: parse_type_descriptor(&constants.utf8(descriptor_index)?).ok_or_else(|| {
                    JvmClassError::InvalidFormat(format!("无效的字段描述符：{}", constants.utf8(descriptor_index).unwrap_or_default()))
                })?,
                access_flags: field_access_flags,
            });
        }

        let methods_count = reader.read_u16()? as usize;
        let mut methods = Vec::with_capacity(methods_count);
        let mut raw_method_code = Vec::with_capacity(methods_count);
        for _ in 0..methods_count {
            let method_access_flags = reader.read_u16()?;
            let name_index = reader.read_u16()?;
            let descriptor_index = reader.read_u16()?;
            let attributes_count = reader.read_u16()? as usize;
            let mut method_code_bytes = None;
            for _ in 0..attributes_count {
                let attr_name_index = reader.read_u16()?;
                let attr_length = reader.read_u32()? as usize;
                let attr_name = constants.utf8(attr_name_index)?;
                if attr_name == "Code" {
                    method_code_bytes = Some(parse_code_attribute_bytes(&mut reader)?);
                }
                else {
                    reader.skip(attr_length)?;
                }
            }
            methods.push(JvmMethodSignature {
                name: constants.utf8(name_index)?,
                descriptor: JvmMethodDescriptor::parse(&constants.utf8(descriptor_index)?)?,
                access_flags: method_access_flags,
                code: None,
            });
            raw_method_code.push(method_code_bytes);
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
            fields,
            methods,
            constant_pool: constants,
            raw_method_code,
        })
    }
}

/// `JVM class` 常量池条目，对应 `constant_pool` 表中的一项。
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantPoolEntry {
    /// `CONSTANT_Utf8_info`，修改后的 UTF-8 字符串。
    Utf8(String),
    /// `CONSTANT_Class_info`，指向类名的 `Utf8` 索引。
    Class(u16),
    /// `CONSTANT_Fieldref_info`，字段引用。
    Fieldref { class_index: u16, name_and_type_index: u16 },
    /// `CONSTANT_Methodref_info`，方法引用。
    Methodref { class_index: u16, name_and_type_index: u16 },
    /// `CONSTANT_InterfaceMethodref_info`，接口方法引用。
    InterfaceMethodref { class_index: u16, name_and_type_index: u16 },
    /// `CONSTANT_String_info`，字符串常量引用。
    String(u16),
    /// `CONSTANT_Integer_info`，`int` 常量的原始位模式。
    Integer(u32),
    /// `CONSTANT_Float_info`，`float` 常量的原始位模式。
    Float(u32),
    /// `CONSTANT_Long_info`，`long` 常量的原始位模式。
    Long(u64),
    /// `CONSTANT_Double_info`，`double` 常量的原始位模式。
    Double(u64),
    /// `CONSTANT_NameAndType_info`，字段或方法的名称与描述符索引。
    NameAndType { name_index: u16, descriptor_index: u16 },
    /// `CONSTANT_MethodHandle_info`，方法句柄。
    MethodHandle { reference_kind: u8, reference_index: u16 },
    /// `CONSTANT_MethodType_info`，方法类型。
    MethodType(u16),
    /// `CONSTANT_InvokeDynamic_info`，动态调用点。
    InvokeDynamic { bootstrap_method_attr_index: u16, name_and_type_index: u16 },
    /// `CONSTANT_Module_info`，模块名引用。
    Module(u16),
    /// `CONSTANT_Package_info`，包名引用。
    Package(u16),
    /// `Long`/`Double` 占用的第二个槽位，不承载有效数据。
    Padding,
}

/// `JVM class` 常量池，保存 `from_bytes` 解析后的全部条目。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ConstantPool {
    /// 常量池条目列表，下标 0 为占位项。
    pub entries: Vec<ConstantPoolEntry>,
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

    /// 获取指定索引处的 `Utf8` 字符串。
    pub fn utf8(&self, index: u16) -> Result<String, JvmClassError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Utf8(value)) => Ok(value.clone()),
            _ => Err(JvmClassError::InvalidConstantReference(index)),
        }
    }

    /// 获取指定索引处的类内部名。
    pub fn class_name(&self, index: u16) -> Result<String, JvmClassError> {
        match self.entries.get(index as usize) {
            Some(ConstantPoolEntry::Class(name_index)) => self.utf8(*name_index),
            _ => Err(JvmClassError::InvalidConstantReference(index)),
        }
    }

    /// 获取指定索引处的类内部名，失败时返回 `None`。
    pub fn class_name_opt(&self, index: u16) -> Option<String> {
        self.class_name(index).ok()
    }

    /// 获取指定索引处的字符串常量值，失败时返回 `None`。
    pub fn string_value(&self, index: u16) -> Option<String> {
        match self.entries.get(index as usize)? {
            ConstantPoolEntry::String(utf8_index) => self.utf8(*utf8_index).ok(),
            _ => None,
        }
    }

    /// 获取指定索引处的 `int` 常量，失败时返回 `None`。
    pub fn integer(&self, index: u16) -> Option<i32> {
        match self.entries.get(index as usize)? {
            ConstantPoolEntry::Integer(bits) => Some(*bits as i32),
            _ => None,
        }
    }

    /// 获取指定索引处的 `long` 常量，失败时返回 `None`。
    pub fn long(&self, index: u16) -> Option<i64> {
        match self.entries.get(index as usize)? {
            ConstantPoolEntry::Long(bits) => Some(*bits as i64),
            _ => None,
        }
    }

    /// 获取指定索引处的 `float` 常量，失败时返回 `None`。
    pub fn float(&self, index: u16) -> Option<f32> {
        match self.entries.get(index as usize)? {
            ConstantPoolEntry::Float(bits) => Some(f32::from_bits(*bits)),
            _ => None,
        }
    }

    /// 获取指定索引处的 `double` 常量，失败时返回 `None`。
    pub fn double(&self, index: u16) -> Option<f64> {
        match self.entries.get(index as usize)? {
            ConstantPoolEntry::Double(bits) => Some(f64::from_bits(*bits)),
            _ => None,
        }
    }

    /// 解析方法引用（`Methodref` / `InterfaceMethodref`）为结构化 [`JvmMethodRef`]。
    pub fn method_ref(&self, index: u16) -> Option<JvmMethodRef> {
        let (class_index, name_and_type_index) = match self.entries.get(index as usize)? {
            ConstantPoolEntry::Methodref { class_index, name_and_type_index }
            | ConstantPoolEntry::InterfaceMethodref { class_index, name_and_type_index } => (*class_index, *name_and_type_index),
            _ => return None,
        };
        let owner = self.class_name(class_index).ok()?;
        let (name_index, descriptor_index) = match self.entries.get(name_and_type_index as usize)? {
            ConstantPoolEntry::NameAndType { name_index, descriptor_index } => (*name_index, *descriptor_index),
            _ => return None,
        };
        let name = self.utf8(name_index).ok()?;
        let descriptor_text = self.utf8(descriptor_index).ok()?;
        let descriptor = JvmMethodDescriptor::parse(&descriptor_text).ok()?;
        Some(JvmMethodRef { owner, name, descriptor })
    }

    /// 解析字段引用（`Fieldref`）为结构化 [`JvmFieldRef`]。
    pub fn field_ref(&self, index: u16) -> Option<JvmFieldRef> {
        let (class_index, name_and_type_index) = match self.entries.get(index as usize)? {
            ConstantPoolEntry::Fieldref { class_index, name_and_type_index } => (*class_index, *name_and_type_index),
            _ => return None,
        };
        let owner = self.class_name(class_index).ok()?;
        let (name_index, descriptor_index) = match self.entries.get(name_and_type_index as usize)? {
            ConstantPoolEntry::NameAndType { name_index, descriptor_index } => (*name_index, *descriptor_index),
            _ => return None,
        };
        let name = self.utf8(name_index).ok()?;
        let descriptor_text = self.utf8(descriptor_index).ok()?;
        let descriptor = parse_type_descriptor(&descriptor_text)?;
        Some(JvmFieldRef { owner, name, descriptor })
    }
}

/// 解析单个类型描述符文本为 [`JvmTypeDescriptor`]。
fn parse_type_descriptor(source: &str) -> Option<JvmTypeDescriptor> {
    let mut cursor = 0usize;
    JvmTypeDescriptor::parse(source, &mut cursor, true).ok()
}

/// `JVM class` 常量池构建器，用于在编码时按需追加并去重常量池条目。
#[derive(Default)]
pub struct ConstantPoolBuilder {
    entries: Vec<BuilderEntry>,
    utf8_entries: BTreeMap<String, u16>,
    class_entries: BTreeMap<String, u16>,
    string_entries: BTreeMap<String, u16>,
    integer_entries: BTreeMap<i32, u16>,
    long_entries: BTreeMap<i64, u16>,
    double_entries: BTreeMap<u64, u16>,
    name_and_type_entries: BTreeMap<(String, String), u16>,
    methodref_entries: BTreeMap<(String, String, String), u16>,
    fieldref_entries: BTreeMap<(String, String, String), u16>,
}

impl ConstantPoolBuilder {
    /// 创建一个新的常量池构建器。
    pub fn new() -> Self {
        Self::default()
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

    fn string(&mut self, value: &str) -> u16 {
        if let Some(index) = self.string_entries.get(value) {
            return *index;
        }
        let utf8_index = self.utf8(value);
        let index = self.push(BuilderEntry::String(utf8_index));
        self.string_entries.insert(value.to_string(), index);
        index
    }

    fn integer(&mut self, value: i32) -> u16 {
        if let Some(index) = self.integer_entries.get(&value) {
            return *index;
        }
        let index = self.push(BuilderEntry::Integer(value as u32));
        self.integer_entries.insert(value, index);
        index
    }

    fn long(&mut self, value: i64) -> u16 {
        if let Some(index) = self.long_entries.get(&value) {
            return *index;
        }
        let index = self.push_wide(BuilderEntry::Long(value as u64));
        self.long_entries.insert(value, index);
        index
    }

    fn double(&mut self, value: f64) -> u16 {
        let bits = value.to_bits();
        if let Some(index) = self.double_entries.get(&bits) {
            return *index;
        }
        let index = self.push_wide(BuilderEntry::Double(bits));
        self.double_entries.insert(bits, index);
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

    fn fieldref(&mut self, owner: &str, name: &str, field_descriptor_text: &str) -> u16 {
        let key = (owner.to_string(), name.to_string(), field_descriptor_text.to_string());
        if let Some(index) = self.fieldref_entries.get(&key) {
            return *index;
        }
        let class_index = self.class(owner);
        // 字段的 NameAndType 仅包含类型描述符，不是方法描述符。
        let name_index = self.utf8(name);
        let descriptor_index = self.utf8(field_descriptor_text);
        let name_and_type_index = self.push(BuilderEntry::NameAndType { name_index, descriptor_index });
        let index = self.push(BuilderEntry::Fieldref { class_index, name_and_type_index });
        self.fieldref_entries.insert(key, index);
        index
    }

    fn push(&mut self, entry: BuilderEntry) -> u16 {
        self.entries.push(entry);
        self.entries.len() as u16
    }

    fn push_wide(&mut self, entry: BuilderEntry) -> u16 {
        let index = self.entries.len() as u16 + 1;
        self.entries.push(entry);
        self.entries.push(BuilderEntry::Padding);
        index
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
                BuilderEntry::Integer(value) => {
                    writer.write_u8(3);
                    writer.write_u32(*value);
                }
                BuilderEntry::Long(value) => {
                    writer.write_u8(5);
                    writer.write_u64(*value);
                }
                BuilderEntry::Double(value) => {
                    writer.write_u8(6);
                    writer.write_u64(*value);
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
                BuilderEntry::Fieldref { class_index, name_and_type_index } => {
                    writer.write_u8(9);
                    writer.write_u16(*class_index);
                    writer.write_u16(*name_and_type_index);
                }
                BuilderEntry::Padding => {}
            }
        }
        Ok(())
    }
}

enum BuilderEntry {
    Utf8(String),
    Class(u16),
    String(u16),
    Integer(u32),
    Long(u64),
    Double(u64),
    NameAndType { name_index: u16, descriptor_index: u16 },
    Methodref { class_index: u16, name_and_type_index: u16 },
    Fieldref { class_index: u16, name_and_type_index: u16 },
    Padding,
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

/// 将结构化指令序列编码为 `JVM` 字节码，解析标签并计算相对跳转偏移。
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
        | JvmInstruction::LConst1
        | JvmInstruction::FConst0
        | JvmInstruction::DConst0
        | JvmInstruction::DConst1
        | JvmInstruction::Dup
        | JvmInstruction::Swap
        | JvmInstruction::Pop
        | JvmInstruction::IALoad
        | JvmInstruction::BALoad
        | JvmInstruction::CALoad
        | JvmInstruction::SALoad
        | JvmInstruction::IAStore
        | JvmInstruction::BAStore
        | JvmInstruction::CAStore
        | JvmInstruction::SAStore
        | JvmInstruction::AALoad
        | JvmInstruction::AAStore
        | JvmInstruction::ArrayLength
        | JvmInstruction::IAdd
        | JvmInstruction::LAdd
        | JvmInstruction::FAdd
        | JvmInstruction::DAdd
        | JvmInstruction::ISub
        | JvmInstruction::LSub
        | JvmInstruction::FSub
        | JvmInstruction::DSub
        | JvmInstruction::IMul
        | JvmInstruction::LMul
        | JvmInstruction::FMul
        | JvmInstruction::DMul
        | JvmInstruction::IDiv
        | JvmInstruction::LDiv
        | JvmInstruction::FDiv
        | JvmInstruction::DDiv
        | JvmInstruction::IRem
        | JvmInstruction::LRem
        | JvmInstruction::FRem
        | JvmInstruction::DRem
        | JvmInstruction::IAnd
        | JvmInstruction::IOr
        | JvmInstruction::IXor
        | JvmInstruction::LAnd
        | JvmInstruction::LOr
        | JvmInstruction::LXor
        | JvmInstruction::IShl
        | JvmInstruction::IShr
        | JvmInstruction::IUShr
        | JvmInstruction::LShl
        | JvmInstruction::LShr
        | JvmInstruction::LUShr
        | JvmInstruction::I2L
        | JvmInstruction::L2I
        | JvmInstruction::I2D
        | JvmInstruction::L2D
        | JvmInstruction::D2I
        | JvmInstruction::INeg
        | JvmInstruction::LNeg
        | JvmInstruction::FNeg
        | JvmInstruction::DNeg
        | JvmInstruction::LCmp
        | JvmInstruction::FCmpL
        | JvmInstruction::FCmpG
        | JvmInstruction::DCmpL
        | JvmInstruction::DCmpG
        | JvmInstruction::IReturn
        | JvmInstruction::LReturn
        | JvmInstruction::FReturn
        | JvmInstruction::DReturn
        | JvmInstruction::AReturn
        | JvmInstruction::Return => 1,
        JvmInstruction::ALoad(index)
        | JvmInstruction::AStore(index)
        | JvmInstruction::ILoad(index)
        | JvmInstruction::IStore(index)
        | JvmInstruction::LLoad(index)
        | JvmInstruction::FLoad(index)
        | JvmInstruction::DLoad(index)
        | JvmInstruction::LStore(index)
        | JvmInstruction::FStore(index)
        | JvmInstruction::DStore(index) => {
            if *index <= 3 {
                1
            }
            else if *index <= u8::MAX as u16 {
                2
            }
            else {
                4
            }
        }
        JvmInstruction::LdcLong(_) | JvmInstruction::LdcDouble(_) => 3,
        JvmInstruction::LdcString(_) => 3,
        JvmInstruction::IConst(value) => match *value {
            -1..=5 => 1,
            value if (i8::MIN as i32..=i8::MAX as i32).contains(&value) => 2,
            value if (i16::MIN as i32..=i16::MAX as i32).contains(&value) => 3,
            _ => 3,
        },
        JvmInstruction::Goto(_)
        | JvmInstruction::IfEq(_)
        | JvmInstruction::IfNe(_)
        | JvmInstruction::IfLt(_)
        | JvmInstruction::IfLe(_)
        | JvmInstruction::IfGt(_)
        | JvmInstruction::IfGe(_)
        | JvmInstruction::IfNull(_)
        | JvmInstruction::IfNonNull(_)
        | JvmInstruction::IfICmpEq(_)
        | JvmInstruction::IfICmpNe(_)
        | JvmInstruction::IfICmpLt(_)
        | JvmInstruction::IfICmpLe(_)
        | JvmInstruction::IfICmpGt(_)
        | JvmInstruction::IfICmpGe(_)
        | JvmInstruction::IfACmpEq(_)
        | JvmInstruction::IfACmpNe(_) => 3,
        JvmInstruction::CheckCast(_) | JvmInstruction::ANewArray(_) => 3,
        JvmInstruction::NewArray(_) | JvmInstruction::NewIntArray => 2,
        JvmInstruction::GetStatic(_)
        | JvmInstruction::PutStatic(_)
        | JvmInstruction::GetField(_)
        | JvmInstruction::PutField(_)
        | JvmInstruction::New(_)
        | JvmInstruction::InvokeStatic(_)
        | JvmInstruction::InvokeVirtual(_)
        | JvmInstruction::InvokeSpecial(_) => 3,
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
        JvmInstruction::ALoad(index) => encode_aload(*index, bytes)?,
        JvmInstruction::ILoad(index) => encode_iload(*index, bytes)?,
        JvmInstruction::LLoad(index) => encode_lload(*index, bytes)?,
        JvmInstruction::FLoad(index) => encode_fload(*index, bytes)?,
        JvmInstruction::DLoad(index) => encode_dload(*index, bytes)?,
        JvmInstruction::AStore(index) => encode_astore(*index, bytes)?,
        JvmInstruction::IStore(index) => encode_istore(*index, bytes)?,
        JvmInstruction::LStore(index) => encode_lstore(*index, bytes)?,
        JvmInstruction::FStore(index) => encode_fstore(*index, bytes)?,
        JvmInstruction::DStore(index) => encode_dstore(*index, bytes)?,
        JvmInstruction::AConstNull => bytes.push(0x01),
        JvmInstruction::IConst(value) => encode_iconst(*value, pool, bytes)?,
        JvmInstruction::LConst0 => bytes.push(0x09),
        JvmInstruction::LConst1 => bytes.push(0x0A),
        JvmInstruction::FConst0 => bytes.push(0x0B),
        JvmInstruction::DConst0 => bytes.push(0x0E),
        JvmInstruction::DConst1 => bytes.push(0x0F),
        JvmInstruction::LdcLong(value) => {
            bytes.push(0x14);
            let long_ref = pool.long(*value);
            bytes.extend_from_slice(&long_ref.to_be_bytes());
        }
        JvmInstruction::LdcDouble(value) => {
            bytes.push(0x14);
            let double_ref = pool.double(f64::from_bits(*value));
            bytes.extend_from_slice(&double_ref.to_be_bytes());
        }
        JvmInstruction::LdcString(value) => {
            bytes.push(0x13);
            let string_ref = pool.string(value);
            bytes.extend_from_slice(&string_ref.to_be_bytes());
        }
        JvmInstruction::Dup => bytes.push(0x59),
        JvmInstruction::Swap => bytes.push(0x5f),
        JvmInstruction::Pop => bytes.push(0x57),
        JvmInstruction::IALoad => bytes.push(0x2E),
        JvmInstruction::BALoad => bytes.push(0x33),
        JvmInstruction::CALoad => bytes.push(0x34),
        JvmInstruction::SALoad => bytes.push(0x35),
        JvmInstruction::IAStore => bytes.push(0x4F),
        JvmInstruction::BAStore => bytes.push(0x54),
        JvmInstruction::CAStore => bytes.push(0x55),
        JvmInstruction::SAStore => bytes.push(0x56),
        JvmInstruction::AALoad => bytes.push(0x32),
        JvmInstruction::AAStore => bytes.push(0x53),
        JvmInstruction::ArrayLength => bytes.push(0xBE),
        JvmInstruction::IAdd => bytes.push(0x60),
        JvmInstruction::LAdd => bytes.push(0x61),
        JvmInstruction::FAdd => bytes.push(0x62),
        JvmInstruction::DAdd => bytes.push(0x63),
        JvmInstruction::ISub => bytes.push(0x64),
        JvmInstruction::LSub => bytes.push(0x65),
        JvmInstruction::FSub => bytes.push(0x66),
        JvmInstruction::DSub => bytes.push(0x67),
        JvmInstruction::IMul => bytes.push(0x68),
        JvmInstruction::LMul => bytes.push(0x69),
        JvmInstruction::FMul => bytes.push(0x6A),
        JvmInstruction::DMul => bytes.push(0x6B),
        JvmInstruction::IDiv => bytes.push(0x6C),
        JvmInstruction::LDiv => bytes.push(0x6D),
        JvmInstruction::FDiv => bytes.push(0x6E),
        JvmInstruction::DDiv => bytes.push(0x6F),
        JvmInstruction::IRem => bytes.push(0x70),
        JvmInstruction::LRem => bytes.push(0x71),
        JvmInstruction::FRem => bytes.push(0x72),
        JvmInstruction::DRem => bytes.push(0x73),
        JvmInstruction::IAnd => bytes.push(0x7E),
        JvmInstruction::IOr => bytes.push(0x80),
        JvmInstruction::IXor => bytes.push(0x82),
        JvmInstruction::LAnd => bytes.push(0x7F),
        JvmInstruction::LOr => bytes.push(0x81),
        JvmInstruction::LXor => bytes.push(0x83),
        JvmInstruction::IShl => bytes.push(0x78),
        JvmInstruction::IShr => bytes.push(0x7A),
        JvmInstruction::IUShr => bytes.push(0x7C),
        JvmInstruction::LShl => bytes.push(0x79),
        JvmInstruction::LShr => bytes.push(0x7B),
        JvmInstruction::LUShr => bytes.push(0x7D),
        JvmInstruction::INeg => bytes.push(0x74),
        JvmInstruction::LNeg => bytes.push(0x75),
        JvmInstruction::FNeg => bytes.push(0x76),
        JvmInstruction::DNeg => bytes.push(0x77),
        JvmInstruction::LCmp => bytes.push(0x94),
        JvmInstruction::FCmpL => bytes.push(0x95),
        JvmInstruction::FCmpG => bytes.push(0x96),
        JvmInstruction::DCmpL => bytes.push(0x97),
        JvmInstruction::DCmpG => bytes.push(0x98),
        JvmInstruction::Goto(label) => {
            bytes.push(0xA7);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfEq(label) => {
            bytes.push(0x99);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfNe(label) => {
            bytes.push(0x9A);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfLt(label) => {
            bytes.push(0x9B);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfGe(label) => {
            bytes.push(0x9C);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfGt(label) => {
            bytes.push(0x9D);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfLe(label) => {
            bytes.push(0x9E);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfNull(label) => {
            bytes.push(0xC6);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfNonNull(label) => {
            bytes.push(0xC7);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpEq(label) => {
            bytes.push(0x9F);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpNe(label) => {
            bytes.push(0xA0);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpLt(label) => {
            bytes.push(0xA1);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpGe(label) => {
            bytes.push(0xA2);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpGt(label) => {
            bytes.push(0xA3);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfICmpLe(label) => {
            bytes.push(0xA4);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfACmpEq(label) => {
            bytes.push(0xA5);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::IfACmpNe(label) => {
            bytes.push(0xA6);
            encode_branch_target(label, offset, 3, label_offsets, bytes)?;
        }
        JvmInstruction::CheckCast(class_name) => {
            bytes.push(0xC0);
            let class_ref = pool.class(class_name);
            bytes.extend_from_slice(&class_ref.to_be_bytes());
        }
        JvmInstruction::NewArray(atype) => {
            bytes.push(0xBC);
            bytes.push(*atype);
        }
        JvmInstruction::NewIntArray => {
            bytes.push(0xBC);
            bytes.push(10);
        }
        JvmInstruction::ANewArray(class_name) => {
            bytes.push(0xBD);
            let class_ref = pool.class(class_name);
            bytes.extend_from_slice(&class_ref.to_be_bytes());
        }
        JvmInstruction::GetStatic(field_ref) => {
            bytes.push(0xB2);
            let descriptor_text = field_descriptor_text(&field_ref.descriptor);
            let fieldref = pool.fieldref(&field_ref.owner, &field_ref.name, &descriptor_text);
            bytes.extend_from_slice(&fieldref.to_be_bytes());
        }
        JvmInstruction::PutStatic(field_ref) => {
            bytes.push(0xB3);
            let descriptor_text = field_descriptor_text(&field_ref.descriptor);
            let fieldref = pool.fieldref(&field_ref.owner, &field_ref.name, &descriptor_text);
            bytes.extend_from_slice(&fieldref.to_be_bytes());
        }
        JvmInstruction::GetField(field_ref) => {
            bytes.push(0xB4);
            let descriptor_text = field_descriptor_text(&field_ref.descriptor);
            let fieldref = pool.fieldref(&field_ref.owner, &field_ref.name, &descriptor_text);
            bytes.extend_from_slice(&fieldref.to_be_bytes());
        }
        JvmInstruction::PutField(field_ref) => {
            bytes.push(0xB5);
            let descriptor_text = field_descriptor_text(&field_ref.descriptor);
            let fieldref = pool.fieldref(&field_ref.owner, &field_ref.name, &descriptor_text);
            bytes.extend_from_slice(&fieldref.to_be_bytes());
        }
        JvmInstruction::New(class_name) => {
            bytes.push(0xBB);
            let class_ref = pool.class(class_name);
            bytes.extend_from_slice(&class_ref.to_be_bytes());
        }
        JvmInstruction::InvokeStatic(method_ref) => {
            bytes.push(0xB8);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::InvokeVirtual(method_ref) => {
            bytes.push(0xB6);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::InvokeSpecial(method_ref) => {
            bytes.push(0xB7);
            let methodref = pool.methodref(&method_ref.owner, &method_ref.name, &method_ref.descriptor);
            bytes.extend_from_slice(&methodref.to_be_bytes());
        }
        JvmInstruction::I2L => bytes.push(0x85),
        JvmInstruction::L2I => bytes.push(0x88),
        JvmInstruction::I2D => bytes.push(0x87),
        JvmInstruction::L2D => bytes.push(0x8A),
        JvmInstruction::D2I => bytes.push(0x8B),
        JvmInstruction::IReturn => bytes.push(0xAC),
        JvmInstruction::LReturn => bytes.push(0xAD),
        JvmInstruction::FReturn => bytes.push(0xAE),
        JvmInstruction::DReturn => bytes.push(0xAF),
        JvmInstruction::AReturn => bytes.push(0xB0),
        JvmInstruction::Return => bytes.push(0xB1),
    }
    Ok(())
}

fn field_descriptor_text(descriptor: &JvmTypeDescriptor) -> String {
    match descriptor {
        JvmTypeDescriptor::Object(name) => format!("L{name};"),
        JvmTypeDescriptor::Array(item) => format!("[{item}"),
        other => other.to_string(),
    }
}

fn encode_branch_target(
    label: &str,
    offset: usize,
    _branch_width: usize,
    label_offsets: &BTreeMap<String, usize>,
    bytes: &mut Vec<u8>,
) -> Result<(), JvmClassError> {
    // JVM branch offsets are relative to the branch opcode address, not the next PC.
    let target = label_offsets.get(label).copied().ok_or_else(|| JvmClassError::InvalidFormat(format!("找不到跳转标签 {label}")))?;
    let relative = target as isize - offset as isize;
    bytes.extend_from_slice(&(relative as i16).to_be_bytes());
    Ok(())
}

/// 编码局部变量访问指令；槽位 >255 时使用 `wide` 前缀（0xC4）。
fn encode_local_index(index: u16, short_opcodes: [u8; 4], normal_opcode: u8, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    match index {
        0 => bytes.push(short_opcodes[0]),
        1 => bytes.push(short_opcodes[1]),
        2 => bytes.push(short_opcodes[2]),
        3 => bytes.push(short_opcodes[3]),
        value if value <= u8::MAX as u16 => {
            bytes.push(normal_opcode);
            bytes.push(value as u8);
        }
        value => {
            bytes.push(0xC4);
            bytes.push(normal_opcode);
            bytes.push((value >> 8) as u8);
            bytes.push(value as u8);
        }
    }
    Ok(())
}

fn encode_aload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x2A, 0x2B, 0x2C, 0x2D], 0x19, bytes)
}

fn encode_iload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x1A, 0x1B, 0x1C, 0x1D], 0x15, bytes)
}

fn encode_lload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x1E, 0x1F, 0x20, 0x21], 0x16, bytes)
}

fn encode_fload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x22, 0x23, 0x24, 0x25], 0x17, bytes)
}

fn encode_dload(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x26, 0x27, 0x28, 0x29], 0x18, bytes)
}

fn encode_astore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x4B, 0x4C, 0x4D, 0x4E], 0x3A, bytes)
}

fn encode_istore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x3B, 0x3C, 0x3D, 0x3E], 0x36, bytes)
}

fn encode_lstore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x3F, 0x40, 0x41, 0x42], 0x37, bytes)
}

fn encode_fstore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x43, 0x44, 0x45, 0x46], 0x38, bytes)
}

fn encode_dstore(index: u16, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
    encode_local_index(index, [0x47, 0x48, 0x49, 0x4A], 0x39, bytes)
}

fn encode_iconst(value: i32, pool: &mut ConstantPoolBuilder, bytes: &mut Vec<u8>) -> Result<(), JvmClassError> {
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
        _ => {
            bytes.push(0x13);
            let integer_ref = pool.integer(value);
            bytes.extend_from_slice(&integer_ref.to_be_bytes());
        }
    }
    Ok(())
}

/// `JVM` 字节码反汇编后的单条指令。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedJvmInstruction {
    /// 指令在方法字节码中的起始字节偏移。
    pub offset: usize,
    /// 操作码字节。
    pub opcode: u8,
    /// 助记符，例如 `nop`、`aload`、`invokevirtual`。
    pub mnemonic: String,
    /// 结构化操作数；`None` 表示无操作数。
    pub operand: Option<DecodedJvmOperand>,
    /// 指令总字节数（含操作码与操作数）。
    pub size: usize,
}

/// `JVM` 字节码反汇编后的结构化操作数。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecodedJvmOperand {
    /// 常量池索引（`ldc` / `ldc_w` / `ldc2_w` / `invokedynamic`）。
    ConstantIndex(u16),
    /// 16 位相对分支偏移（`ifeq` / `goto` 等）。
    Branch(i32),
    /// 32 位相对分支偏移（`goto_w` / `jsr_w`）。
    BranchWide(i32),
    /// 方法引用（`invokevirtual` / `invokespecial` / `invokestatic`）。
    MethodRef(JvmMethodRef),
    /// 字段引用（`getstatic` / `putstatic` / `getfield` / `putfield`）。
    FieldRef(JvmFieldRef),
    /// 类内部名引用（`new` / `anewarray` / `checkcast` / `instanceof`）。
    ClassRef(String),
    /// `int` 立即数（`bipush` / `sipush` / `newarray`）。
    Int(i32),
    /// `long` 立即数。
    Long(i64),
    /// `float` 立即数。
    Float(f32),
    /// `double` 立即数。
    Double(f64),
    /// 局部变量索引（`iload` / `astore` / `ret` 等）。
    Local(u16),
    /// `tableswitch` 操作数：默认偏移、`low`、`high` 及各 `case` 偏移。
    TableSwitch { default: i32, low: i32, high: i32, offsets: Vec<i32> },
    /// `lookupswitch` 操作数：默认偏移及 `match-offset` 键值对。
    LookupSwitch { default: i32, pairs: Vec<(i32, i32)> },
    /// `invokeinterface` 操作数：方法引用与参数槽位数。
    InvokeInterface { method: JvmMethodRef, count: u8 },
    /// `multianewarray` 操作数：类引用与维度数。
    MultiANewArray { class: String, dimensions: u8 },
    /// `wide` 扩展的局部变量索引。
    WideLocal(u16),
}

/// 将 `JVM` 字节码反汇编为结构化指令列表，覆盖 `0x00`–`0xC9` 全部操作码。
///
/// `tableswitch`(`0xAA`) 与 `lookupswitch`(`0xAB`) 按 `JVM` 规范完整解码，
/// 包含 0–3 字节对齐填充与全部 `case` 偏移。
pub fn decode_instructions(code: &[u8], constant_pool: &ConstantPool) -> Vec<DecodedJvmInstruction> {
    let mut result = Vec::new();
    let mut offset = 0usize;
    while offset < code.len() {
        let opcode = code[offset];
        let rest = &code[offset..];
        let (mnemonic, size, operand) = decode_single_instruction(opcode, rest, offset, constant_pool);
        result.push(DecodedJvmInstruction { offset, opcode, mnemonic, operand, size });
        if size == 0 {
            break;
        }
        offset += size;
    }
    result
}

/// 解码单条指令，返回 (助记符, 指令长度, 操作数)。
fn decode_single_instruction(opcode: u8, rest: &[u8], pc: usize, pool: &ConstantPool) -> (String, usize, Option<DecodedJvmOperand>) {
    let read_u16_at = |pos: usize| u16::from_be_bytes([rest.get(pos).copied().unwrap_or(0), rest.get(pos + 1).copied().unwrap_or(0)]);
    let read_i16_at = |pos: usize| i16::from_be_bytes([rest.get(pos).copied().unwrap_or(0), rest.get(pos + 1).copied().unwrap_or(0)]) as i32;
    let read_i32_at = |pos: usize| {
        i32::from_be_bytes([
            rest.get(pos).copied().unwrap_or(0),
            rest.get(pos + 1).copied().unwrap_or(0),
            rest.get(pos + 2).copied().unwrap_or(0),
            rest.get(pos + 3).copied().unwrap_or(0),
        ])
    };
    match opcode {
        0x00 => ("nop".into(), 1, None),
        0x01 => ("aconst_null".into(), 1, None),
        0x02 => ("iconst_m1".into(), 1, None),
        0x03 => ("iconst_0".into(), 1, None),
        0x04 => ("iconst_1".into(), 1, None),
        0x05 => ("iconst_2".into(), 1, None),
        0x06 => ("iconst_3".into(), 1, None),
        0x07 => ("iconst_4".into(), 1, None),
        0x08 => ("iconst_5".into(), 1, None),
        0x09 => ("lconst_0".into(), 1, None),
        0x0A => ("lconst_1".into(), 1, None),
        0x0B => ("fconst_0".into(), 1, None),
        0x0C => ("fconst_1".into(), 1, None),
        0x0D => ("fconst_2".into(), 1, None),
        0x0E => ("dconst_0".into(), 1, None),
        0x0F => ("dconst_1".into(), 1, None),
        0x10 => ("bipush".into(), 2, Some(DecodedJvmOperand::Int(rest.get(1).copied().unwrap_or(0) as i8 as i32))),
        0x11 => ("sipush".into(), 3, Some(DecodedJvmOperand::Int(read_i16_at(1)))),
        0x12 => ("ldc".into(), 2, Some(DecodedJvmOperand::ConstantIndex(rest.get(1).copied().unwrap_or(0) as u16))),
        0x13 => ("ldc_w".into(), 3, Some(DecodedJvmOperand::ConstantIndex(read_u16_at(1)))),
        0x14 => ("ldc2_w".into(), 3, Some(DecodedJvmOperand::ConstantIndex(read_u16_at(1)))),
        // 加载指令
        0x15 => ("iload".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x16 => ("lload".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x17 => ("fload".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x18 => ("dload".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x19 => ("aload".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x1A => ("iload_0".into(), 1, None),
        0x1B => ("iload_1".into(), 1, None),
        0x1C => ("iload_2".into(), 1, None),
        0x1D => ("iload_3".into(), 1, None),
        0x1E => ("lload_0".into(), 1, None),
        0x1F => ("lload_1".into(), 1, None),
        0x20 => ("lload_2".into(), 1, None),
        0x21 => ("lload_3".into(), 1, None),
        0x22 => ("fload_0".into(), 1, None),
        0x23 => ("fload_1".into(), 1, None),
        0x24 => ("fload_2".into(), 1, None),
        0x25 => ("fload_3".into(), 1, None),
        0x26 => ("dload_0".into(), 1, None),
        0x27 => ("dload_1".into(), 1, None),
        0x28 => ("dload_2".into(), 1, None),
        0x29 => ("dload_3".into(), 1, None),
        0x2A => ("aload_0".into(), 1, None),
        0x2B => ("aload_1".into(), 1, None),
        0x2C => ("aload_2".into(), 1, None),
        0x2D => ("aload_3".into(), 1, None),
        // 数组加载
        0x2E => ("iaload".into(), 1, None),
        0x2F => ("laload".into(), 1, None),
        0x30 => ("faload".into(), 1, None),
        0x31 => ("daload".into(), 1, None),
        0x32 => ("aaload".into(), 1, None),
        0x33 => ("baload".into(), 1, None),
        0x34 => ("caload".into(), 1, None),
        0x35 => ("saload".into(), 1, None),
        // 存储指令
        0x36 => ("istore".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x37 => ("lstore".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x38 => ("fstore".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x39 => ("dstore".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x3A => ("astore".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        0x3B => ("istore_0".into(), 1, None),
        0x3C => ("istore_1".into(), 1, None),
        0x3D => ("istore_2".into(), 1, None),
        0x3E => ("istore_3".into(), 1, None),
        0x3F => ("lstore_0".into(), 1, None),
        0x40 => ("lstore_1".into(), 1, None),
        0x41 => ("lstore_2".into(), 1, None),
        0x42 => ("lstore_3".into(), 1, None),
        0x43 => ("fstore_0".into(), 1, None),
        0x44 => ("fstore_1".into(), 1, None),
        0x45 => ("fstore_2".into(), 1, None),
        0x46 => ("fstore_3".into(), 1, None),
        0x47 => ("dstore_0".into(), 1, None),
        0x48 => ("dstore_1".into(), 1, None),
        0x49 => ("dstore_2".into(), 1, None),
        0x4A => ("dstore_3".into(), 1, None),
        0x4B => ("astore_0".into(), 1, None),
        0x4C => ("astore_1".into(), 1, None),
        0x4D => ("astore_2".into(), 1, None),
        0x4E => ("astore_3".into(), 1, None),
        // 数组存储
        0x4F => ("iastore".into(), 1, None),
        0x50 => ("lastore".into(), 1, None),
        0x51 => ("fastore".into(), 1, None),
        0x52 => ("dastore".into(), 1, None),
        0x53 => ("aastore".into(), 1, None),
        0x54 => ("bastore".into(), 1, None),
        0x55 => ("castore".into(), 1, None),
        0x56 => ("sastore".into(), 1, None),
        0x57 => ("pop".into(), 1, None),
        0x58 => ("pop2".into(), 1, None),
        0x59 => ("dup".into(), 1, None),
        0x5A => ("dup_x1".into(), 1, None),
        0x5B => ("dup_x2".into(), 1, None),
        0x5C => ("dup2".into(), 1, None),
        0x5D => ("dup2_x1".into(), 1, None),
        0x5E => ("dup2_x2".into(), 1, None),
        0x5F => ("swap".into(), 1, None),
        // 算术
        0x60 => ("iadd".into(), 1, None),
        0x61 => ("ladd".into(), 1, None),
        0x62 => ("fadd".into(), 1, None),
        0x63 => ("dadd".into(), 1, None),
        0x64 => ("isub".into(), 1, None),
        0x65 => ("lsub".into(), 1, None),
        0x66 => ("fsub".into(), 1, None),
        0x67 => ("dsub".into(), 1, None),
        0x68 => ("imul".into(), 1, None),
        0x69 => ("lmul".into(), 1, None),
        0x6A => ("fmul".into(), 1, None),
        0x6B => ("dmul".into(), 1, None),
        0x6C => ("idiv".into(), 1, None),
        0x6D => ("ldiv".into(), 1, None),
        0x6E => ("fdiv".into(), 1, None),
        0x6F => ("ddiv".into(), 1, None),
        0x70 => ("irem".into(), 1, None),
        0x71 => ("lrem".into(), 1, None),
        0x72 => ("frem".into(), 1, None),
        0x73 => ("drem".into(), 1, None),
        0x74 => ("ineg".into(), 1, None),
        0x75 => ("lneg".into(), 1, None),
        0x76 => ("fneg".into(), 1, None),
        0x77 => ("dneg".into(), 1, None),
        0x78 => ("ishl".into(), 1, None),
        0x79 => ("lshl".into(), 1, None),
        0x7A => ("ishr".into(), 1, None),
        0x7B => ("lshr".into(), 1, None),
        0x7C => ("iushr".into(), 1, None),
        0x7D => ("lushr".into(), 1, None),
        0x7E => ("iand".into(), 1, None),
        0x7F => ("land".into(), 1, None),
        0x80 => ("ior".into(), 1, None),
        0x81 => ("lor".into(), 1, None),
        0x82 => ("ixor".into(), 1, None),
        0x83 => ("lxor".into(), 1, None),
        // 转换
        0x84 => ("iinc".into(), 3, Some(DecodedJvmOperand::Int(rest.get(1).copied().unwrap_or(0) as i8 as i32))),
        0x85 => ("i2l".into(), 1, None),
        0x86 => ("i2f".into(), 1, None),
        0x87 => ("i2d".into(), 1, None),
        0x88 => ("l2i".into(), 1, None),
        0x89 => ("l2f".into(), 1, None),
        0x8A => ("l2d".into(), 1, None),
        0x8B => ("f2i".into(), 1, None),
        0x8C => ("f2l".into(), 1, None),
        0x8D => ("f2d".into(), 1, None),
        0x8E => ("d2i".into(), 1, None),
        0x8F => ("d2l".into(), 1, None),
        0x90 => ("d2f".into(), 1, None),
        0x91 => ("i2b".into(), 1, None),
        0x92 => ("i2c".into(), 1, None),
        0x93 => ("i2s".into(), 1, None),
        // 比较
        0x94 => ("lcmp".into(), 1, None),
        0x95 => ("fcmpl".into(), 1, None),
        0x96 => ("fcmpg".into(), 1, None),
        0x97 => ("dcmpl".into(), 1, None),
        0x98 => ("dcmpg".into(), 1, None),
        // 分支
        0x99 => ("ifeq".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9A => ("ifne".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9B => ("iflt".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9C => ("ifge".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9D => ("ifgt".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9E => ("ifle".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0x9F => ("if_icmpeq".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA0 => ("if_icmpne".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA1 => ("if_icmplt".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA2 => ("if_icmpge".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA3 => ("if_icmpgt".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA4 => ("if_icmple".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA5 => ("if_acmpeq".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA6 => ("if_acmpne".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA7 => ("goto".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA8 => ("jsr".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xA9 => ("ret".into(), 2, Some(DecodedJvmOperand::Local(rest.get(1).copied().unwrap_or(0) as u16))),
        // tableswitch / lookupswitch
        0xAA => decode_tableswitch(rest, pc),
        0xAB => decode_lookupswitch(rest, pc),
        // 返回
        0xAC => ("ireturn".into(), 1, None),
        0xAD => ("lreturn".into(), 1, None),
        0xAE => ("freturn".into(), 1, None),
        0xAF => ("dreturn".into(), 1, None),
        0xB0 => ("areturn".into(), 1, None),
        0xB1 => ("return".into(), 1, None),
        // 字段
        0xB2 => ("getstatic".into(), 3, pool.field_ref(read_u16_at(1)).map(DecodedJvmOperand::FieldRef)),
        0xB3 => ("putstatic".into(), 3, pool.field_ref(read_u16_at(1)).map(DecodedJvmOperand::FieldRef)),
        0xB4 => ("getfield".into(), 3, pool.field_ref(read_u16_at(1)).map(DecodedJvmOperand::FieldRef)),
        0xB5 => ("putfield".into(), 3, pool.field_ref(read_u16_at(1)).map(DecodedJvmOperand::FieldRef)),
        // 方法调用
        0xB6 => ("invokevirtual".into(), 3, pool.method_ref(read_u16_at(1)).map(DecodedJvmOperand::MethodRef)),
        0xB7 => ("invokespecial".into(), 3, pool.method_ref(read_u16_at(1)).map(DecodedJvmOperand::MethodRef)),
        0xB8 => ("invokestatic".into(), 3, pool.method_ref(read_u16_at(1)).map(DecodedJvmOperand::MethodRef)),
        0xB9 => (
            "invokeinterface".into(),
            5,
            pool.method_ref(read_u16_at(1))
                .map(|method| DecodedJvmOperand::InvokeInterface { method, count: rest.get(3).copied().unwrap_or(0) }),
        ),
        0xBA => ("invokedynamic".into(), 5, Some(DecodedJvmOperand::ConstantIndex(read_u16_at(1)))),
        // 对象
        0xBB => ("new".into(), 3, pool.class_name_opt(read_u16_at(1)).map(DecodedJvmOperand::ClassRef)),
        0xBC => ("newarray".into(), 2, Some(DecodedJvmOperand::Int(rest.get(1).copied().unwrap_or(0) as i32))),
        0xBD => ("anewarray".into(), 3, pool.class_name_opt(read_u16_at(1)).map(DecodedJvmOperand::ClassRef)),
        0xBE => ("arraylength".into(), 1, None),
        0xBF => ("athrow".into(), 1, None),
        0xC0 => ("checkcast".into(), 3, pool.class_name_opt(read_u16_at(1)).map(DecodedJvmOperand::ClassRef)),
        0xC1 => ("instanceof".into(), 3, pool.class_name_opt(read_u16_at(1)).map(DecodedJvmOperand::ClassRef)),
        0xC2 => ("monitorenter".into(), 1, None),
        0xC3 => ("monitorexit".into(), 1, None),
        // wide
        0xC4 => decode_wide(rest),
        // multidim array
        0xC5 => (
            "multianewarray".into(),
            4,
            pool.class_name_opt(read_u16_at(1))
                .map(|class| DecodedJvmOperand::MultiANewArray { class, dimensions: rest.get(3).copied().unwrap_or(0) }),
        ),
        0xC6 => ("ifnull".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xC7 => ("ifnonnull".into(), 3, Some(DecodedJvmOperand::Branch(read_i16_at(1)))),
        0xC8 => ("goto_w".into(), 5, Some(DecodedJvmOperand::BranchWide(read_i32_at(1)))),
        0xC9 => ("jsr_w".into(), 5, Some(DecodedJvmOperand::BranchWide(read_i32_at(1)))),
        _ => (format!("unknown(0x{:02X})", opcode), 1, None),
    }
}

/// 解码 `tableswitch` 指令（0xAA），按 JVM 规范处理 0–3 字节对齐填充。
fn decode_tableswitch(rest: &[u8], pc: usize) -> (String, usize, Option<DecodedJvmOperand>) {
    let pad = (4 - ((pc + 1) % 4)) % 4;
    let base = 1 + pad;
    let default = i32::from_be_bytes([
        rest.get(base).copied().unwrap_or(0),
        rest.get(base + 1).copied().unwrap_or(0),
        rest.get(base + 2).copied().unwrap_or(0),
        rest.get(base + 3).copied().unwrap_or(0),
    ]);
    let low = i32::from_be_bytes([
        rest.get(base + 4).copied().unwrap_or(0),
        rest.get(base + 5).copied().unwrap_or(0),
        rest.get(base + 6).copied().unwrap_or(0),
        rest.get(base + 7).copied().unwrap_or(0),
    ]);
    let high = i32::from_be_bytes([
        rest.get(base + 8).copied().unwrap_or(0),
        rest.get(base + 9).copied().unwrap_or(0),
        rest.get(base + 10).copied().unwrap_or(0),
        rest.get(base + 11).copied().unwrap_or(0),
    ]);
    let count = if high >= low { (high - low + 1) as usize } else { 0 };
    let mut offsets = Vec::with_capacity(count);
    for i in 0..count {
        let pos = base + 12 + i * 4;
        let off = i32::from_be_bytes([
            rest.get(pos).copied().unwrap_or(0),
            rest.get(pos + 1).copied().unwrap_or(0),
            rest.get(pos + 2).copied().unwrap_or(0),
            rest.get(pos + 3).copied().unwrap_or(0),
        ]);
        offsets.push(off);
    }
    let size = base + 12 + count * 4;
    ("tableswitch".into(), size, Some(DecodedJvmOperand::TableSwitch { default, low, high, offsets }))
}

/// 解码 `lookupswitch` 指令（0xAB），按 JVM 规范处理 0–3 字节对齐填充。
fn decode_lookupswitch(rest: &[u8], pc: usize) -> (String, usize, Option<DecodedJvmOperand>) {
    let pad = (4 - ((pc + 1) % 4)) % 4;
    let base = 1 + pad;
    let default = i32::from_be_bytes([
        rest.get(base).copied().unwrap_or(0),
        rest.get(base + 1).copied().unwrap_or(0),
        rest.get(base + 2).copied().unwrap_or(0),
        rest.get(base + 3).copied().unwrap_or(0),
    ]);
    let npairs = i32::from_be_bytes([
        rest.get(base + 4).copied().unwrap_or(0),
        rest.get(base + 5).copied().unwrap_or(0),
        rest.get(base + 6).copied().unwrap_or(0),
        rest.get(base + 7).copied().unwrap_or(0),
    ]) as usize;
    let mut pairs = Vec::with_capacity(npairs);
    for i in 0..npairs {
        let pos = base + 8 + i * 8;
        let match_val = i32::from_be_bytes([
            rest.get(pos).copied().unwrap_or(0),
            rest.get(pos + 1).copied().unwrap_or(0),
            rest.get(pos + 2).copied().unwrap_or(0),
            rest.get(pos + 3).copied().unwrap_or(0),
        ]);
        let offset = i32::from_be_bytes([
            rest.get(pos + 4).copied().unwrap_or(0),
            rest.get(pos + 5).copied().unwrap_or(0),
            rest.get(pos + 6).copied().unwrap_or(0),
            rest.get(pos + 7).copied().unwrap_or(0),
        ]);
        pairs.push((match_val, offset));
    }
    let size = base + 8 + npairs * 8;
    ("lookupswitch".into(), size, Some(DecodedJvmOperand::LookupSwitch { default, pairs }))
}

/// 解码 `wide` 指令（0xC4）。
fn decode_wide(rest: &[u8]) -> (String, usize, Option<DecodedJvmOperand>) {
    let sub_opcode = rest.get(1).copied().unwrap_or(0);
    let index = u16::from_be_bytes([rest.get(2).copied().unwrap_or(0), rest.get(3).copied().unwrap_or(0)]);
    if sub_opcode == 0x84 {
        // wide iinc: opcode(1) + sub(1) + index(2) + const(2) = 6
        ("wide".into(), 6, Some(DecodedJvmOperand::WideLocal(index)))
    }
    else {
        ("wide".into(), 4, Some(DecodedJvmOperand::WideLocal(index)))
    }
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

#[allow(dead_code)]
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

/// 解析 `Code` 属性，提取原始字节码并跳过异常表与嵌套属性。
fn parse_code_attribute_bytes(reader: &mut ClassReader<'_>) -> Result<Vec<u8>, JvmClassError> {
    reader.read_u16()?; // max_stack
    reader.read_u16()?; // max_locals
    let code_length = reader.read_u32()? as usize;
    let code = reader.read_bytes(code_length)?.to_vec();
    let exception_table_length = reader.read_u16()? as usize;
    for _ in 0..exception_table_length {
        reader.read_u16()?; // start_pc
        reader.read_u16()?; // end_pc
        reader.read_u16()?; // handler_pc
        reader.read_u16()?; // catch_type
    }
    let nested_attributes_count = reader.read_u16()? as usize;
    for _ in 0..nested_attributes_count {
        skip_attribute(reader)?;
    }
    Ok(code)
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

    fn write_u64(&mut self, value: u64) {
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
