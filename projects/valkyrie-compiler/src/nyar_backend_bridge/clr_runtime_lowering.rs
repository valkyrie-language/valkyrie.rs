use crate::lir::{LirModule, LirRuntimeContinuation, LirRuntimeFrame};
use clr_backend::{msil::MsilField, MsilInstruction, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilOpcode, MsilType, MsilTypeDef};
use valkyrie_types::hir::ValkyrieType as HirType;

pub(super) fn lower_runtime_carrier_types(lir: &LirModule) -> Vec<MsilTypeDef> {
    let runtime_namespace = format!("{}.runtime", lir.name);
    let mut types = Vec::new();
    for function in &lir.functions {
        types.extend(function.runtime_frames.iter().map(|frame| lower_runtime_frame_type(frame, &runtime_namespace)));
        types.extend(
            function.runtime_continuations.iter().map(|continuation| lower_runtime_continuation_type(continuation, &runtime_namespace)),
        );
    }
    types
}

fn lower_runtime_frame_type(frame: &LirRuntimeFrame, runtime_namespace: &str) -> MsilTypeDef {
    let qualified_name = format!("{runtime_namespace}.{}", frame.carrier);
    let mut fields = vec![
        MsilField { name: "state_id".to_string(), ty: MsilType::Int32 },
        MsilField { name: "resume_target".to_string(), ty: MsilType::Int32 },
    ];
    fields.extend(
        frame.slots.iter().map(|slot| MsilField {
            name: slot.field_name.clone(),
            ty: slot.value_type.as_ref().map(lower_hir_type).unwrap_or(MsilType::Object),
        }),
    );
    MsilTypeDef {
        full_name: frame.carrier.clone(),
        namespace: runtime_namespace.to_string(),
        fields,
        methods: vec![build_runtime_carrier_constructor(&qualified_name)],
        is_value_type: false,
    }
}

fn lower_runtime_continuation_type(continuation: &LirRuntimeContinuation, runtime_namespace: &str) -> MsilTypeDef {
    let qualified_name = format!("{runtime_namespace}.{}", continuation.carrier);
    let fields = vec![
        MsilField { name: "dispatch_block".to_string(), ty: MsilType::Int32 },
        MsilField { name: "resume_target".to_string(), ty: MsilType::Int32 },
        MsilField { name: "resume_parameter_ref".to_string(), ty: MsilType::Int32 },
        MsilField {
            name: continuation.resume_parameter_field.clone(),
            ty: continuation.resume_parameter_type.as_ref().map(lower_hir_type).unwrap_or(MsilType::Object),
        },
        MsilField { name: "handler_exit".to_string(), ty: MsilType::Int32 },
    ];
    MsilTypeDef {
        full_name: continuation.carrier.clone(),
        namespace: runtime_namespace.to_string(),
        fields,
        methods: vec![build_runtime_carrier_constructor(&qualified_name)],
        is_value_type: false,
    }
}

fn build_runtime_carrier_constructor(qualified_name: &str) -> MsilMethodBody {
    let instructions = vec![
        MsilInstruction { label: None, opcode: MsilOpcode::Ldarg0, operand: None },
        MsilInstruction {
            label: None,
            opcode: MsilOpcode::Call,
            operand: Some(clr_backend::MsilInstructionOperand::Method(MsilMethodRef {
                owner: Some("[mscorlib]System.Object".to_string()),
                name: ".ctor".to_string(),
                signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
            })),
        },
        MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
    ];
    MsilMethodBody {
        method: MsilMethodRef {
            owner: Some(qualified_name.to_string()),
            name: ".ctor".to_string(),
            signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
        },
        locals: Vec::new(),
        instructions,
        max_stack: 1,
        is_entry_point: false,
    }
}

fn lower_hir_type(ty: &HirType) -> MsilType {
    match ty {
        HirType::Void | HirType::Unit => MsilType::Void,
        HirType::Boolean => MsilType::Bool,
        HirType::Character => MsilType::Char,
        HirType::Integer8 { signed } => {
            if *signed {
                MsilType::Int8
            }
            else {
                MsilType::UInt8
            }
        }
        HirType::Integer16 { signed } => {
            if *signed {
                MsilType::Int16
            }
            else {
                MsilType::UInt16
            }
        }
        HirType::Integer32 { signed } => {
            if *signed {
                MsilType::Int32
            }
            else {
                MsilType::UInt32
            }
        }
        HirType::Integer64 { signed } => {
            if *signed {
                MsilType::Int64
            }
            else {
                MsilType::UInt64
            }
        }
        HirType::Float32 => MsilType::Float32,
        HirType::Float64 => MsilType::Float64,
        HirType::Utf8 | HirType::Utf16 => MsilType::String,
        HirType::Named(name) => MsilType::Named(name.to_string()),
        HirType::Array(item) => MsilType::sz_array(lower_hir_type(item)),
        _ => MsilType::Object,
    }
}
