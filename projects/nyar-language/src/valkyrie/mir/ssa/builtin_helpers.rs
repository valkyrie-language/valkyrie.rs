//! Builtin / pattern helpers for MIR lowering.
//!
//! ADR 0010: IntrinsicOpcode tables deleted. Do not restore operator→opcode maps.

use std::collections::BTreeMap;

use crate::types::hir::{HirAttribute, HirFunction, HirModule, HirPattern, ValkyrieType};

use super::{MirOperand, MirValueRef};

pub(super) fn plain_type_pattern_matches(pattern: &HirPattern, ty: &ValkyrieType) -> bool {
    let _ = (pattern, ty);
    false
}

/// DELETED: do not collect IntrinsicOpcode into MirModule.
pub(super) fn collect_intrinsic_opcodes(_module: &HirModule) -> BTreeMap<String, ()> {
    BTreeMap::new()
}

pub(crate) fn intrinsic_opcode_for_function(_function: &HirFunction) -> Option<()> {
    None
}

pub(super) fn extract_intrinsic_opcode(_attribute: &HirAttribute) -> Option<()> {
    None
}

pub(super) fn intrinsic_opcode_output_type(
    _opcode: &(),
    _arguments: &[MirOperand],
    _value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<ValkyrieType> {
    None
}

pub(super) fn intrinsic_opcode_for_operator(_name: &str) -> Option<()> {
    None
}

pub(super) fn array_index_call_output_type(
    _arguments: &[MirOperand],
    _value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<ValkyrieType> {
    None
}
