use clr_backend::{MsilInstructionOperand, MsilOpcode};
use nyar_types::SourceID;
use valkyrie_compiler::{
    lir::validation::validate_module, lower_lir_to_jvm_class, lower_lir_to_msil, lower_lir_to_native_assembly, lower_lir_to_wasm_module,
    ValkyrieCompiler,
};

fn find_clr_entry_method(msil: &clr_backend::MsilModule) -> &clr_backend::MsilMethodBody {
    msil.global_methods.iter().find(|method| method.is_entry_point).expect("expected clr entry method")
}

fn assert_clr_frame_resume_roundtrip(source: &str, version_id: u32) {
    let compiler = ValkyrieCompiler::new(SourceID { version_id });
    let lir = compiler.compile_source_to_lir(source).expect("lir ok");
    let function = &lir.functions[0];
    let runtime_frame = function.runtime_frames.first().expect("expected runtime frame");
    let frame_owner = format!("{}.runtime.{}", lir.name, runtime_frame.carrier);
    let resume_skip_prefix = format!("CLR_RUNTIME_FRAME_RESUME_SKIP_{}", runtime_frame.resume_target.0);
    let msil = lower_lir_to_msil(&lir);
    let main_method = find_clr_entry_method(&msil);

    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Stfld,
                Some(MsilInstructionOperand::Field(owner, field))
            ) if owner == &frame_owner && field == "state_id"
        )
    }));
    assert!(main_method
        .instructions
        .iter()
        .any(|instruction| instruction.label.as_deref().is_some_and(|label| label.starts_with(&resume_skip_prefix))));

    for slot in &runtime_frame.slots {
        assert!(main_method.instructions.iter().any(|instruction| {
            matches!(
                (&instruction.opcode, &instruction.operand),
                (
                    MsilOpcode::Stfld,
                    Some(MsilInstructionOperand::Field(owner, field))
                ) if owner == &frame_owner && field == &slot.field_name
            )
        }));
        assert!(main_method.instructions.iter().any(|instruction| {
            matches!(
                (&instruction.opcode, &instruction.operand),
                (
                    MsilOpcode::Ldfld,
                    Some(MsilInstructionOperand::Field(owner, field))
                ) if owner == &frame_owner && field == &slot.field_name
            )
        }));
    }
}

#[test]
fn records_runtime_frames_from_frame_layouts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9001 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .expect("lir ok");

    let function = &lir.functions[0];
    let runtime_frame = function.runtime_frames.first().expect("expected runtime frame");
    let layout = function.frame_layouts.first().expect("expected frame layout");
    assert!(runtime_frame.carrier.contains("$clr_state_"));
    assert_eq!(runtime_frame.state_id, layout.state_id);
    assert_eq!(runtime_frame.resume_target, layout.resume_target);
    assert_eq!(runtime_frame.slots.len(), layout.slots.len());
    for (slot, layout_slot) in runtime_frame.slots.iter().zip(layout.slots.iter()) {
        assert_eq!(slot.field_name, format!("slot_{}", layout_slot.slot_index));
    }
}

#[test]
fn rejects_runtime_continuation_drift() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9002 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    catch raise true {
        else:
            resume true
    }
    return
}
"#,
        )
        .expect("lir ok");
    lir.functions[0].runtime_continuations[0].resume_parameter_field = "wrong".to_string();

    let error = validate_module(&lir).expect_err("expected runtime continuation drift to fail");
    assert!(error.to_string().contains("runtime continuation"));
}

#[test]
fn lowers_runtime_carriers_into_clr_msil_types() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9003 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    catch raise true {
        else:
            resume true
    }
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .expect("lir ok");
    let msil = lower_lir_to_msil(&lir);

    let frame_type =
        msil.types.iter().find(|item| item.full_name == lir.functions[0].runtime_frames[0].carrier).expect("expected clr runtime frame type");
    assert!(frame_type.namespace.ends_with(".runtime"));
    assert!(frame_type.fields.iter().any(|field| field.name == "state_id"));
    for slot in &lir.functions[0].runtime_frames[0].slots {
        assert!(frame_type.fields.iter().any(|field| field.name == slot.field_name));
    }

    let continuation_type = msil
        .types
        .iter()
        .find(|item| item.full_name == lir.functions[0].runtime_continuations[0].carrier)
        .expect("expected clr runtime continuation type");
    assert!(continuation_type.fields.iter().any(|field| field.name == "dispatch_block"));
    assert!(continuation_type.fields.iter().any(|field| field.name == "resume_value"));

    let main_method = find_clr_entry_method(&msil);
    let frame_owner = format!("{}.runtime.{}", lir.name, lir.functions[0].runtime_frames[0].carrier);
    let continuation_owner = format!("{}.runtime.{}", lir.name, lir.functions[0].runtime_continuations[0].carrier);
    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Newobj,
                Some(MsilInstructionOperand::Method(method))
            ) if method.owner.as_deref() == Some(frame_owner.as_str())
        )
    }));
    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Stfld,
                Some(MsilInstructionOperand::Field(owner, field))
            ) if owner == &frame_owner && field == "state_id"
        )
    }));
    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Newobj,
                Some(MsilInstructionOperand::Method(method))
            ) if method.owner.as_deref() == Some(continuation_owner.as_str())
        )
    }));
    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Stfld,
                Some(MsilInstructionOperand::Field(owner, field))
            ) if owner == &continuation_owner && field == "dispatch_block"
        )
    }));
}

