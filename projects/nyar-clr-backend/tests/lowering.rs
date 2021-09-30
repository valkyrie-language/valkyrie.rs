use nyar::TargetLoweringLane;
use nyar_clr_backend::{lower_lir_to_msil, ClrLirLoweringLane};
use valkyrie_compiler::ValkyrieCompiler;

#[test]
fn lowers_lir_module_into_msil_module() {
    let compiler = ValkyrieCompiler::default();
    let lir = compiler.compile_source_to_lir("micro main() -> i64 {\n    return 0;\n}\n").unwrap();

    let msil = lower_lir_to_msil(&lir);
    assert_eq!(msil.assembly.name, lir.name);
    assert_eq!(msil.global_methods.len(), 1);
    assert!(msil.global_methods[0].is_entry_point);
}

#[test]
fn lowers_through_nyar_lane_protocol() {
    let compiler = ValkyrieCompiler::default();
    let lir = compiler.compile_source_to_lir("micro helper() -> i64 {\n    return 1;\n}\n").unwrap();
    let lane = ClrLirLoweringLane::new();
    let result = lane.lower_partition(lir).unwrap();

    assert_eq!(result.input.assembly.name, result.artifact_name);
}

/// 验证结构体定义能通过完整管线降级为 MSIL 类型定义。
///
/// 源码定义 `Point { x: i32, y: i32 }`，在 `main` 中构造实例并读取字段。
/// 期望 MSIL 模块包含一个 `Point` 类型，带 2 个字段和 1 个自动生成的 `.ctor`。
#[test]
fn lowers_struct_definition_to_msil_type() {
    let source =
        "structure Point {\n    x: i32\n    y: i32\n}\n\nmicro main() -> i32 {\n    let p = Point { x: 1, y: 2 };\n    return p.x;\n}\n";
    let compiler = ValkyrieCompiler::default();
    let lir = compiler.compile_source_to_lir(source).unwrap();

    let msil = lower_lir_to_msil(&lir);

    // 验证结构体类型定义。
    assert_eq!(msil.types.len(), 1, "应生成 1 个用户类型定义");
    let type_def = &msil.types[0];
    assert_eq!(type_def.full_name, "Point");
    assert_eq!(type_def.fields.len(), 2, "Point 应有 2 个字段");
    assert_eq!(type_def.fields[0].name, "x");
    assert_eq!(type_def.fields[1].name, "y");

    // 验证自动生成的构造函数。
    assert_eq!(type_def.methods.len(), 1, "Point 应有 1 个自动生成的 .ctor");
    let ctor = &type_def.methods[0];
    assert_eq!(ctor.method.name, ".ctor");
    assert_eq!(ctor.method.signature.parameter_types.len(), 2, ".ctor 应接收 2 个参数");
}
