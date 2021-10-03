/// Nyar 指令头码。
///
/// 只表示编码层中的头码字段，不等同于解码后的完整指令对象。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NyarHeadCode {
    // --- 控制流 ---
    /// 无操作。
    #[default]
    Nop = 0x00,
    /// 无条件跳转。
    Jump = 0x01,
    /// 条件跳转（真）。
    JumpIfTrue = 0x02,
    /// 条件跳转（假）。
    JumpIfFalse = 0x03,
    /// 函数调用。
    Call = 0x04,
    /// 返回。
    Return = 0x05,
    /// 尾调用。
    TailCall = 0x06,
    /// 异常抛出。
    Throw = 0x07,
    /// 异常捕获。
    Catch = 0x08,
    /// 异常保护块（对应 WASM `try_table`）。
    TryBlock = 0x09,
    /// 异步让出。
    Yield = 0x0A,
    /// 异步恢复。
    Resume = 0x0B,
    /// 效应处理（已废弃，请使用 [`Self::PerformEffect`]）。
    EffectHandle = 0x0C,
    /// 触发代数效应（raise），捕获续延并调用 handler。
    PerformEffect = 0x0D,
    /// 重新抛出异常（对应 WASM `throw_ref`）。
    Rethrow = 0x0E,
    /// 静态分派调用（编译期确定目标，无间接开销）。
    CallStatic = 0x0F,

    // --- 栈操作 ---
    /// 压入常量。
    Const = 0x10,
    /// 弹出栈顶。
    Pop = 0x11,
    /// 复制栈顶。
    Dup = 0x12,
    /// 交换栈顶两个元素。
    Swap = 0x13,

    // --- 效应处理 ---
    /// 进入效应处理器作用域，将效应处理器推入栈。
    EnterEffectHandler = 0x14,
    /// 退出效应处理器作用域，将效应处理器从栈弹出。
    ExitEffectHandler = 0x15,
    /// 进入 try 效应捕获块，将指定效应类型注册为捕获目标。
    EnterTry = 0x16,
    /// 退出 try 效应捕获块，卸载效应捕获。
    ExitTry = 0x17,

    /// FFI 调用应通过 std 适配器声明，禁止直接生成此操作码。
    BuiltinCall = 0x19,

    // --- 局部变量 ---
    /// 加载局部变量。
    LoadLocal = 0x20,
    /// 存储局部变量。
    StoreLocal = 0x21,
    /// 加载参数。
    LoadArg = 0x22,
    /// 加载全局变量。
    LoadGlobal = 0x23,
    /// 存储全局变量。
    StoreGlobal = 0x24,
    /// 存储参数。
    StoreArg = 0x25,

    // --- i32 操作 ---
    /// i32 加法。
    I32Add = 0x30,
    /// i32 减法。
    I32Sub = 0x31,
    /// i32 乘法。
    I32Mul = 0x32,
    /// i32 有符号除法。
    I32DivS = 0x33,
    /// i32 无符号除法。
    I32DivU = 0x34,
    /// i32 有符号取余。
    I32RemS = 0x35,
    /// i32 无符号取余。
    I32RemU = 0x36,
    /// i32 取反。
    I32Neg = 0x37,
    /// i32 按位与。
    I32And = 0x38,
    /// i32 按位或。
    I32Or = 0x39,
    /// i32 按位异或。
    I32Xor = 0x3A,
    /// i32 左移。
    I32Shl = 0x3B,
    /// i32 有符号右移。
    I32ShrS = 0x3C,
    /// i32 无符号右移。
    I32ShrU = 0x3D,
    /// i32 按位取反。
    I32Not = 0x3E,

    // --- i32 比较 ---
    /// i32 相等。
    I32Eq = 0x40,
    /// i32 不等。
    I32Ne = 0x41,
    /// i32 有符号小于。
    I32LtS = 0x42,
    /// i32 无符号小于。
    I32LtU = 0x43,
    /// i32 有符号小于等于。
    I32LeS = 0x44,
    /// i32 无符号小于等于。
    I32LeU = 0x45,
    /// i32 有符号大于。
    I32GtS = 0x46,
    /// i32 无符号大于。
    I32GtU = 0x47,
    /// i32 有符号大于等于。
    I32GeS = 0x48,
    /// i32 无符号大于等于。
    I32GeU = 0x49,
    /// 将栈上的 `any`（Object）拆箱为 i32（int）。
    AnyToI32 = 0x4A,
    /// 将栈上的 `any`（Object）转换为 utf8（String）。
    AnyToUtf8 = 0x4B,

    // --- i64 操作 ---
    /// i64 加法。
    I64Add = 0x50,
    /// i64 减法。
    I64Sub = 0x51,
    /// i64 乘法。
    I64Mul = 0x52,
    /// i64 有符号除法。
    I64DivS = 0x53,
    /// i64 无符号除法。
    I64DivU = 0x54,
    /// i64 取反。
    I64Neg = 0x55,
    /// i64 有符号取余。
    I64RemS = 0x56,
    /// i64 无符号取余。
    I64RemU = 0x57,
    /// i64 按位与。
    I64And = 0x58,
    /// i64 按位或。
    I64Or = 0x59,
    /// i64 按位异或。
    I64Xor = 0x5A,
    /// i64 左移。
    I64Shl = 0x5B,
    /// i64 有符号右移。
    I64ShrS = 0x5C,
    /// i64 无符号右移。
    I64ShrU = 0x5D,
    /// i64 按位取反。
    I64Not = 0x5E,

    // --- i64 比较 ---
    /// i64 相等。
    I64Eq = 0x5F,

    // --- f32 操作 ---
    /// f32 加法。
    F32Add = 0x60,
    /// f32 减法。
    F32Sub = 0x61,
    /// f32 乘法。
    F32Mul = 0x62,
    /// f32 除法。
    F32Div = 0x63,
    /// f32 取反。
    F32Neg = 0x64,

    /// i64 不等。
    I64Ne = 0x65,
    /// i64 有符号小于。
    I64LtS = 0x66,
    /// i64 有符号小于等于。
    I64LeS = 0x67,
    /// i64 有符号大于。
    I64GtS = 0x68,
    /// i64 有符号大于等于。
    I64GeS = 0x69,
    /// 引用相等（用于 `null` / 对象引用语义）。
    RefEq = 0x6A,
    /// 引用不等（用于 `null` / 对象引用语义）。
    RefNe = 0x6B,
    /// i64 无符号小于。
    I64LtU = 0x6C,
    /// i64 无符号小于等于。
    I64LeU = 0x6D,
    /// i64 无符号大于。
    I64GtU = 0x6E,
    /// i64 无符号大于等于。
    I64GeU = 0x6F,

    // --- f64 操作 ---
    /// f64 加法。
    F64Add = 0x70,
    /// f64 减法。
    F64Sub = 0x71,
    /// f64 乘法。
    F64Mul = 0x72,
    /// f64 除法。
    F64Div = 0x73,
    /// f64 取反。
    F64Neg = 0x74,
    /// f64 平方根。
    F64Sqrt = 0x75,
    /// f64 相等比较。
    F64Eq = 0x76,
    /// f64 不等比较。
    F64Ne = 0x77,
    /// f64 小于比较。
    F64Lt = 0x78,
    /// f64 小于等于比较。
    F64Le = 0x79,
    /// f64 大于比较。
    F64Gt = 0x7A,
    /// f64 大于等于比较。
    F64Ge = 0x7B,

    // --- 类型转换 ---
    /// i32 扩展到 i64（有符号）。
    I32ExtendI64S = 0x80,
    /// i32 扩展到 i64（无符号）。
    I32ExtendI64U = 0x81,
    /// i64 截断到 i32（有符号）。
    I64TruncI32S = 0x82,
    /// i64 截断到 i32（无符号）。
    I64TruncI32U = 0x83,
    /// i32 转换到 f32（有符号）。
    I32ToF32S = 0x84,
    /// i32 转换到 f64（有符号）。
    I32ToF64S = 0x85,
    /// i64 转换到 f64（有符号）。
    I64ToF64 = 0x86,
    /// f64 转换到 i32（有符号）。
    F64ToI32 = 0x87,
    /// f64 转换到 i64（有符号）。
    F64ToI64 = 0x88,

    // --- 内存操作 ---
    /// 分配内存。
    Alloc = 0x90,
    /// 释放内存。
    Free = 0x91,
    /// 加载 i32。
    I32Load = 0x92,
    /// 存储 i32。
    I32Store = 0x93,
    /// 加载 i64。
    I64Load = 0x94,
    /// 存储 i64。
    I64Store = 0x95,

    // --- 对象操作 ---
    /// 创建对象。
    NewObject = 0xA0,
    /// 获取属性。
    GetField = 0xA1,
    /// 设置属性。
    SetField = 0xA2,
    /// 数组读取。
    ArrayGet = 0xA3,
    /// 数组写入。
    ArraySet = 0xA4,
    /// 获取长度。
    Length = 0xA5,
    /// 创建闭包。
    NewClosure = 0xA6,
    /// 获取上值。
    GetUpvalue = 0xA7,
    /// 设置上值。
    SetUpvalue = 0xA8,
    /// 见证表分派调用（通过 Witness Table 间接调用，支持热更新）。
    CallWitness = 0xA9,
    /// 动态分派调用（运行时方法查找 + 内联缓存）。
    CallDynamic = 0xAA,
    /// 静态字段访问（编译期确定偏移量）。
    AccessStatic = 0xAB,
    /// 见证表字段访问（通过 Witness Table 间接访问）。
    AccessWitness = 0xAC,
    /// 动态字段访问（运行时名称查找 + 内联缓存）。
    AccessDynamic = 0xAD,
    /// 内联缓存更新（JIT 去虚拟化时使用）。
    InlineCacheUpdate = 0xAE,
    /// 字段写入（字段名由常量池索引指定）。
    FieldStore = 0xAF,

    // --- UTF-8 文本操作 ---
    /// UTF-8 文本拼接。
    Utf8Concat = 0xB0,
    /// UTF-8 文本长度（字节）。
    Utf8LenBytes = 0xB1,
    /// UTF-8 文本长度（字符）。
    Utf8LenChars = 0xB2,
    /// UTF-8 文本子串。
    Utf8Substr = 0xB3,
    /// UTF-8 文本相等比较。
    Utf8Eq = 0xB4,
    /// UTF-8 文本不等比较。
    Utf8Ne = 0xB5,
    /// 索引写入（对象、索引、值从栈弹出）。
    IndexStore = 0xB6,
    /// 数组追加元素（将值追加到列表末尾）。
    ArrayPush = 0xB7,
    /// 获取序数索引。
    GetOrdinalIndex = 0xB8,
    /// 设置序数索引。
    SetOrdinalIndex = 0xB9,
    /// 获取偏移索引。
    GetOffsetIndex = 0xBA,
    /// 设置偏移索引。
    SetOffsetIndex = 0xBB,

    // --- BigInt 操作 ---
    /// BigInt 加法。
    BigIntAdd = 0xC0,
    /// BigInt 减法。
    BigIntSub = 0xC1,
    /// BigInt 乘法。
    BigIntMul = 0xC2,

    // --- FFI 操作 ---
    /// 调用内置函数（通过 Intrinsic ID 索引）。
    CallIntrinsic = 0xD0,
    /// 调用原生函数（通过 FFI 函数名索引）。
    CallNative = 0xD1,
    /// 加载原生库（将库路径压入 FFI 管理器）。
    LoadNativeLib = 0xD2,
    /// 获取原生函数指针（从已加载库中查找导出函数）。
    GetNativeFunc = 0xD3,

    // --- SIMD 操作 ---
    /// SIMD 指令前缀的编码字段。
    Simd = 0xE0,
}

