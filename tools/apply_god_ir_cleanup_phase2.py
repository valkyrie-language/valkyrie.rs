#!/usr/bin/env python3
"""Phase 2: orphan consumers after God IR schema deletion."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def write(rel: str, text: str) -> None:
    (ROOT / rel).write_text(text, encoding="utf-8", newline="\n")
    print(f"wrote {rel}")


def patch_mod_intrinsics_calls() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/mod.rs"
    t = read(rel)
    t = t.replace("                module_intrinsics,\n", "")
    t = t.replace("        module_intrinsics,\n", "")
    write(rel, t)


def patch_mir_mod_exports() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/mod.rs"
    t = read(rel)
    t = t.replace(
        """pub use ssa::{
    AggregateLayout, AggregateLayoutPlan, FieldLayout, FlagsLayout, IntrinsicBinaryOp, IntrinsicBitwiseOp, IntrinsicCompareOp, IntrinsicOpcode,
    LayoutId, MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirDispatchKind, MirEffectKind, MirField, MirFunction, MirInstruction,
    MirInstructionKind, MirLowerer, MirModule, MirOperand, MirStorageKind, MirStruct, MirTerminator, MirTextConversionSemantics,
    MirTextEncoding, MirTextProjectionBoundary, MirValue, MirValueOrigin, MirValueRef, ReceiverPassingKind, SumTypeLayout, SumVariantLayout,""",
        """pub use ssa::{
    AggregateLayout, AggregateLayoutPlan, FieldLayout, FlagsLayout,
    LayoutId, MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirEffectKind, MirField, MirFunction, MirInstruction,
    MirInstructionKind, MirLowerer, MirModule, MirOperand, MirStorageKind, MirStruct, MirTerminator,
    MirValue, MirValueOrigin, MirValueRef, SumTypeLayout, SumVariantLayout,""",
    )
    write(rel, t)


def patch_frontend_executable_imports() -> None:
    rel = "projects/nyar-language/src/valkyrie/frontend_contract/executable.rs"
    t = read(rel)
    t = t.replace(
        "    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirOperand,\n    MirTerminator, MirValue, MirValueOrigin, MirValueRef, concretize_type_lossy,\n    mir::\n        continuation_runtime::{SuspendLoweringPlan as MirSuspendLoweringPlan, SuspendState as MirSuspendState},\n        ssa::{\n            MirCaseArm, MirCaseChain, MirContinuation, MirDiagnostic, MirFrameLayout, MirFrameSlot, MirSuspendPoint,\n            ReceiverPassingKind as MirReceiverPassingKind,\n        },\n    },\n",
        "    MirBlock, MirBlockRef, MirConstant, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirOperand,\n    MirTerminator, MirValue, MirValueOrigin, MirValueRef, concretize_type_lossy,\n    mir::\n        continuation_runtime::{SuspendLoweringPlan as MirSuspendLoweringPlan, SuspendState as MirSuspendState},\n        ssa::{MirCaseArm, MirCaseChain, MirContinuation, MirDiagnostic, MirFrameLayout, MirFrameSlot, MirSuspendPoint},\n    },\n",
    )
    write(rel, t)


def patch_assembly() -> None:
    rel = "projects/nyar-language/src/valkyrie/assembly/mod.rs"
    t = read(rel)
    t = t.replace("        intrinsics: mir.intrinsics.clone(),\n", "")
    write(rel, t)


def patch_expr_lowering_imports_and_kinds() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/expr_lowering.rs"
    t = read(rel)
    t = t.replace(
        "    IntrinsicOpcode, MirBuilder, MirConstant, MirDispatchKind, MirInstruction, MirInstructionKind, MirOperand, MirStorageKind, MirTerminator,\n    MirValueOrigin, ReceiverPassingKind,\n",
        "    MirBuilder, MirConstant, MirInstruction, MirInstructionKind, MirOperand, MirStorageKind, MirTerminator,\n    MirValueOrigin,\n",
    )
    t = t.replace(
        "kind: MirInstructionKind::TupleNew { element_types, fields, storage, layout_id }",
        "kind: MirInstructionKind::TupleNew { element_types, fields }",
    )
    t = t.replace(
        "kind: MirInstructionKind::FieldGet { object: receiver_operand, field: field.to_string(), storage, layout_id }",
        "kind: MirInstructionKind::FieldGet { object: receiver_operand, field: field.to_string() }",
    )
    t = t.replace(
        "kind: MirInstructionKind::StructNew { type_name: name.to_string(), storage, layout_id, fields }",
        "kind: MirInstructionKind::StructNew { type_name: name.to_string(), fields }",
    )
    t = t.replace(
        "kind: MirInstructionKind::FieldGet { object: object_operand, field: field.to_string(), storage, layout_id }",
        "kind: MirInstructionKind::FieldGet { object: object_operand, field: field.to_string() }",
    )
    t = re.sub(
        r"kind: MirInstructionKind::FieldSet \{\n                        object: object_operand,\n                        field: field\.to_string\(\),\n                        value: value_operand,\n                        storage,\n                        layout_id,\n                    \}",
        "kind: MirInstructionKind::FieldSet { object: object_operand, field: field.to_string(), value: value_operand }",
        t,
    )
    t = re.sub(
        r"kind: MirInstructionKind::FixedArrayNew \{\n                            element_type: element_type\.clone\(\),\n                            length: \*length,\n                            items: item_operands,\n                            storage,\n                            layout_id,\n                        \}",
        "kind: MirInstructionKind::FixedArrayNew { element_type: element_type.clone(), length: *length, items: item_operands }",
        t,
    )
    t = t.replace(
        "kind: MirInstructionKind::AggregateCopy { source: operand.clone(), dest: MirOperand::Value(dest), layout_id }",
        "kind: MirInstructionKind::AggregateCopy { source: operand.clone(), dest: MirOperand::Value(dest) }",
    )
    write(rel, t)


def patch_pattern_lowering() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/pattern_lowering.rs"
    t = read(rel)
    t = t.replace(
        "kind: MirInstructionKind::AggregateCopy { source: operand, dest: MirOperand::Value(value), layout_id }",
        "kind: MirInstructionKind::AggregateCopy { source: operand, dest: MirOperand::Value(value) }",
    )
    t = t.replace(
        "kind: MirInstructionKind::FieldGet { object: value, field: \"payload\".to_string(), storage, layout_id }",
        "kind: MirInstructionKind::FieldGet { object: value, field: \"payload\".to_string() }",
    )
    t = t.replace(
        "kind: MirInstructionKind::FieldGet { object, field: field_name.to_string(), storage, layout_id }",
        "kind: MirInstructionKind::FieldGet { object, field: field_name.to_string() }",
    )
    write(rel, t)


def patch_try_propagate() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/ssa/try_propagate_lowering.rs"
    t = read(rel)
    t = t.replace(
        "kind: MirInstructionKind::StructNew { type_name: class_name.to_string(), storage, layout_id, fields: struct_fields }",
        "kind: MirInstructionKind::StructNew { type_name: class_name.to_string(), fields: struct_fields }",
    )
    write(rel, t)


def patch_validation() -> None:
    rel = "projects/nyar-language/src/valkyrie/mir/validation.rs"
    t = read(rel)
    t = t.replace(
        "use crate::mir::\n    IntrinsicOpcode, MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirEffectKind, MirFunction, MirInstructionKind, MirModule, MirOperand,\n    MirTerminator, MirValueOrigin, MirValueRef,\n};",
        "use crate::mir::\n    MirBlock, MirBlockRef, MirConstant, MirDiagnostic, MirEffectKind, MirFunction, MirInstructionKind, MirModule, MirOperand,\n    MirTerminator, MirValueOrigin, MirValueRef,\n};",
    )
    # Remove StructNew layout validation block
    t = re.sub(
        r"            if let MirInstructionKind::StructNew \{ type_name, storage, layout_id, fields \} = &instruction\.kind \{.*?\n                continue;\n            \}\n",
        "",
        t,
        count=1,
        flags=re.S,
    )
    # Simplify field access validation — drop layout/storage authority
    t = t.replace(
        """            let (field, storage, layout_id, value) = match &instruction.kind {
                MirInstructionKind::FieldGet { field, storage, layout_id, .. } => (field, storage, layout_id, None),
                MirInstructionKind::FieldSet { field, storage, layout_id, value, .. } => (field, storage, layout_id, Some(value)),
                _ => continue,
            };
            let location = format!("block {} instruction {index}", block.id.0);
            let Some(layout_id) = layout_id
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: "aggregate field access has no layout id".to_string(),
                });
            };
            let Some(layout) = module.aggregate_layouts.layouts.iter().find(|layout| layout.id == *layout_id)
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: format!("aggregate field access references unknown layout {layout_id}"),
                });
            };
            if layout.storage != *storage {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: "aggregate field access storage differs from its declared layout".to_string(),
                });
            }
            let Some(declared) = layout.fields.iter().find(|candidate| candidate.name == *field)
            else {
                return Err(SemanticMirContractError {
                    code: "SMIR010",
                    function: function.symbol.clone(),
                    location,
                    detail: format!("aggregate layout {} has no field `{field}`", layout.name),
                });
            };
            if let Some(output) = instruction.output {
                let actual = function.value_types.get(&output).and_then(|ty| concretize_type(ty).ok());
                if actual.as_ref() != Some(&declared.ty) {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "FieldGet result type differs from declared aggregate field type".to_string(),
                    });
                }
            }
            if let Some(MirOperand::Value(value)) = value {
                let actual = function.value_types.get(value).and_then(|ty| concretize_type(ty).ok());
                if actual.as_ref() != Some(&declared.ty) {
                    return Err(SemanticMirContractError {
                        code: "SMIR010",
                        function: function.symbol.clone(),
                        location,
                        detail: "FieldSet value type differs from declared aggregate field type".to_string(),
                    });
                }
            }""",
        "",
    )
    t = re.sub(
        r"\n            if let MirInstructionKind::TextConvert \{.*?\n            \}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"\n            if let MirInstructionKind::Call \{ arguments, \.\. \} = &instruction\.kind \{\n                validate_static_call_resolution.*?\n                if let Some\(opcode\) = intrinsic_opcode \{.*?\n                \}\n            \}\n",
        """
            if let MirInstructionKind::Call { .. } = &instruction.kind {
                validate_static_call_resolution(module, function, &instruction.kind, location.clone())?;
            }
