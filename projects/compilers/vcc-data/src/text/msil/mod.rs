#![doc = include_str!("readme.md")]
#![allow(missing_docs)]

mod lexer;
mod method_body;
mod model;
/// ECMA-335 标准 CIL 操作码表与操作数解码。
pub mod op_codes;
mod parser;

pub use method_body::{MethodBodyEncoder, MethodBodyError};
pub use model::{
    MsilAssembly, MsilField, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule,
    MsilOpcode, MsilType, MsilTypeDef,
};
pub use op_codes::{IlOperand, OP_CODES, OpCodeInfo, OperandType, decode_op_code, get_operand_type, lookup_opcode, read_operand};
pub use parser::{MsilParser, MsilTextMethod};
