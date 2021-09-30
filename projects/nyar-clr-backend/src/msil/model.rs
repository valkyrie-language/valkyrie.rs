use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilAssembly {
    pub name: String,
    pub externs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilModule {
    pub assembly: MsilAssembly,
    pub types: Vec<MsilTypeDef>,
    pub global_methods: Vec<MsilMethodBody>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilTypeDef {
    /// 类型简单名（如 `TextSpan`）。
    pub full_name: String,
    /// 所属命名空间（点分隔，如 `core.text`，空串表示全局命名空间）。
    pub namespace: String,
    pub fields: Vec<MsilField>,
    pub methods: Vec<MsilMethodBody>,
    /// 是否为值类型（`structure`）。`true` 时 `Extends` 指向 `System.ValueType`，
    /// `Flags` 使用 `SequentialLayout`；`false` 时 `Extends` 指向 `System.Object`，
    /// `Flags` 使用 `BeforeFieldInit`。
    pub is_value_type: bool,
}

impl MsilTypeDef {
    /// 返回命名空间限定的全名（如 `core.text.TextSpan`），用于字段 token 解析等需要唯一键的场景。
    pub fn qualified_name(&self) -> String {
        if self.namespace.is_empty() {
            self.full_name.clone()
        }
        else {
            format!("{}.{}", self.namespace, self.full_name)
        }
    }
}

/// `MSIL` 字段定义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilField {
    /// 字段名。
    pub name: String,
    /// 字段类型。
    pub ty: MsilType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilMethodRef {
    pub owner: Option<String>,
    pub name: String,
    pub signature: MsilMethodSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilMethodBody {
    pub method: MsilMethodRef,
    pub locals: Vec<MsilType>,
    pub instructions: Vec<MsilInstruction>,
    pub max_stack: u16,
    pub is_entry_point: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MsilMethodSignature {
    pub return_type: MsilType,
    pub parameter_types: Vec<MsilType>,
}

impl MsilMethodSignature {
    pub fn new(return_type: MsilType, parameter_types: Vec<MsilType>) -> Self {
        Self { return_type, parameter_types }
    }

    pub fn parameter_list_text(&self) -> String {
        let parameters = self.parameter_types.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
        format!("({parameters})")
    }
}

impl Display for MsilMethodSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.return_type, self.parameter_list_text())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum MsilType {
    Void,
    Bool,
    Char,
    Int8,
    UInt8,
    Int16,
    UInt16,
    Int32,
    UInt32,
    Int64,
    UInt64,
    Float32,
    Float64,
    String,
    Object,
    IntPtr,
    UIntPtr,
    SzArray(Box<MsilType>),
    Named(String),
}

impl MsilType {
    pub fn sz_array(item: MsilType) -> Self {
        Self::SzArray(Box::new(item))
    }
}

impl Display for MsilType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Bool => write!(f, "bool"),
            Self::Char => write!(f, "char"),
            Self::Int8 => write!(f, "int8"),
            Self::UInt8 => write!(f, "uint8"),
            Self::Int16 => write!(f, "int16"),
            Self::UInt16 => write!(f, "uint16"),
            Self::Int32 => write!(f, "int32"),
            Self::UInt32 => write!(f, "uint32"),
            Self::Int64 => write!(f, "int64"),
            Self::UInt64 => write!(f, "uint64"),
            Self::Float32 => write!(f, "float32"),
            Self::Float64 => write!(f, "float64"),
            Self::String => write!(f, "string"),
            Self::Object => write!(f, "object"),
            Self::IntPtr => write!(f, "IntPtr"),
            Self::UIntPtr => write!(f, "UIntPtr"),
            Self::SzArray(item) => write!(f, "{}[]", item),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsilInstruction {
    pub label: Option<String>,
    pub opcode: MsilOpcode,
    pub operand: Option<MsilInstructionOperand>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MsilInstructionOperand {
    /// 整数常量。
    Integer(i64),
    /// 浮点数常量（文本表示）。
    Float(String),
    /// 字符串字面量。
    StringLiteral(String),
    /// 符号引用。
    Symbol(String),
    /// 方法引用。
    Method(MsilMethodRef),
    /// 类型引用。
    Type(String),
    /// 字段引用：(所属类型名, 字段名)。
    Field(String, String),
    /// 元数据 token。
    Token(u32),
    /// 分支目标标签。
    BranchTarget(String),
    /// 原始文本。
    Raw(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum MsilOpcode {
    Nop,
    Break,
    Ldarg0,
    Ldarg1,
    Ldarg2,
    Ldarg3,
    Ldarg,
    Ldloc0,
    Ldloc1,
    Ldloc2,
    Ldloc3,
    Stloc0,
    Stloc1,
    Stloc2,
    Stloc3,
    Ldnull,
    LdcI4M1,
    LdcI4_0,
    LdcI4_1,
    LdcI4_2,
    LdcI4_3,
    LdcI4_4,
    LdcI4_5,
    LdcI4_6,
    LdcI4_7,
    LdcI4_8,
    LdcI4S,
    LdcI4,
    LdcI8,
    Ldstr,
    Ldloc,
    Stloc,
    Ldsfld,
    Stsfld,
    Call,
    Callvirt,
    Tail,
    Ret,
    // 控制流：长跳转
    Br,
    Brfalse,
    Brtrue,
    Beq,
    BneUn,
    Bge,
    Bgt,
    Ble,
    Blt,
    BgeUn,
    BgtUn,
    BleUn,
    BltUn,
    // 控制流：短跳转
    BrS,
    BrfalseS,
    BrtrueS,
    BeqS,
    BneUnS,
    BgeS,
    BgtS,
    BleS,
    BltS,
    BgeUnS,
    BgtUnS,
    BleUnS,
    BltUnS,
    // 算术运算
    Add,
    Sub,
    Mul,
    Div,
    DivUn,
    Rem,
    RemUn,
    // 逻辑运算
    And,
    Or,
    Xor,
    // 移位运算
    Shl,
    Shr,
    ShrUn,
    // 一元运算
    Neg,
    Not,
    // 比较运算
    Ceq,
    Clt,
    Cgt,
    CltUn,
    CgtUn,
    // 数组操作
    Ldlen,
    LdelemRef,
    LdelemU1,
    StelemI1,
    StelemRef,
    /// 创建一维零基数组（`newarr`，ECMA-335 III.4.20）。
    /// 操作数是元素类型的 `TypeDefOrRef` token（4 字节）。
    Newarr,
    // 类型转换
    ConvI4,
    // 对象与字段操作
    /// 创建新对象实例（构造函数调用）。
    Newobj,
    /// 加载实例字段值。
    Ldfld,
    /// 存储实例字段值。
    Stfld,
}

impl MsilOpcode {
    pub fn encoding(self) -> u16 {
        match self {
            Self::Nop => 0x00,
            Self::Break => 0x01,
            Self::Ldarg0 => 0x02,
            Self::Ldarg1 => 0x03,
            Self::Ldarg2 => 0x04,
            Self::Ldarg3 => 0x05,
            Self::Ldarg => 0xFE09,
            Self::Ldloc0 => 0x06,
            Self::Ldloc1 => 0x07,
            Self::Ldloc2 => 0x08,
            Self::Ldloc3 => 0x09,
            Self::Stloc0 => 0x0A,
            Self::Stloc1 => 0x0B,
            Self::Stloc2 => 0x0C,
            Self::Stloc3 => 0x0D,
            Self::Ldnull => 0x14,
            Self::LdcI4M1 => 0x15,
            Self::LdcI4_0 => 0x16,
            Self::LdcI4_1 => 0x17,
            Self::LdcI4_2 => 0x18,
            Self::LdcI4_3 => 0x19,
            Self::LdcI4_4 => 0x1A,
            Self::LdcI4_5 => 0x1B,
            Self::LdcI4_6 => 0x1C,
            Self::LdcI4_7 => 0x1D,
            Self::LdcI4_8 => 0x1E,
            Self::LdcI4S => 0x1F,
            Self::LdcI4 => 0x20,
            Self::LdcI8 => 0x21,
            Self::Ldstr => 0x72,
            Self::Ldloc => 0xFE0C,
            Self::Stloc => 0xFE0E,
            Self::Ldsfld => 0x7E,
            Self::Stsfld => 0x80,
            Self::Call => 0x28,
            Self::Callvirt => 0x6F,
            Self::Tail => 0xFE14,
            Self::Ret => 0x2A,
            // 控制流：长跳转
            Self::Br => 0x38,
            Self::Brfalse => 0x39,
            Self::Brtrue => 0x3A,
            Self::Beq => 0x3B,
            Self::BneUn => 0x40,
            Self::Bge => 0x3C,
            Self::Bgt => 0x3D,
            Self::Ble => 0x3E,
            Self::Blt => 0x3F,
            Self::BgeUn => 0x41,
            Self::BgtUn => 0x42,
            Self::BleUn => 0x43,
            Self::BltUn => 0x44,
            // 控制流：短跳转
            Self::BrS => 0x2B,
            Self::BrfalseS => 0x2C,
            Self::BrtrueS => 0x2D,
            Self::BeqS => 0x2E,
            Self::BgeS => 0x2F,
            Self::BgtS => 0x30,
            Self::BleS => 0x31,
            Self::BltS => 0x32,
            Self::BneUnS => 0x33,
            Self::BgeUnS => 0x34,
            Self::BgtUnS => 0x35,
            Self::BleUnS => 0x36,
            Self::BltUnS => 0x37,
            // 算术运算
            Self::Add => 0x58,
            Self::Sub => 0x59,
            Self::Mul => 0x5A,
            Self::Div => 0x5B,
            Self::DivUn => 0x5C,
            Self::Rem => 0x5D,
            Self::RemUn => 0x5E,
            // 逻辑运算
            Self::And => 0x5F,
            Self::Or => 0x60,
            Self::Xor => 0x61,
            // 移位运算
            Self::Shl => 0x62,
            Self::Shr => 0x63,
            Self::ShrUn => 0x64,
            // 一元运算
            Self::Neg => 0x65,
            Self::Not => 0x66,
            // 比较运算
            Self::Ceq => 0xFE01,
            Self::Clt => 0xFE04,
            Self::Cgt => 0xFE02,
            Self::CltUn => 0xFE05,
            Self::CgtUn => 0xFE03,
            Self::Ldlen => 0x8E,
            Self::LdelemRef => 0x9A,
            Self::LdelemU1 => 0x91,
            Self::StelemI1 => 0x9C,
            Self::StelemRef => 0xA2,
            Self::Newarr => 0x8D,
            Self::ConvI4 => 0x69,
            // 对象与字段操作
            Self::Newobj => 0x73,
            Self::Ldfld => 0x7B,
            Self::Stfld => 0x7D,
        }
    }
}

impl Display for MsilOpcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Nop => "nop",
            Self::Break => "break",
            Self::Ldarg0 => "ldarg.0",
            Self::Ldarg1 => "ldarg.1",
            Self::Ldarg2 => "ldarg.2",
            Self::Ldarg3 => "ldarg.3",
            Self::Ldarg => "ldarg",
            Self::Ldloc0 => "ldloc.0",
            Self::Ldloc1 => "ldloc.1",
            Self::Ldloc2 => "ldloc.2",
            Self::Ldloc3 => "ldloc.3",
            Self::Stloc0 => "stloc.0",
            Self::Stloc1 => "stloc.1",
            Self::Stloc2 => "stloc.2",
            Self::Stloc3 => "stloc.3",
            Self::Ldnull => "ldnull",
            Self::LdcI4M1 => "ldc.i4.m1",
            Self::LdcI4_0 => "ldc.i4.0",
            Self::LdcI4_1 => "ldc.i4.1",
            Self::LdcI4_2 => "ldc.i4.2",
            Self::LdcI4_3 => "ldc.i4.3",
            Self::LdcI4_4 => "ldc.i4.4",
            Self::LdcI4_5 => "ldc.i4.5",
            Self::LdcI4_6 => "ldc.i4.6",
            Self::LdcI4_7 => "ldc.i4.7",
            Self::LdcI4_8 => "ldc.i4.8",
            Self::LdcI4S => "ldc.i4.s",
            Self::LdcI4 => "ldc.i4",
            Self::LdcI8 => "ldc.i8",
            Self::Ldstr => "ldstr",
            Self::Ldloc => "ldloc",
            Self::Stloc => "stloc",
            Self::Ldsfld => "ldsfld",
            Self::Stsfld => "stsfld",
            Self::Call => "call",
            Self::Callvirt => "callvirt",
            Self::Tail => "tail.",
            Self::Ret => "ret",
            // 控制流：长跳转
            Self::Br => "br",
            Self::Brfalse => "brfalse",
            Self::Brtrue => "brtrue",
            Self::Beq => "beq",
            Self::BneUn => "bne.un",
            Self::Bge => "bge",
            Self::Bgt => "bgt",
            Self::Ble => "ble",
            Self::Blt => "blt",
            Self::BgeUn => "bge.un",
            Self::BgtUn => "bgt.un",
            Self::BleUn => "ble.un",
            Self::BltUn => "blt.un",
            // 控制流：短跳转
            Self::BrS => "br.s",
            Self::BrfalseS => "brfalse.s",
            Self::BrtrueS => "brtrue.s",
            Self::BeqS => "beq.s",
            Self::BgeS => "bge.s",
            Self::BgtS => "bgt.s",
            Self::BleS => "ble.s",
            Self::BltS => "blt.s",
            Self::BneUnS => "bne.un.s",
            Self::BgeUnS => "bge.un.s",
            Self::BgtUnS => "bgt.un.s",
            Self::BleUnS => "ble.un.s",
            Self::BltUnS => "blt.un.s",
            // 算术运算
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::DivUn => "div.un",
            Self::Rem => "rem",
            Self::RemUn => "rem.un",
            // 逻辑运算
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            // 移位运算
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::ShrUn => "shr.un",
            // 一元运算
            Self::Neg => "neg",
            Self::Not => "not",
            // 比较运算
            Self::Ceq => "ceq",
            Self::Clt => "clt",
            Self::Cgt => "cgt",
            Self::CltUn => "clt.un",
            Self::CgtUn => "cgt.un",
            Self::Ldlen => "ldlen",
            Self::LdelemRef => "ldelem.ref",
            Self::LdelemU1 => "ldelem.u1",
            Self::StelemI1 => "stelem.i1",
            Self::StelemRef => "stelem.ref",
            Self::Newarr => "newarr",
            Self::ConvI4 => "conv.i4",
            // 对象与字段操作
            Self::Newobj => "newobj",
            Self::Ldfld => "ldfld",
            Self::Stfld => "stfld",
        };
        f.write_str(text)
    }
}

impl From<MsilOpcode> for String {
    fn from(value: MsilOpcode) -> Self {
        value.to_string()
    }
}

impl FromStr for MsilOpcode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nop" => Ok(Self::Nop),
            "break" => Ok(Self::Break),
            "ldarg.0" => Ok(Self::Ldarg0),
            "ldarg.1" => Ok(Self::Ldarg1),
            "ldarg.2" => Ok(Self::Ldarg2),
            "ldarg.3" => Ok(Self::Ldarg3),
            "ldarg" => Ok(Self::Ldarg),
            "ldloc.0" => Ok(Self::Ldloc0),
            "ldloc.1" => Ok(Self::Ldloc1),
            "ldloc.2" => Ok(Self::Ldloc2),
            "ldloc.3" => Ok(Self::Ldloc3),
            "stloc.0" => Ok(Self::Stloc0),
            "stloc.1" => Ok(Self::Stloc1),
            "stloc.2" => Ok(Self::Stloc2),
            "stloc.3" => Ok(Self::Stloc3),
            "ldnull" => Ok(Self::Ldnull),
            "ldc.i4.m1" => Ok(Self::LdcI4M1),
            "ldc.i4.0" => Ok(Self::LdcI4_0),
            "ldc.i4.1" => Ok(Self::LdcI4_1),
            "ldc.i4.2" => Ok(Self::LdcI4_2),
            "ldc.i4.3" => Ok(Self::LdcI4_3),
            "ldc.i4.4" => Ok(Self::LdcI4_4),
            "ldc.i4.5" => Ok(Self::LdcI4_5),
            "ldc.i4.6" => Ok(Self::LdcI4_6),
            "ldc.i4.7" => Ok(Self::LdcI4_7),
            "ldc.i4.8" => Ok(Self::LdcI4_8),
            "ldc.i4.s" => Ok(Self::LdcI4S),
            "ldc.i4" => Ok(Self::LdcI4),
            "ldc.i8" => Ok(Self::LdcI8),
            "ldstr" => Ok(Self::Ldstr),
            "ldloc" => Ok(Self::Ldloc),
            "stloc" => Ok(Self::Stloc),
            "ldsfld" => Ok(Self::Ldsfld),
            "stsfld" => Ok(Self::Stsfld),
            "call" => Ok(Self::Call),
            "callvirt" => Ok(Self::Callvirt),
            "tail." => Ok(Self::Tail),
            "ret" => Ok(Self::Ret),
            // 控制流：长跳转
            "br" => Ok(Self::Br),
            "brfalse" => Ok(Self::Brfalse),
            "brtrue" => Ok(Self::Brtrue),
            "beq" => Ok(Self::Beq),
            "bne.un" => Ok(Self::BneUn),
            "bge" => Ok(Self::Bge),
            "bgt" => Ok(Self::Bgt),
            "ble" => Ok(Self::Ble),
            "blt" => Ok(Self::Blt),
            "bge.un" => Ok(Self::BgeUn),
            "bgt.un" => Ok(Self::BgtUn),
            "ble.un" => Ok(Self::BleUn),
            "blt.un" => Ok(Self::BltUn),
            // 控制流：短跳转
            "br.s" => Ok(Self::BrS),
            "brfalse.s" => Ok(Self::BrfalseS),
            "brtrue.s" => Ok(Self::BrtrueS),
            "beq.s" => Ok(Self::BeqS),
            "bge.s" => Ok(Self::BgeS),
            "bgt.s" => Ok(Self::BgtS),
            "ble.s" => Ok(Self::BleS),
            "blt.s" => Ok(Self::BltS),
            "bne.un.s" => Ok(Self::BneUnS),
            "bge.un.s" => Ok(Self::BgeUnS),
            "bgt.un.s" => Ok(Self::BgtUnS),
            "ble.un.s" => Ok(Self::BleUnS),
            "blt.un.s" => Ok(Self::BltUnS),
            // 算术运算
            "add" => Ok(Self::Add),
            "sub" => Ok(Self::Sub),
            "mul" => Ok(Self::Mul),
            "div" => Ok(Self::Div),
            "div.un" => Ok(Self::DivUn),
            "rem" => Ok(Self::Rem),
            "rem.un" => Ok(Self::RemUn),
            // 逻辑运算
            "and" => Ok(Self::And),
            "or" => Ok(Self::Or),
            "xor" => Ok(Self::Xor),
            // 移位运算
            "shl" => Ok(Self::Shl),
            "shr" => Ok(Self::Shr),
            "shr.un" => Ok(Self::ShrUn),
            // 一元运算
            "neg" => Ok(Self::Neg),
            "not" => Ok(Self::Not),
            // 比较运算
            "ceq" => Ok(Self::Ceq),
            "clt" => Ok(Self::Clt),
            "cgt" => Ok(Self::Cgt),
            "clt.un" => Ok(Self::CltUn),
            "cgt.un" => Ok(Self::CgtUn),
            "ldlen" => Ok(Self::Ldlen),
            "ldelem.ref" => Ok(Self::LdelemRef),
            "ldelem.u1" => Ok(Self::LdelemU1),
            "stelem.i1" => Ok(Self::StelemI1),
            "stelem.ref" => Ok(Self::StelemRef),
            "newarr" => Ok(Self::Newarr),
            "conv.i4" => Ok(Self::ConvI4),
            // 对象与字段操作
            "newobj" => Ok(Self::Newobj),
            "ldfld" => Ok(Self::Ldfld),
            "stfld" => Ok(Self::Stfld),
            _ => Err(format!("unknown MSIL opcode: {}", s)),
        }
    }
}

impl TryFrom<String> for MsilOpcode {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
