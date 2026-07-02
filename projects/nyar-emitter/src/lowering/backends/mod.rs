pub(crate) use crate::lowering::{
    features::{pattern_matching_contract, singleton},
    sanitize_jvm_method_symbol, sanitize_operation_symbol, sanitize_symbol,
    shared::{executable, interop, intrinsic_opcode, nullable, suspend_sm, suspend_witness, witness_abi},
};

pub(crate) mod clr;
#[path = "clr/mir.rs"]
pub(crate) mod clr_mir;
#[path = "clr/nominal.rs"]
pub(crate) mod clr_nominal;
#[path = "clr/suspend.rs"]
pub(crate) mod clr_suspend;
#[path = "clr/types.rs"]
pub(crate) mod clr_types;
#[path = "clr/witness.rs"]
pub(crate) mod clr_witness;

pub(crate) mod jvm;
#[path = "jvm/mir.rs"]
pub(crate) mod jvm_mir;
#[path = "jvm/suspend.rs"]
pub(crate) mod jvm_suspend;
#[path = "jvm/witness.rs"]
pub(crate) mod jvm_witness;

pub(crate) mod native;
#[path = "native/witness.rs"]
pub(crate) mod witness;

pub(crate) mod nyar_vm;
#[path = "nyar_vm/mir.rs"]
pub(crate) mod nyar_vm_mir;

pub(crate) mod wasm;
