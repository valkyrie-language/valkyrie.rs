#![doc = include_str!("readme.md")]

/// `SSA`-based `MIR` main representation.
pub mod ssa;
pub mod validation;

use valkyrie_parser::{AstParser, ParseError, ValkyrieRoot};

use crate::{hir::ValkyrieCompiler, validation::ControlFlowScheduler};

pub use ssa::{
    MirBlock, MirBlockRef, MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirConstant, MirDispatchKind, MirEffectKind, MirField,
    MirFunction, MirInstruction, MirInstructionKind, MirLowerer, MirModule, MirOperand, MirStruct, MirTerminator, MirValue, MirValueOrigin,
    MirValueRef,
};

impl ValkyrieCompiler {
    /// Lowers parser output into MIR through the current minimal pipeline.
    pub fn lower_root_to_mir(&self, root: &ValkyrieRoot) -> Result<MirModule, ParseError> {
        let hir = self.lower_root(root)?;
        ControlFlowScheduler::validate_hir_module(&hir)?;
        let mir = MirLowerer::lower_module(&hir);
        ControlFlowScheduler::validate_mir_module(&mir)?;
        Ok(mir)
    }

    /// Parses source text and lowers it into MIR.
    pub fn compile_source_to_mir(&self, source: &str) -> Result<MirModule, ParseError> {
        let root = AstParser::parse_root(source)?;
        self.lower_root_to_mir(&root)
    }
}
