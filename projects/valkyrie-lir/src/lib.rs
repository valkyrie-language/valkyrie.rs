use crate::encoder::{CanonicalImport, WastEncoder};
pub use crate::{
    dag::DependentGraph,
    encoder::{CanonicalWasi, encode_id, encode_kebab},
    instances::WasiInstance,
    operations::{
        WasiInstruction,
        branch::{JumpBranch, JumpCondition, JumpTable},
        infix::{InfixCall, InfixOperator},
        looping::{LoopEach, LoopRepeat, LoopUntilBody, LoopWhileBody},
    },
    symbols::{
        exports::WasiExport,
        identifiers::WasmIdentifier,
        imports::WasiImport,
        wasi_publisher::{WasiModule, WasiPublisher},
    },
    wasi_types::{
        WasiType,
        array::WasiArrayType,
        enumerations::{WasiEnumeration, WasiSemanticIndex},
        flags::WasiFlags,
        functions::{WasiFunction, WasiFunctionBody, WasiParameter},
        records::{WasiRecordField, WasiRecordType},
        reference::{WasiOwnership, WasiTypeReference},
        resources::WasiResource,
        variants::{WasiVariantItem, WasiVariantType},
    },
    wasi_values::{WasiValue, array::ArrayValue, record::RecordValue},
};

mod dag;
mod encoder;
pub mod helpers;
mod instances;
mod operations;
mod symbols;
mod wasi_types;
mod wasi_values;
