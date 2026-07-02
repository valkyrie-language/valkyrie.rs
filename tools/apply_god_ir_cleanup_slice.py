#!/usr/bin/env python3
"""Destructive God IR cleanup slice — Tier A+B+C+D partial. Compile break expected."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def write(rel: str, text: str) -> None:
    p = ROOT / rel
    p.write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {rel}")


def sub(rel: str, old: str, new: str, count: int = 0) -> None:
    text = read(rel)
    if old not in text:
        raise SystemExit(f"MISSING in {rel}: {old[:80]!r}...")
    replaced = text.replace(old, new, count or -1)
    write(rel, replaced)


def patch_ssa_mod() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/mod.rs"
    t = read(rel)
    t = t.replace("mod intrinsic_opcode;\n", "")
    t = t.replace(
        "pub use intrinsic_opcode::{IntrinsicBinaryOp, IntrinsicBitwiseOp, IntrinsicCompareOp, IntrinsicOpcode};\n",
        "",
    )
    t = t.replace(
        "use builtin_helpers::{collect_intrinsic_opcodes, intrinsic_opcode_for_function, intrinsic_opcode_output_type, plain_type_pattern_matches};\n",
        "use builtin_helpers::{plain_type_pattern_matches};\n",
    )
    # Remove deleted enums block
    t = re.sub(
        r"\n/// Language-level encoding identity.*?\n}\n\n/// Transitional MIR call-shape hint.*?\n}\n\n/// How an instance-method receiver.*?\n}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    # MirModule.intrinsics
    t = re.sub(
        r"\n    /// `symbol → intrinsic opcode`.*?\n    pub intrinsics: BTreeMap<String, IntrinsicOpcode>,\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    # MirExternalCallContract — symbol only
    t = re.sub(
        r"pub struct MirExternalCallContract \{\n    /// Exact source-level symbol.*?\n    pub symbol: NamePath,\n    /// Only an explicitly exported.*?\n    pub dispatch: MirDispatchKind,\n    /// Formal parameters.*?\n    pub parameter_types: Vec<ValkyrieType>,\n    /// Resolved result type\.\n    pub return_type: ValkyrieType,\n\}",
        "pub struct MirExternalCallContract {\n    /// Exact source-level symbol selected by HIR overload resolution.\n    pub symbol: NamePath,\n}",
        t,
        count=1,
        flags=re.S,
    )
    # MirFunction.intrinsic
    t = re.sub(
        r"\n    /// Opcode from `\[intrinsic\(\"\.\"\)\]`.*?\n    pub intrinsic: Option<IntrinsicOpcode>,\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    # StructNew strip storage/layout_id
    t = t.replace(
        """    StructNew {
        /// 结构体类型名。
        type_name: String,
        /// 存储种类：`structure` / tuple / fixed array 等为值布局，`class` 为引用布局。
        storage: value_semantics::MirStorageKind,
        layout_id: Option<value_semantics::LayoutId>,
        /// 字段初始化列表（字段名, 操作数）。
        fields: Vec<(String, MirOperand)>,
    },""",
        """    StructNew {
        /// 结构体类型名。
        type_name: String,
        /// 字段初始化列表（字段名, 操作数）。
        fields: Vec<(String, MirOperand)>,
    },""",
    )
    t = t.replace(
        """    TupleNew {
        element_types: Vec<ValkyrieType>,
        fields: Vec<MirOperand>,
        storage: value_semantics::MirStorageKind,
        layout_id: Option<value_semantics::LayoutId>,
    },""",
        """    TupleNew {
        element_types: Vec<ValkyrieType>,
        fields: Vec<MirOperand>,
    },""",
    )
    t = t.replace(
        """    FixedArrayNew {
        element_type: ValkyrieType,
        length: usize,
        items: Vec<MirOperand>,
        storage: value_semantics::MirStorageKind,
        layout_id: Option<value_semantics::LayoutId>,
    },""",
        """    FixedArrayNew {
        element_type: ValkyrieType,
        length: usize,
        items: Vec<MirOperand>,
    },""",
    )
    t = t.replace(
        """    AggregateCopy {
        source: MirOperand,
        dest: MirOperand,
        layout_id: value_semantics::LayoutId,
    },""",
        """    AggregateCopy {
        source: MirOperand,
        dest: MirOperand,
    },""",
    )
    t = t.replace(
        """    FieldGet {
        /// 字段所属对象。
        object: MirOperand,
        /// 字段名。
        field: String,
        storage: value_semantics::MirStorageKind,
        layout_id: Option<value_semantics::LayoutId>,
    },""",
        """    FieldGet {
        /// 字段所属对象。
        object: MirOperand,
        /// 字段名。
        field: String,
    },""",
    )
    t = t.replace(
        """    FieldSet {
        /// 字段所属对象。
        object: MirOperand,
        /// 字段名。
        field: String,
        /// 待写入的值。
        value: MirOperand,
        storage: value_semantics::MirStorageKind,
        layout_id: Option<value_semantics::LayoutId>,
    },""",
        """    FieldSet {
        /// 字段所属对象。
        object: MirOperand,
        /// 字段名。
        field: String,
        /// 待写入的值。
        value: MirOperand,
    },""",
    )
    # TextConvert variant
    t = re.sub(
        r"\n    /// Convert text through an explicit encoding.*?\n    TextConvert \{.*?\n    \},\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    # collect_external_call_contracts
    t = t.replace(
        """                MirExternalCallContract {
                    symbol,
                    dispatch: MirDispatchKind::Static,
                    parameter_types: function.params.iter().map(|parameter| parameter.ty.clone()).collect(),
                    return_type: function.return_type.clone(),
                }""",
        "MirExternalCallContract { symbol }",
    )
    # Remove intrinsic collection in lower_module_semantic / lower_module
    for pat in [
        r"        let mut module_intrinsics = collect_intrinsic_opcodes\(module\);\n",
        r"                &mut module_intrinsics,\n",
        r"            &mut module_intrinsics,\n",
        r"        for function in &functions \{\n            if let Some\(opcode\) = &function\.intrinsic \{\n                module_intrinsics\.insert\(function\.symbol\.clone\(\), opcode\.clone\(\)\);\n            \}\n        \}\n",
        r"            intrinsics: module_intrinsics,\n",
    ]:
        t = re.sub(pat, "", t)
    # lower_function_semantic intrinsic bits
    t = re.sub(
        r"    let function_intrinsic = intrinsic_opcode_for_function\(function\);\n\n    // Empty intrinsic micros.*?\n    \}\n\n",
        "",
        t,
        count=1,
        flags=re.S,
    )
    t = t.replace("        intrinsic: function_intrinsic,\n", "")
    t = re.sub(
        r"    module_intrinsics: &mut BTreeMap<String, IntrinsicOpcode>,\n",
        "",
        t,
    )
    t = re.sub(
        r"    /// `\[intrinsic\(\"…\"\)\]` micros keyed by stable symbol.*?\n    pub\(super\) intrinsics: BTreeMap<String, IntrinsicOpcode>,\n",
        "",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"        intrinsics: BTreeMap<String, IntrinsicOpcode>,\n",
        "",
        t,
    )
    t = re.sub(
        r"        module_intrinsics\.clone\(\),\n",
        "",
        t,
    )
    t = re.sub(
        r"            intrinsics,\n",
        "",
        t,
    )
    t = re.sub(
        r"    module_intrinsics\.extend\(builder\.intrinsics\);\n",
        "",
        t,
    )
    write(rel, t)


def patch_executable_mod() -> None:
    rel = "projects/nyar-types/src/executable/mod.rs"
    t = read(rel)
    t = t.replace("pub mod intrinsic;\npub use intrinsic::*;\n\n", "")
    t = re.sub(
        r"\n/// Language-level encoding identity.*?\n}\n\n/// Instruction / terminator operand\.",
        "\n\n/// Instruction / terminator operand.",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"\n/// Transitional call-shape hint.*?\n}\n\n/// How an instance-method receiver.*?\n}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    # strip storage/layout from variants (same patterns as MIR)
    for old, new in [
        (
            """    StructNew {
        /// Type name.
        type_name: String,
        /// Storage class.
        storage: StorageKind,
        /// Optional layout id.
        layout_id: Option<LayoutId>,
        /// Field initializers `(name, value)`.
        fields: Vec<(String, Operand)>,
    },""",
            """    StructNew {
        /// Type name.
        type_name: String,
        /// Field initializers `(name, value)`.
        fields: Vec<(String, Operand)>,
    },""",
        ),
        (
            """    TupleNew {
        /// Element types.
        element_types: Vec<NyarType>,
        /// Element values.
        fields: Vec<Operand>,
        /// Storage class.
        storage: StorageKind,
        /// Optional layout id.
        layout_id: Option<LayoutId>,
    },""",
            """    TupleNew {
        /// Element types.
        element_types: Vec<NyarType>,
        /// Element values.
        fields: Vec<Operand>,
    },""",
        ),
        (
            """    FixedArrayNew {
        /// Element type.
        element_type: NyarType,
        /// Element count.
        length: usize,
        /// Element values.
        items: Vec<Operand>,
        /// Storage class.
        storage: StorageKind,
        /// Optional layout id.
        layout_id: Option<LayoutId>,
    },""",
            """    FixedArrayNew {
        /// Element type.
        element_type: NyarType,
        /// Element count.
        length: usize,
        /// Element values.
        items: Vec<Operand>,
    },""",
        ),
        (
            """    AggregateCopy {
        /// Source aggregate.
        source: Operand,
        /// Destination aggregate.
        dest: Operand,
        /// Layout id.
        layout_id: LayoutId,
    },""",
            """    AggregateCopy {
        /// Source aggregate.
        source: Operand,
        /// Destination aggregate.
        dest: Operand,
    },""",
        ),
        (
            """    FieldGet {
        /// Object / aggregate.
        object: Operand,
        /// Field name.
        field: String,
        /// Storage class.
        storage: StorageKind,
        /// Optional layout id.
        layout_id: Option<LayoutId>,
    },""",
            """    FieldGet {
        /// Object / aggregate.
        object: Operand,
        /// Field name.
        field: String,
    },""",
        ),
        (
            """    FieldSet {
        /// Object / aggregate.
        object: Operand,
        /// Field name.
        field: String,
        /// Value to write.
        value: Operand,
        /// Storage class.
        storage: StorageKind,
        /// Optional layout id.
        layout_id: Option<LayoutId>,
    },""",
            """    FieldSet {
        /// Object / aggregate.
        object: Operand,
        /// Field name.
        field: String,
        /// Value to write.
        value: Operand,
    },""",
        ),
    ]:
        t = t.replace(old, new)
    t = re.sub(
        r"\n    /// Convert text through an explicit encoding.*?\n    TextConvert \{.*?\n    \},\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"\n    /// Intrinsic opcode when this micro has an empty intrinsic body\.\n    pub intrinsic: Option<IntrinsicOpcode>,\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_intrinsic_opcode_tombstone() -> None:
    write(
        "projects/nyar-language/src/valkyrie/mir/ssa/intrinsic_opcode.rs",
        """//! DELETED — ADR 0010 / 0011: intrinsic opcode authority removed from Semantic MIR.
