use nyar_types::SourceID;
use valkyrie_compiler::{
    lir::{validation::validate_module, LirTerminator},
    lower_lir_to_msil, ValkyrieCompiler,
};

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
    assert_eq!(runtime_frame.slots[0].field_name, "slot_0");
}

#[test]
fn rejects_runtime_continuation_drift() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9002 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    catch raise true {
        default:
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
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    catch raise true {
        default:
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
    for function in &mut lir.functions {
        for block in &mut function.blocks {
            if let LirTerminator::PerformEffect { resume_target, .. } = &block.terminator {
                block.terminator = LirTerminator::Jump { target: *resume_target, arguments: Vec::new() };
            }
        }
    }
    let msil = lower_lir_to_msil(&lir);

    let frame_type =
        msil.types.iter().find(|item| item.full_name == lir.functions[0].runtime_frames[0].carrier).expect("expected clr runtime frame type");
    assert!(frame_type.namespace.ends_with(".runtime"));
    assert!(frame_type.fields.iter().any(|field| field.name == "state_id"));
    assert!(frame_type.fields.iter().any(|field| field.name == "slot_0"));

    let continuation_type = msil
        .types
        .iter()
        .find(|item| item.full_name == lir.functions[0].runtime_continuations[0].carrier)
        .expect("expected clr runtime continuation type");
    assert!(continuation_type.fields.iter().any(|field| field.name == "dispatch_block"));
    assert!(continuation_type.fields.iter().any(|field| field.name == "resume_value"));
}
