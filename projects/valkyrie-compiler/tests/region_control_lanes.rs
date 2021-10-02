use clr_backend::MsilOpcode;
use jvm_backend::JvmInstruction;
use nyar_types::SourceID;
use std::path::PathBuf;
use valkyrie_compiler::{
    lir::validation::validate_module, lower_lir_to_jvm_class, lower_lir_to_msil, lower_lir_to_native_assembly, lower_lir_to_wasm_module,
    lower_to_driver_input_for_partition, HostProjectionBoundary, LirLowerer, LirTargetLane, TargetBackendFamily, ValkyrieCompiler,
};

fn lower_for_lane(source: &str, lane: LirTargetLane, version_id: u32) -> valkyrie_compiler::LirModule {
    let compiler = ValkyrieCompiler::new(SourceID { version_id });
    let hir = compiler.compile_source(source).expect("hir ok");
    let lir = LirLowerer::lower_module_for_lane(&hir, lane);
    validate_module(&lir).expect("lir ok");
    lir
}

fn assert_wasm_and_native_accept_region_control(source: &str, version_id: u32) {
    let wasm_lir = lower_for_lane(source, LirTargetLane::Wasm, version_id);
    let wasm = lower_lir_to_wasm_module(&wasm_lir).expect("wasm ok");
    assert!(wasm
        .custom_sections()
        .iter()
        .any(|section| section.name == "nyar.functions" && String::from_utf8_lossy(&section.bytes).contains("main")));

    let native_lir = lower_for_lane(source, LirTargetLane::Native, version_id + 1);
    let native = lower_lir_to_native_assembly(&native_lir).expect("native ok");
    assert!(native.sections.iter().any(|section| section.name == ".text"));
    assert!(native.symbols.iter().any(|symbol| symbol.name == "main"));
}

fn collect_jvm_main_instructions(class_file: &jvm_backend::JvmClassFile) -> Vec<&JvmInstruction> {
    class_file
        .methods
        .iter()
        .filter(|method| method.name == "main")
        .filter_map(|method| method.code.as_ref())
        .flat_map(|code| code.instructions.iter())
        .collect()
}

fn find_clr_entry_method(msil: &clr_backend::MsilModule) -> &clr_backend::MsilMethodBody {
    msil.global_methods.iter().find(|method| method.is_entry_point).expect("expected clr entry method")
}

#[test]
fn basic_continue_passes_on_all_lanes() {
    let source = r#"micro main() {
    let seed: i64 = 0
    while true {
        let seed: i64 = 1
        continue
    }
}"#;

    let clr_lir = lower_for_lane(source, LirTargetLane::Clr, 9101);
    let msil = lower_lir_to_msil(&clr_lir);
    let clr_main = find_clr_entry_method(&msil);
    assert!(clr_main.instructions.iter().any(|instruction| instruction.opcode == MsilOpcode::Br));

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9102);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Goto(_))));

    assert_wasm_and_native_accept_region_control(source, 9103);
}

#[test]
fn basic_break_expr_passes_on_all_lanes() {
    let source = r#"micro main() -> i64 {
    let value: i64 = loop {
        break 7
    }
    return value
}"#;

    let clr_lir = lower_for_lane(source, LirTargetLane::Clr, 9111);
    let msil = lower_lir_to_msil(&clr_lir);
    let clr_main = find_clr_entry_method(&msil);
    assert!(clr_main.instructions.iter().any(|instruction| instruction.opcode == MsilOpcode::Br));
    assert!(clr_main.instructions.iter().any(|instruction| instruction.opcode == MsilOpcode::Ret));

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9112);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Goto(_))));
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::LReturn)));

    assert_wasm_and_native_accept_region_control(source, 9113);
}