""",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"\nfn validate_text_convert_contract\(.*?\n\}\n\nfn validate_static_call_resolution",
        "\nfn validate_static_call_resolution",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"    let MirInstructionKind::Call \{\} = kind\n    else \{\n        return Ok\(\(\)\);\n    \};\n    if \*dispatch != crate::mir::MirDispatchKind::Static \|\| intrinsic_opcode\.is_some\(\) \{\n        return Ok\(\(\)\);\n    \}\n    let MirOperand::Symbol\(symbol\) = callee",
        """    let MirInstructionKind::Call { callee, .. } = kind
    else {
        return Ok(());
    };
    let MirOperand::Symbol(symbol) = callee""",
        t,
    )
    t = t.replace(
        "let exact_external = module.external_calls.iter().any(|candidate| candidate.dispatch == *dispatch && candidate.symbol == *symbol);",
        "let exact_external = module.external_calls.iter().any(|candidate| candidate.symbol == *symbol);",
    )
    t = re.sub(r"\nfn validate_semantic_intrinsic\(.*?\n\}\n\npub fn validate_module", "\n\npub fn validate_module", t, count=1, flags=re.S)
    # Test helper at bottom
    t = t.replace("MirDispatchKind,", "")
    t = t.replace("dispatch: MirDispatchKind::Static,\n            parameter_types:", "parameter_types:")
    t = re.sub(
        r"MirExternalCallContract \{\n            symbol:.*?\n            dispatch: MirDispatchKind::Static,\n            parameter_types:.*?\n            return_type:.*?\n        \}",
        "MirExternalCallContract { symbol: NamePath::new(vec![Identifier::new(\"foo\")]) }",
        t,
        count=1,
        flags=re.S,
    )
    write(rel, t)


def patch_semantic_mir_contract() -> None:
    rel = "projects/nyar-emitter/src/lowering/features/semantic_mir_contract.rs"
    t = read(rel)
    t = re.sub(
        r"            let ExecutableInstructionKind::Call \{\} = &instruction\.kind\n            else \{\n                continue;\n            \};\n            if !matches!\(dispatch, crate::contracts::DispatchKind::Static\) \|\| intrinsic_opcode\.is_some\(\) \{\n                continue;\n            \}\n            let location",
        "            let ExecutableInstructionKind::Call { callee, .. } = &instruction.kind\n            else {\n                continue;\n            };\n            let location",
        t,
    )
    t = re.sub(
        r"\n            if let ExecutableInstructionKind::TextConvert \{.*?\n            \}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(
        r"\n            if let ExecutableInstructionKind::Call \{ arguments, \.\. \} = &instruction\.kind \{\n                let location = format!.*?\n                \};\n                if arguments\.len\(\) != parameter_types\.len\(\) \{.*?\n                \}\n            \}\n            if let ExecutableInstructionKind::Call \{ arguments, \.\. \} = &instruction\.kind \{\n                validate_intrinsic_call\(.*?\n            \}\n",
        "\n",
        t,
        count=1,
        flags=re.S,
    )
    t = re.sub(r"\nfn validate_text_convert_contract\(.*?\n\}\n\nfn validate_intrinsic_call", "\nfn validate_intrinsic_call", t, count=1, flags=re.S)
    t = re.sub(r"\nfn validate_intrinsic_call\(.*?\n\}\n\nfn instruction_operands", "\n\nfn instruction_operands", t, count=1, flags=re.S)
    t = t.replace(
        """        Call { callee, arguments, witness, effect, .. } => {
            let mut values = vec![callee];
            values.extend(arguments);
            if let Some(witness) = witness {
                values.push(witness);
            }
            if let Some(effect) = effect {
                values.push(effect);
            }
            values
        }""",
        """        Call { callee, arguments } => {
            let mut values = vec![callee];
            values.extend(arguments);
            values
        }""",
    )
    t = t.replace("        TextConvert { value, .. } => vec![value],\n", "")
    write(rel, t)


def patch_blockers() -> None:
    rel = Path(ROOT).parent / "设计，文档，决策，卡点" / "06-blockers" / "01-current-blockers.md"
    t = rel.read_text(encoding="utf-8")
    marker = "**LegacyCall 已砍至 `{ callee, arguments }`"
    addition = """
