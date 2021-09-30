#![doc = include_str!("readme.md")]
#![allow(missing_docs)]

mod method_body;
mod model;
mod parser;
mod writer;

pub use method_body::{MethodBodyEncoder, MethodBodyError};
pub use model::{
    MsilAssembly, MsilField, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule,
    MsilOpcode, MsilType, MsilTypeDef,
};
pub use parser::{MsilParser, MsilTextMethod};
pub use writer::MsilTextWriter;
