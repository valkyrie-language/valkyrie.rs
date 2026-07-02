//! `WASM` 指令解码模型。
//!
//! 定义解码后的指令与操作数结构，供指令解码层与上层消费者使用。

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// 前缀指令的类别。
///
/// `WASM` 使用 `0xFB` 与 `0xFC` 作为前缀字节，其后跟随子操作码，
/// 分别对应 `GC` 提案与杂项数值/内存批量操作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmPrefix {
    /// `GC` 前缀（`0xFB`），对应 `wasm-gc` 提案。
    Gc,
    /// 杂项前缀（`0xFC`），对应数值转换与内存批量操作。
    Miscellaneous,
}

/// 解码后的单条 `WASM` 指令操作数。
///
/// 一条指令可能携带零个或多个操作数，每个操作数以强类型变体表达。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecodedOperand {
    /// 类型段索引，例如 `struct.new` 的类型参数。
    TypeIndex(u32),
    /// 字段索引，例如 `struct.get` 的字段参数。
    FieldIndex(u32),
    /// 计数，例如 `array.new_fixed` 的元素数。
    Count(u32),
    /// 局部变量索引。
    LocalIndex(u32),
    /// 全局变量索引。
    GlobalIndex(u32),
    /// 跳转标签索引。
    LabelIndex(u32),
    /// 函数索引。
    FuncIndex(u32),
    /// 前缀指令的子操作码索引。
    SubOpcode(u32),
    /// 内存操作的对齐与偏移。
    MemArg {
        /// 内存对齐提示（`log2` 字节数）。
        align: u32,
        /// 内存偏移。
        offset: u32,
    },
    /// 块类型的有符号整数表示。
    BlockType(i64),
    /// `br_table` 的跳转目标列表与默认目标。
    BrTargets {
        /// 分支目标标签列表。
        targets: Vec<u32>,
        /// 默认分支目标。
        default: u32,
    },
    /// `i32.const` 立即数。
    ValueI32(i32),
    /// `i64.const` 立即数。
    ValueI64(i64),
    /// `f32.const` 立即数。
    ValueF32(f32),
    /// `f64.const` 立即数。
    ValueF64(f64),
    /// `ref.null` 的引用类型。
    RefNull(u32),
    /// `ref.func` 的函数索引。
    RefFunc(u32),
    /// `select` 的结果类型列表。
    SelectTypes(Vec<u8>),
    /// `call_indirect` 的类型索引与表索引。
    CallIndirect {
        /// 类型段索引。
        type_idx: u32,
        /// 表索引。
        table_idx: u32,
    },
}

/// 解码后的单条 `WASM` 指令。
///
/// 助记符使用 `Cow<'static, str>`：在解码路径上为零分配的静态字符串，
/// 在反序列化路径上可承载拥有的字符串，从而完整支持 `Serialize` / `Deserialize`。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecodedInstruction {
    /// 指令在字节流中的起始偏移。
    pub offset: usize,
    /// 指令操作码字节。
    pub opcode: u8,
    /// 指令助记符。
    pub mnemonic: Cow<'static, str>,
    /// 指令操作数列表。
    pub operands: Vec<DecodedOperand>,
    /// 原始字节大小，对齐 `spy` 的尺寸语义（多为操作码字节数）。
    pub raw_size: usize,
}