**本切片（Tier A+B+C+D partial）额外删除：**
- `MirDispatchKind` / `ReceiverPassingKind` / `MirTextEncoding*` / `TextConvert` instruction
- `MirModule.intrinsics` / `MirFunction.intrinsic` / `intrinsic_opcode` re-exports
- aggregate 指令上的 `storage` / `layout_id`（StructNew/TupleNew/FixedArrayNew/FieldGet/FieldSet/AggregateCopy）
- `MirExternalCallContract` 仅保留 `symbol`
- `witness_payload` / `collect_witness_calls_from_mir` MIR 扫描
- Wasm `try_emit_array_length_call` intrinsic 旁路；`emit_intrinsic_opcode` → panic
- V `ssa.v` Binary/Unary/intrinsic 绑定；V `executable.v` Call 胖字段
"""
    if "本切片（Tier A+B+C+D partial）" not in t and marker in t:
        t = t.replace(marker, addition.strip() + "\n" + marker)
        rel.write_text(t, encoding="utf-8", newline="\n")
        print(f"wrote {rel}")


def main() -> None:
    patch_mod_intrinsics_calls()
    patch_mir_mod_exports()
    patch_frontend_executable_imports()
    patch_assembly()
    patch_expr_lowering_imports_and_kinds()
    patch_pattern_lowering()
    patch_try_propagate()
    patch_validation()
    patch_semantic_mir_contract()
    patch_blockers()
    print("phase2 done")


if __name__ == "__main__":
    main()