impl NyarHeadCode {
    /// C# `string_concat` 语义别名。
    pub const STRING_CONCAT: Self = Self::Utf8Concat;
    /// C# `string_len_bytes` 语义别名。
    pub const STRING_LEN_BYTES: Self = Self::Utf8LenBytes;
    /// C# `string_len_chars` 语义别名。
    pub const STRING_LEN_CHARS: Self = Self::Utf8LenChars;
    /// C# `string_substr` 语义别名。
    pub const STRING_SUBSTR: Self = Self::Utf8Substr;
    /// C# `string_eq` 语义别名。
    pub const STRING_EQ: Self = Self::Utf8Eq;
    /// C# `string_ne` 语义别名。
    pub const STRING_NE: Self = Self::Utf8Ne;

    /// 将原始字节值解析为头码；未定义值返回 `None`。
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Nop),
            0x01 => Some(Self::Jump),
            0x02 => Some(Self::JumpIfTrue),
            0x03 => Some(Self::JumpIfFalse),
            0x04 => Some(Self::Call),
            0x05 => Some(Self::Return),
            0x06 => Some(Self::TailCall),
            0x07 => Some(Self::Throw),
            0x08 => Some(Self::Catch),
            0x09 => Some(Self::TryBlock),
            0x0A => Some(Self::Yield),
            0x0B => Some(Self::Resume),
            0x0C => Some(Self::EffectHandle),
            0x0D => Some(Self::PerformEffect),
            0x0E => Some(Self::Rethrow),
            0x0F => Some(Self::CallStatic),
            0x10 => Some(Self::Const),
            0x11 => Some(Self::Pop),
            0x12 => Some(Self::Dup),
            0x13 => Some(Self::Swap),
            0x14 => Some(Self::EnterEffectHandler),
            0x15 => Some(Self::ExitEffectHandler),
            0x16 => Some(Self::EnterTry),
            0x17 => Some(Self::ExitTry),
            0x19 => Some(Self::BuiltinCall),
            0x20 => Some(Self::LoadLocal),
            0x21 => Some(Self::StoreLocal),
            0x22 => Some(Self::LoadArg),
            0x23 => Some(Self::LoadGlobal),
            0x24 => Some(Self::StoreGlobal),
            0x25 => Some(Self::StoreArg),
            0x30 => Some(Self::I32Add),
            0x31 => Some(Self::I32Sub),
            0x32 => Some(Self::I32Mul),
            0x33 => Some(Self::I32DivS),
            0x34 => Some(Self::I32DivU),
            0x35 => Some(Self::I32RemS),
            0x36 => Some(Self::I32RemU),
            0x37 => Some(Self::I32Neg),
            0x38 => Some(Self::I32And),
            0x39 => Some(Self::I32Or),
            0x3A => Some(Self::I32Xor),
            0x3B => Some(Self::I32Shl),
            0x3C => Some(Self::I32ShrS),
            0x3D => Some(Self::I32ShrU),
            0x3E => Some(Self::I32Not),
            0x40 => Some(Self::I32Eq),
            0x41 => Some(Self::I32Ne),
            0x42 => Some(Self::I32LtS),
            0x43 => Some(Self::I32LtU),
            0x44 => Some(Self::I32LeS),
            0x45 => Some(Self::I32LeU),
            0x46 => Some(Self::I32GtS),
            0x47 => Some(Self::I32GtU),
            0x48 => Some(Self::I32GeS),
            0x49 => Some(Self::I32GeU),
            0x4A => Some(Self::AnyToI32),
            0x4B => Some(Self::AnyToUtf8),
            0x50 => Some(Self::I64Add),
            0x51 => Some(Self::I64Sub),
            0x52 => Some(Self::I64Mul),
            0x53 => Some(Self::I64DivS),
            0x54 => Some(Self::I64DivU),
            0x55 => Some(Self::I64Neg),
            0x56 => Some(Self::I64RemS),
            0x57 => Some(Self::I64RemU),
            0x58 => Some(Self::I64And),
            0x59 => Some(Self::I64Or),
            0x5A => Some(Self::I64Xor),
            0x5B => Some(Self::I64Shl),
            0x5C => Some(Self::I64ShrS),
            0x5D => Some(Self::I64ShrU),
            0x5E => Some(Self::I64Not),
            0x5F => Some(Self::I64Eq),
            0x60 => Some(Self::F32Add),
            0x61 => Some(Self::F32Sub),
            0x62 => Some(Self::F32Mul),
            0x63 => Some(Self::F32Div),
            0x64 => Some(Self::F32Neg),
            0x65 => Some(Self::I64Ne),
            0x66 => Some(Self::I64LtS),
            0x67 => Some(Self::I64LeS),
            0x68 => Some(Self::I64GtS),
            0x69 => Some(Self::I64GeS),
            0x6A => Some(Self::RefEq),
            0x6B => Some(Self::RefNe),
            0x6C => Some(Self::I64LtU),
            0x6D => Some(Self::I64LeU),
            0x6E => Some(Self::I64GtU),
            0x6F => Some(Self::I64GeU),
            0x70 => Some(Self::F64Add),
            0x71 => Some(Self::F64Sub),
            0x72 => Some(Self::F64Mul),
            0x73 => Some(Self::F64Div),
            0x74 => Some(Self::F64Neg),
            0x75 => Some(Self::F64Sqrt),
            0x76 => Some(Self::F64Eq),
            0x77 => Some(Self::F64Ne),
            0x78 => Some(Self::F64Lt),
            0x79 => Some(Self::F64Le),
            0x7A => Some(Self::F64Gt),
            0x7B => Some(Self::F64Ge),
            0x80 => Some(Self::I32ExtendI64S),
            0x81 => Some(Self::I32ExtendI64U),
            0x82 => Some(Self::I64TruncI32S),
            0x83 => Some(Self::I64TruncI32U),
            0x84 => Some(Self::I32ToF32S),
            0x85 => Some(Self::I32ToF64S),
            0x86 => Some(Self::I64ToF64),
            0x87 => Some(Self::F64ToI32),
            0x88 => Some(Self::F64ToI64),
            0x90 => Some(Self::Alloc),
            0x91 => Some(Self::Free),
            0x92 => Some(Self::I32Load),
            0x93 => Some(Self::I32Store),
            0x94 => Some(Self::I64Load),
            0x95 => Some(Self::I64Store),
            0xA0 => Some(Self::NewObject),
            0xA1 => Some(Self::GetField),
            0xA2 => Some(Self::SetField),
            0xA3 => Some(Self::ArrayGet),
            0xA4 => Some(Self::ArraySet),
            0xA5 => Some(Self::Length),
            0xA6 => Some(Self::NewClosure),
            0xA7 => Some(Self::GetUpvalue),
            0xA8 => Some(Self::SetUpvalue),
            0xA9 => Some(Self::CallWitness),
            0xAA => Some(Self::CallDynamic),
            0xAB => Some(Self::AccessStatic),
            0xAC => Some(Self::AccessWitness),
            0xAD => Some(Self::AccessDynamic),
            0xAE => Some(Self::InlineCacheUpdate),
            0xAF => Some(Self::FieldStore),
            0xB0 => Some(Self::Utf8Concat),
            0xB1 => Some(Self::Utf8LenBytes),
            0xB2 => Some(Self::Utf8LenChars),
            0xB3 => Some(Self::Utf8Substr),
            0xB4 => Some(Self::Utf8Eq),
            0xB5 => Some(Self::Utf8Ne),
            0xB6 => Some(Self::IndexStore),
            0xB7 => Some(Self::ArrayPush),
            0xB8 => Some(Self::GetOrdinalIndex),
            0xB9 => Some(Self::SetOrdinalIndex),
            0xBA => Some(Self::GetOffsetIndex),
            0xBB => Some(Self::SetOffsetIndex),
            0xC0 => Some(Self::BigIntAdd),
            0xC1 => Some(Self::BigIntSub),
            0xC2 => Some(Self::BigIntMul),
            0xD0 => Some(Self::CallIntrinsic),
            0xD1 => Some(Self::CallNative),
            0xD2 => Some(Self::LoadNativeLib),
            0xD3 => Some(Self::GetNativeFunc),
            0xE0 => Some(Self::Simd),
            _ => None,
        }
    }

    /// 判断字节值是否为已定义的头码。
    pub fn is_defined(value: u8) -> bool {
        Self::from_u8(value).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{
        NyarConstant, NyarConstantKind, NyarConstantValue, NyarEncoder, NyarFunction, NyarModuleData,
    };

    fn minimal_module() -> NyarModuleData {
        let code_bytes = vec![
            NyarHeadCode::Const as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            NyarHeadCode::Const as u8,
            0x01,
            0x00,
            0x00,
            0x00,
            NyarHeadCode::I32Add as u8,
            NyarHeadCode::Return as u8,
        ];

        NyarModuleData {
            version: 1,
            name: "test".to_string(),
            constants: vec![
                NyarConstant::new(NyarConstantKind::Integer32, NyarConstantValue::Integer32(10)),
                NyarConstant::new(NyarConstantKind::Integer32, NyarConstantValue::Integer32(20)),
            ],
            functions: vec![NyarFunction::new("main", 0, 0, code_bytes.len() as i32)],
            imports: Vec::new(),
            exports: Vec::new(),
            witness_entries: Vec::new(),
            code_bytes: Some(code_bytes)
        }
    }

    #[test]
    fn opcode_values_match_csharp() {
        assert_eq!(NyarHeadCode::Const as u8, 0x10);
        assert_eq!(NyarHeadCode::I32Add as u8, 0x30);
        assert_eq!(NyarHeadCode::Return as u8, 0x05);
        assert_eq!(NyarHeadCode::STRING_CONCAT as u8, 0xB0);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = super::super::decode::NyarDecoder::new()
            .decode(&encoded)
            .expect("decode");
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}