#[test]
fn clr_runtime_mainline_consumes_resume_loader_and_handler_exit_cleanup() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9005 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    catch raise true {
        else:
            resume true
    }
    return
}
"#,
        )
        .expect("lir ok");
    let continuation = &lir.functions[0].runtime_continuations[0];
    let continuation_owner = format!("{}.runtime.{}", lir.name, continuation.carrier);
    let handler_exit_label = format!("BB{}", lir.functions[0].continuations[0].handler_exit.0);
    let resume_skip_prefix = format!("CLR_RUNTIME_RESUME_SKIP_{}", lir.functions[0].continuations[0].resume_target.0);
    let msil = lower_lir_to_msil(&lir);
    let main_method = find_clr_entry_method(&msil);

    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Stfld,
                Some(MsilInstructionOperand::Field(owner, field))
            ) if owner == &continuation_owner && field == "resume_value"
        )
    }));
    assert!(main_method.instructions.iter().any(|instruction| {
        matches!(
            (&instruction.opcode, &instruction.operand),
            (
                MsilOpcode::Ldfld,
                Some(MsilInstructionOperand::Field(owner, field))
            ) if owner == &continuation_owner && field == "resume_value"
        )
    }));
    assert!(main_method
        .instructions
        .iter()
        .any(|instruction| instruction.label.as_deref().is_some_and(|label| label.starts_with(&resume_skip_prefix))));

    let handler_exit_index = main_method
        .instructions
        .iter()
        .position(|instruction| instruction.label.as_deref() == Some(handler_exit_label.as_str()))
        .expect("expected handler exit label");
    assert_eq!(main_method.instructions[handler_exit_index].opcode, MsilOpcode::Ldnull);
    assert!(
        matches!(main_method.instructions.get(handler_exit_index + 1), Some(instruction) if matches!(instruction.opcode, MsilOpcode::Stloc0 | MsilOpcode::Stloc1 | MsilOpcode::Stloc2 | MsilOpcode::Stloc3 | MsilOpcode::Stloc))
    );
    assert_eq!(main_method.instructions[handler_exit_index + 2].opcode, MsilOpcode::Ldnull);
    assert!(
        matches!(main_method.instructions.get(handler_exit_index + 3), Some(instruction) if matches!(instruction.opcode, MsilOpcode::Stloc0 | MsilOpcode::Stloc1 | MsilOpcode::Stloc2 | MsilOpcode::Stloc3 | MsilOpcode::Stloc))
    );
}

#[test]
fn keeps_jvm_effect_lowering_as_explicit_diagnostic() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9004 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    future.await
    return
}
"#,
        )
        .expect("lir ok");
    let error = lower_lir_to_jvm_class(&lir).expect_err("jvm should keep explicit effect diagnostic");
    assert!(error.to_string().contains("JVM backend 暂未支持"));
    assert!(error.to_string().contains("Await"));
}

#[test]
fn clr_runtime_roundtrips_yield_resume_state() {
    assert_clr_frame_resume_roundtrip(
        r#"micro main() {
    let kept: bool = true
    yield 1
    let sink: bool = kept
    return
}
"#,
        9006,
    );
}

#[test]
fn clr_runtime_roundtrips_await_resume_state() {
    assert_clr_frame_resume_roundtrip(
        r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        9007,
    );
}

#[test]
fn clr_runtime_roundtrips_block_resume_state() {
    assert_clr_frame_resume_roundtrip(
        r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.block
    let sink: bool = kept
    return
}
"#,
        9008,
    );
}

#[test]
fn keeps_wasm_effect_lowering_as_explicit_diagnostic() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9009 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    future.await
    return
}
"#,
        )
        .expect("lir ok");
    let error = lower_lir_to_wasm_module(&lir).expect_err("wasm should keep explicit effect diagnostic");
    assert!(error.to_string().contains("WASM backend 暂未支持"));
    assert!(error.to_string().contains("Await"));
}

#[test]
fn keeps_native_effect_lowering_as_explicit_diagnostic() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9010 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    future.block
    return
}
"#,
        )
        .expect("lir ok");
    let error = lower_lir_to_native_assembly(&lir).expect_err("native should keep explicit effect diagnostic");
    assert!(error.to_string().contains("Native backend 暂未支持"));
    assert!(error.to_string().contains("AsyncBlock"));
}