#[test]
fn basic_early_return_passes_on_all_lanes() {
    let source = r#"micro main(flag: bool) -> i64 {
    if flag {
        return 1
    }
    return 2
}"#;

    let clr_lir = lower_for_lane(source, LirTargetLane::Clr, 9121);
    let msil = lower_lir_to_msil(&clr_lir);
    let clr_main = find_clr_entry_method(&msil);
    assert!(clr_main.instructions.iter().any(|instruction| instruction.opcode == MsilOpcode::Ret));

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9122);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::LReturn)));

    assert_wasm_and_native_accept_region_control(source, 9123);
}

#[test]
fn labeled_continue_passes_on_jvm_wasm_and_native() {
    let source = r#"micro main() {
    let seed: i64 = 0
    'outer: while true {
        let seed: i64 = 1
        while true {
            continue 'outer
        }
    }
}"#;

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9131);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Goto(_))));

    assert_wasm_and_native_accept_region_control(source, 9132);
}

#[test]
fn labeled_break_expr_passes_on_jvm_wasm_and_native() {
    let source = r#"micro main() -> i64 {
    let value: i64 = 'outer: loop {
        while true {
            break 'outer 7
        }
    }
    return value
}"#;

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9141);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Goto(_))));
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::LReturn)));

    assert_wasm_and_native_accept_region_control(source, 9142);
}

#[test]
fn statement_fallthrough_passes_on_jvm_wasm_and_native() {
    let source = r#"micro main(value: i64) {
    case value {
        case 0:
            fallthrough
        case 1:
            return
        else:
            return
    };
    return
}"#;

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9151);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    let jvm_main_instructions = collect_jvm_main_instructions(&jvm);
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Goto(_))));
    assert!(jvm_main_instructions.iter().any(|instruction| matches!(instruction, JvmInstruction::Return)));

    assert_wasm_and_native_accept_region_control(source, 9152);
}

#[test]
fn jvm_and_native_mangle_non_entry_symbols_before_emit() {
    let source = r#"namespace demo.tools;
micro helper() -> i64 {
    return 7
}

micro main() -> i64 {
    return helper()
}"#;

    let jvm_lir = lower_for_lane(source, LirTargetLane::Jvm, 9161);
    let jvm = lower_lir_to_jvm_class(&jvm_lir).expect("jvm ok");
    assert!(jvm.methods.iter().any(|method| method.name == "main"));
    assert!(jvm.methods.iter().any(|method| method.name.contains("helper") && !method.name.contains("::") && method.name != "helper"));
    assert!(!jvm.methods.iter().any(|method| method.name == "demo.tools::helper"));

    let native_lir = lower_for_lane(source, LirTargetLane::Native, 9162);
    let native = lower_lir_to_native_assembly(&native_lir).expect("native ok");
    assert!(native.symbols.iter().any(|symbol| symbol.name == "main"));
    assert!(native.symbols.iter().any(|symbol| symbol.name.contains("helper") && !symbol.name.contains("::") && symbol.name != "helper"));
    assert!(!native.symbols.iter().any(|symbol| symbol.name == "demo.tools::helper"));
}

#[test]
fn jvm_driver_bridge_skips_unreachable_blocks_for_exit_code_entry() {
    let source = r#"micro find_target(limit: i64): i64 {
    let outer: i64 = 0
    while outer < limit {
        let inner: i64 = 0
        while inner < 4 {
            if outer == 1 {
                if inner == 2 {
                    return outer * 10 + inner
                }
            }
            inner = inner + 1
        }
        outer = outer + 1
    }
    return 0
}

[main]
micro main(): ExitCode {
    if find_target(3) != 12 {
        return ExitCode(1)
    }
    return ExitCode(0)
}"#;

    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9183 });
    let hir = compiler.compile_source(source).expect("hir ok");
    let lir = LirLowerer::lower_module_for_lane(&hir, LirTargetLane::Jvm);
    validate_module(&lir).expect("lir ok");
    let driver_input = lower_to_driver_input_for_partition(
        &hir,
        lir,
        TargetBackendFamily::Jvm,
        HostProjectionBoundary::Jvm,
        PathBuf::from("target/tmp-driver-input"),
        &[],
    )
    .expect("driver input ok");
    let _ = driver_input;
}