//! Do not re-export IntrinsicOpcode from this module.
""",
    )


def patch_suspend_analysis() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/suspend_analysis.rs"
    sub(
        rel,
        """        MirInstructionKind::Call { callee, arguments, .. } => {
            let mut used_values = collect_operand_uses(callee);
            for argument in arguments {
                used_values.extend(collect_operand_uses(argument));
            }
            if let Some(witness) = witness {
                used_values.extend(collect_operand_uses(witness));
            }
            if let Some(effect) = effect {
                used_values.extend(collect_operand_uses(effect));
            }
            used_values
        }""",
        """        MirInstructionKind::Call { callee, arguments } => {
            let mut used_values = collect_operand_uses(callee);
            for argument in arguments {
                used_values.extend(collect_operand_uses(argument));
            }
            used_values
        }""",
    )
    sub(
        rel,
        "        MirInstructionKind::TextConvert { value, .. } => collect_operand_uses(value),\n",
        "",
    )


def patch_lir() -> None:
    rel = "projects/nyar-language/src/valkyrie/lir/mod.rs"
    t = read(rel)
    t = t.replace(
        "        MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirLowerer,\n",
        "        MirBlock, MirBlockRef, MirConstant, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirLowerer,\n",
    )
    t = re.sub(
        r"\n/// Dispatch kind preserved from `MIR`\.\n\[derive\(Debug, Clone, Copy, PartialEq, Eq\)\]\npub\(crate\) enum LirDispatchKind \{.*?\n\}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = t.replace(
        """    Call {
        /// Dispatch strategy chosen upstream.
        dispatch: LirDispatchKind,
        /// Callee operand.
        callee: LirOperand,
        /// Positional arguments.
        arguments: Vec<LirOperand>,
        /// Optional witness operand.
        witness: Option<LirOperand>,
        /// Optional effect operand.
        effect: Option<LirOperand>,
    },""",
        """    Call {
        /// Callee operand.
        callee: LirOperand,
        /// Positional arguments.
        arguments: Vec<LirOperand>,
    },""",
    )
    t = t.replace(
        """        MirInstructionKind::Call {} => {
            LirOperationKind::Call {
                dispatch: lower_dispatch_kind(*dispatch),
                callee: lower_mir_operand(callee.clone()),
                arguments: arguments.iter().cloned().map(lower_mir_operand).collect(),
                witness: witness.clone().map(lower_mir_operand),
                effect: effect.clone().map(lower_mir_operand),
            }
        }""",
        """        MirInstructionKind::Call { callee, arguments } => LirOperationKind::Call {
            callee: lower_mir_operand(callee.clone()),
            arguments: arguments.iter().cloned().map(lower_mir_operand).collect(),
        }""",
    )
    t = re.sub(
        r"\n/// Preserves the upstream dispatch category when lowering `MIR` calls into `LIR`\.\npub fn lower_dispatch_kind.*?\n\}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = t.replace("        MirInstructionKind::StructNew { type_name, storage, layout_id: _, fields }", "        MirInstructionKind::StructNew { type_name, fields }")
    t = t.replace(
        """        MirInstructionKind::StructNew { type_name, fields } => LirOperationKind::StructNew {
            type_name: type_name.clone(),
            storage: *storage,
            fields: fields.iter().map(|(name, value)| (name.clone(), lower_mir_operand(value.clone()))).collect(),
        },""",
        """        MirInstructionKind::StructNew { type_name, fields } => LirOperationKind::StructNew {
            type_name: type_name.clone(),
            storage: crate::valkyrie::mir::MirStorageKind::Value,
            fields: fields.iter().map(|(name, value)| (name.clone(), lower_mir_operand(value.clone()))).collect(),
        },""",
    )
    t = t.replace("        MirInstructionKind::TupleNew { fields, storage, .. }", "        MirInstructionKind::TupleNew { fields, .. }")
    t = t.replace(
        """        MirInstructionKind::TupleNew { fields, .. } => LirOperationKind::StructNew {
            type_name: format!("Tuple{}", fields.len()),
            storage: *storage,""",
        """        MirInstructionKind::TupleNew { fields, .. } => LirOperationKind::StructNew {
            type_name: format!("Tuple{}", fields.len()),
            storage: crate::valkyrie::mir::MirStorageKind::Value,""",
    )
    t = t.replace("        MirInstructionKind::FixedArrayNew { items, storage, .. }", "        MirInstructionKind::FixedArrayNew { items, .. }")
    t = t.replace(
        """        MirInstructionKind::FixedArrayNew { items, .. } => LirOperationKind::StructNew {
            type_name: format!("FixedArray{}", items.len()),
            storage: *storage,""",
        """        MirInstructionKind::FixedArrayNew { items, .. } => LirOperationKind::StructNew {
            type_name: format!("FixedArray{}", items.len()),
            storage: crate::valkyrie::mir::MirStorageKind::Value,""",
    )
    t = t.replace(
        "        MirInstructionKind::FieldGet { object, field, storage: _, layout_id: _ }",
        "        MirInstructionKind::FieldGet { object, field }",
    )
    t = t.replace(
        "        MirInstructionKind::FieldSet { object, field, value, storage: _, layout_id: _ }",
        "        MirInstructionKind::FieldSet { object, field, value }",
    )
    t = re.sub(
        r"\n        MirInstructionKind::TextConvert \{ value, \.\. \} => LirOperationKind::Move \{ source: lower_mir_operand\(value\.clone\(\)\) \},\n",
        "\n",
        t,
    )
    write(rel, t)


def patch_frontend_executable() -> None:
    rel = "projects/nyar-language/src/valkyrie/frontend_contract/executable.rs"
    t = read(rel)
    t = t.replace(
        "    Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, DispatchKind, EffectKind, ExecutableFunction,\n    FrameLayout, FrameSlot, Instruction, InstructionKind, Operand, ReceiverPassingKind, SuspendLoweringPlan, SuspendPoint, SuspendState,\n    Terminator, Value, ValueOrigin, ValueRef,\n    executable::{TextConversionSemantics, TextEncoding, TextProjectionBoundary},\n",
        "    Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, EffectKind, ExecutableFunction,\n    FrameLayout, FrameSlot, Instruction, InstructionKind, Operand, SuspendLoweringPlan, SuspendPoint, SuspendState,\n    Terminator, Value, ValueOrigin, ValueRef,\n",
    )
    t = t.replace(
        "    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirOperand,\n    MirTerminator, MirValue, MirValueOrigin, MirValueRef, concretize_type_lossy,\n    mir::\n        continuation_runtime::{SuspendLoweringPlan as MirSuspendLoweringPlan, SuspendState as MirSuspendState},\n        ssa::{\n            MirCaseArm, MirCaseChain, MirContinuation, MirDiagnostic, MirFrameLayout, MirFrameSlot, MirSuspendPoint,\n            ReceiverPassingKind as MirReceiverPassingKind,\n        },\n    },\n",
        "    MirBlock, MirBlockRef, MirConstant, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirOperand,\n    MirTerminator, MirValue, MirValueOrigin, MirValueRef, concretize_type_lossy,\n    mir::\n        continuation_runtime::{SuspendLoweringPlan as MirSuspendLoweringPlan, SuspendState as MirSuspendState},\n        ssa::{MirCaseArm, MirCaseChain, MirContinuation, MirDiagnostic, MirFrameLayout, MirFrameSlot, MirSuspendPoint},\n    },\n",
    )
    t = re.sub(
        r"\nfn convert_dispatch\(dispatch: MirDispatchKind\) -> DispatchKind \{.*?\n\}\n",
        "\n",
        t,
        flags=re.S,
    )
    t = re.sub(
        r"\nfn convert_receiver\(kind: MirReceiverPassingKind\) -> ReceiverPassingKind \{.*?\n\}\n",
        "\n",
        t,
        flags=re.S,
    )
    t = t.replace("        intrinsic: function.intrinsic,\n", "")
    t = t.replace(
        """        MirInstructionKind::Call {            callee,
            arguments,
} => InstructionKind::Call {            callee: convert_operand(callee),
            arguments: arguments.iter().map(convert_operand).collect(),
},""",
        """        MirInstructionKind::Call { callee, arguments } => InstructionKind::Call {
            callee: convert_operand(callee),
            arguments: arguments.iter().map(convert_operand).collect(),
        },""",
    )
    for old in [
        "type_name, storage, layout_id, fields",
        "element_types, fields, storage, layout_id",
        "element_type, length, items, storage, layout_id",
        "source, dest, layout_id",
        "object, field, storage, layout_id",
        "object, field, value, storage, layout_id",
    ]:
        pass
    t = t.replace(
        "MirInstructionKind::StructNew { type_name, storage, layout_id, fields }",
        "MirInstructionKind::StructNew { type_name, fields }",
    )
    t = t.replace(
        """InstructionKind::StructNew {
            type_name: type_name.clone(),
            storage: *storage,
            layout_id: *layout_id,
            fields: fields.iter().map(|(name, value)| (name.clone(), convert_operand(value))).collect(),
        },""",
        """InstructionKind::StructNew {
            type_name: type_name.clone(),
            fields: fields.iter().map(|(name, value)| (name.clone(), convert_operand(value))).collect(),
        },""",
    )
    t = t.replace(
        "MirInstructionKind::TupleNew { element_types, fields, storage, layout_id }",
        "MirInstructionKind::TupleNew { element_types, fields }",
    )
    t = t.replace(
        """InstructionKind::TupleNew {
            element_types: element_types.iter().map(concretize_type_lossy).collect(),
            fields: fields.iter().map(convert_operand).collect(),
            storage: *storage,
            layout_id: *layout_id,
        },""",
        """InstructionKind::TupleNew {
            element_types: element_types.iter().map(concretize_type_lossy).collect(),
            fields: fields.iter().map(convert_operand).collect(),
        },""",
    )
    t = t.replace(
        "MirInstructionKind::FixedArrayNew { element_type, length, items, storage, layout_id }",
        "MirInstructionKind::FixedArrayNew { element_type, length, items }",
    )
    t = t.replace(
        """InstructionKind::FixedArrayNew {
            element_type: concretize_type_lossy(element_type),
            length: *length,
            items: items.iter().map(convert_operand).collect(),
            storage: *storage,
            layout_id: *layout_id,
        },""",
        """InstructionKind::FixedArrayNew {
            element_type: concretize_type_lossy(element_type),
            length: *length,
            items: items.iter().map(convert_operand).collect(),
        },""",
    )
    t = t.replace(
        "MirInstructionKind::AggregateCopy { source, dest, layout_id }",
        "MirInstructionKind::AggregateCopy { source, dest }",
    )
    t = t.replace(
        "InstructionKind::AggregateCopy { source: convert_operand(source), dest: convert_operand(dest), layout_id: *layout_id }",
        "InstructionKind::AggregateCopy { source: convert_operand(source), dest: convert_operand(dest) }",
    )
    t = t.replace(
        "MirInstructionKind::FieldGet { object, field, storage, layout_id }",
        "MirInstructionKind::FieldGet { object, field }",
    )
    t = t.replace(
        "InstructionKind::FieldGet { object: convert_operand(object), field: field.clone(), storage: *storage, layout_id: *layout_id }",
        "InstructionKind::FieldGet { object: convert_operand(object), field: field.clone() }",
    )
    t = t.replace(
        "MirInstructionKind::FieldSet { object, field, value, storage, layout_id }",
        "MirInstructionKind::FieldSet { object, field, value }",
    )
    t = t.replace(
        """InstructionKind::FieldSet {
            object: convert_operand(object),
            field: field.clone(),
            value: convert_operand(value),
            storage: *storage,
            layout_id: *layout_id,
        },""",
        """InstructionKind::FieldSet {
            object: convert_operand(object),
            field: field.clone(),
            value: convert_operand(value),
        },""",
    )
    t = re.sub(
        r"\n        MirInstructionKind::TextConvert \{.*?\n        \},\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_planning_witness() -> None:
    rel = "projects/nyar-language/src/valkyrie/frontend_contract/planning.rs"
    t = read(rel)
    t = t.replace(
        "    mir::ssa::{MirDispatchKind, MirInstructionKind, MirLowerer, MirModule, MirOperand},\n",
        "    mir::ssa::{MirInstructionKind, MirLowerer, MirModule, MirOperand},\n",
    )
    t = t.replace(
        "    let (witness_tables, witness_calls, witness_capability) = witness_payload(module, &external_call_edges);\n",
        "    let witness_tables = Vec::new();\n    let witness_calls = Vec::new();\n    let witness_capability = false;\n",
    )
    t = re.sub(
        r"\nfn witness_payload\(module: &HirModule.*?\n\}\n\nfn collect_witness_calls_from_mir.*?\n\}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_wasm_intrinsics() -> None:
    rel = "projects/nyar-emitter/src/lowering/backends/wasm/mir/intrinsics.rs"
    t = read(rel)
    t = re.sub(
        r"    pub\(crate\) fn emit_intrinsic_opcode\(&mut self, opcode: IntrinsicOpcode, arguments: &\[MirOperand\], output: Option<MirValueRef>\) \{.*?\n    \}\n",
        '    pub(crate) fn emit_intrinsic_opcode(&mut self, _opcode: IntrinsicOpcode, _arguments: &[MirOperand], _output: Option<MirValueRef>) {\n        panic!("DELETED ADR 0010: intrinsic opcode authority removed");\n    }\n',
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_wasm_calls() -> None:
    rel = "projects/nyar-emitter/src/lowering/backends/wasm/mir/calls.rs"
    t = read(rel)
    t = re.sub(
        r"\n    fn try_emit_array_length_call\(&mut self, callee: &MirOperand, arguments: &\[MirOperand\], output: Option<MirValueRef>\) -> bool \{.*?\n        true\n    \}\n",
        "\n    fn try_emit_array_length_call(&mut self, _callee: &MirOperand, _arguments: &[MirOperand], _output: Option<MirValueRef>) -> bool {\n        false\n    }\n",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_wasm_plan() -> None:
    rel = "projects/nyar-emitter/src/lowering/backends/wasm/mir/plan.rs"
    t = read(rel)
    t = re.sub(
        r"            MirInstructionKind::Call \{ callee, arguments, \.\. \} => \{\n                if let Some\(opcode\) = intrinsic_opcode \{.*?\n                \}\n",
        "            MirInstructionKind::Call { callee, arguments, .. } => {\n                let _ = (callee, arguments);\n",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_nyar_types_lib() -> None:
    rel = "projects/nyar-types/src/lib.rs"
    t = read(rel)
    t = t.replace(
        """        Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, DispatchKind, EffectKind, ExecutableFunction,
        FrameLayout, FrameSlot, Instruction, InstructionKind, IntrinsicBinaryOp, IntrinsicBitwiseOp, IntrinsicCompareOp, IntrinsicOpcode,
        Operand, ReceiverPassingKind, SuspendLoweringPlan, SuspendPoint, SuspendState, Terminator, Value, ValueOrigin, ValueRef,""",
        """        Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, EffectKind, ExecutableFunction,
        FrameLayout, FrameSlot, Instruction, InstructionKind, Operand, SuspendLoweringPlan, SuspendPoint, SuspendState, Terminator, Value, ValueOrigin, ValueRef,""",
    )
    write(rel, t)


def patch_nyar_language_lib() -> None:
    rel = "projects/nyar-language/src/lib.rs"
    t = read(rel)
    t = t.replace(
        "        IntrinsicOpcode, LayoutId, MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirDispatchKind, MirEffectKind, MirFunction,\n",
        "        LayoutId, MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirEffectKind, MirFunction,\n",
    )
    t = t.replace(
        "        MirTextEncoding, MirTextProjectionBoundary, MirValue, MirValueOrigin, MirValueRef, ReceiverPassingKind, SingletonInstancePlan,\n",
        "        MirValue, MirValueOrigin, MirValueRef, SingletonInstancePlan,\n",
    )
    write(rel, t)


def main() -> None:
    patch_ssa_mod()
    patch_executable_mod()
    patch_intrinsic_opcode_tombstone()
    patch_suspend_analysis()
    patch_lir()
    patch_frontend_executable()
    patch_planning_witness()
    patch_wasm_intrinsics()
    patch_wasm_calls()
    patch_wasm_plan()
    patch_nyar_types_lib()
    patch_nyar_language_lib()
    print("done")


if __name__ == "__main__":
    main()
