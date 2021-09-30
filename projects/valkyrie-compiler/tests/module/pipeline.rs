//! 模块系统管线集成测试：验证导入导出流程与作用域隔离。
//!
//! 这些测试验证：
//! - `using` 导入从 `HIR` 经 `MIR` 到 `LIR` 的完整传递
//! - 依赖图模拟 `legion.tools → nyar → std` 的依赖解析
//! - 模块间作用域隔离的边界条件

use valkyrie_compiler::{module::*, LirLowerer, LirTargetLane, MirLowerer};
use valkyrie_types::{
    hir::{HirDocumentation, HirModule, HirStruct, HirVisibility},
    Identifier, NamePath,
};

use nyar_types::QualifiedName;

/// 从 `::` 分隔的字符串构造 `NamePath`。
fn name_path(s: &str) -> NamePath {
    let parts: Vec<Identifier> = s.split("::").filter(|p| !p.is_empty()).map(Identifier::new).collect();
    NamePath::new(parts)
}

/// 构造一个带 `using` 导入的 `HirModule`。
fn build_module_with_imports(name: &str, imports: &[&str]) -> HirModule {
    HirModule {
        name: name_path(name),
        doc: HirDocumentation::default(),
        imports: imports.iter().map(|s| name_path(s)).collect(),
        submodules: Vec::new(),
        functions: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    }
}

/// 构造一个简单的值类型结构体用于测试。
fn build_value_struct(name: &str) -> HirStruct {
    HirStruct {
        name: Identifier::new(name),
        namespace: Vec::new(),
        doc: HirDocumentation::default(),
        generics: Vec::new(),
        parents: Vec::new(),
        fields: Vec::new(),
        methods: Vec::new(),
        properties: Vec::new(),
        visibility: HirVisibility::public(),
        is_value_type: true,
        is_abstract: false,
        is_sealed: false,
        is_final: false,
        is_open: false,
        abstract_methods: Vec::new(),
        abstract_properties: Vec::new(),
        derives: Vec::new(),
    }
}

/// 验证空导入列表的模块能正确通过管线。
#[test]
fn test_empty_imports_pipeline() {
    let module = build_module_with_imports("empty", &[]);

    let mir = MirLowerer::lower_module(&module);
    assert!(mir.imports.is_empty());

    let lir = LirLowerer::lower_module(&module);
    assert!(lir.module_imports.is_empty());
}

/// 验证单个 `using` 导入能从 `HIR` 传递到 `MIR` 再到 `LIR`。
#[test]
fn test_single_import_propagates_through_pipeline() {
    let module = build_module_with_imports("legion.tools", &["nyar"]);

    let mir = MirLowerer::lower_module(&module);
    assert_eq!(mir.imports.len(), 1);
    assert_eq!(mir.imports[0], "nyar");

    let lir = LirLowerer::lower_module(&module);
    assert_eq!(lir.module_imports.len(), 1);
    assert_eq!(lir.module_imports[0], "nyar");
}

/// 验证多个 `using` 导入能完整传递且顺序保持一致。
#[test]
fn test_multiple_imports_preserve_order() {
    let imports = ["std.io", "std.math.graph_theory", "std.data.text.von"];
    let module = build_module_with_imports("legion.tools", &imports);

    let mir = MirLowerer::lower_module(&module);
    assert_eq!(mir.imports.len(), imports.len());
    for (i, import) in imports.iter().enumerate() {
        assert_eq!(mir.imports[i], *import);
    }

    let lir = LirLowerer::lower_module(&module);
    assert_eq!(lir.module_imports.len(), imports.len());
    for (i, import) in imports.iter().enumerate() {
        assert_eq!(lir.module_imports[i], *import);
    }
}

/// 验证 `legion.tools` 的完整导入列表（模拟实际源码 `_.v`）。
#[test]
fn test_legion_tools_imports_match_source() {
    let imports = ["std.data.text.von", "std.io", "std.math.graph_theory"];
    let module = build_module_with_imports("legion", &imports);

    let lir = LirLowerer::lower_module(&module);

    assert_eq!(lir.module_imports.len(), 3);
    assert!(lir.module_imports.contains(&"std.data.text.von".to_string()));
    assert!(lir.module_imports.contains(&"std.io".to_string()));
    assert!(lir.module_imports.contains(&"std.math.graph_theory".to_string()));
}

/// 验证 `legion.tools → nyar → std` 依赖链能在依赖图中正确建模。
#[test]
fn test_legion_tools_dependency_chain_in_graph() {
    let mut graph = DependencyGraph::new();

    let legion_tools = QualifiedName::from("legion.tools");
    let nyar = QualifiedName::from("nyar");
    let std = QualifiedName::from("std");
    let std_data_text_von = QualifiedName::from("std.data.text.von");
    let core = QualifiedName::from("core");

    graph.add_dependency(legion_tools.clone(), nyar.clone());
    graph.add_dependency(legion_tools.clone(), std.clone());
    graph.add_dependency(legion_tools.clone(), std_data_text_von.clone());
    graph.add_dependency(legion_tools.clone(), core.clone());

    assert!(!graph.detect_cycle().is_some());

    let sorted = graph.topological_sort().unwrap();

    let core_pos = sorted.iter().position(|n| *n == core).unwrap();
    let std_pos = sorted.iter().position(|n| *n == std).unwrap();
    let nyar_pos = sorted.iter().position(|n| *n == nyar).unwrap();
    let legion_pos = sorted.iter().position(|n| *n == legion_tools).unwrap();

    assert!(core_pos < legion_pos);
    assert!(std_pos < legion_pos);
    assert!(nyar_pos < legion_pos);
}

/// 验证 `nyar` 的 `auto_link` 隐式依赖在图中正确建模。
#[test]
fn test_nyar_auto_link_dependencies() {
    let mut graph = DependencyGraph::new();

    let nyar = QualifiedName::from("nyar");
    let core = QualifiedName::from("core");
    let std = QualifiedName::from("std");

    graph.add_dependency(nyar.clone(), core.clone());
    graph.add_dependency(nyar.clone(), std.clone());

    let deps = graph.dependencies(&nyar).unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.contains(&core));
    assert!(deps.contains(&std));

    assert!(graph.detect_cycle().is_none());
}

/// 验证 `std` 的多平台适配器依赖在图中正确建模且无循环。
#[test]
fn test_std_multi_platform_dependencies() {
    let mut graph = DependencyGraph::new();

    let std = QualifiedName::from("std");
    let adaptors = [
        "std.adaptor.nyar",
        "std.adaptor.clr",
        "std.adaptor.jvm",
        "std.adaptor.wasm",
        "std.adaptor.wasip1",
        "std.adaptor.wasip2",
        "std.adaptor.windows",
        "std.adaptor.linux",
        "std.adaptor.macos",
    ];

    for adaptor in &adaptors {
        graph.add_dependency(std.clone(), QualifiedName::from(*adaptor));
    }

    assert_eq!(graph.len(), 1 + adaptors.len());
    assert!(graph.detect_cycle().is_none());

    let deps = graph.dependencies(&std).unwrap();
    assert_eq!(deps.len(), adaptors.len());
}

/// 验证跨模块作用域隔离：两个模块可以独立声明同名结构体。
#[test]
fn test_scope_isolation_same_struct_name() {
    let mut module_a = build_module_with_imports("module_a", &[]);
    module_a.structs.push(build_value_struct("Point"));

    let mut module_b = build_module_with_imports("module_b", &[]);
    module_b.structs.push(build_value_struct("Point"));

    let mir_a = MirLowerer::lower_module(&module_a);
    let mir_b = MirLowerer::lower_module(&module_b);

    assert_eq!(mir_a.structs.len(), 1);
    assert_eq!(mir_b.structs.len(), 1);
    assert_eq!(mir_a.structs[0].name, "Point");
    assert_eq!(mir_b.structs[0].name, "Point");

    assert_eq!(mir_a.name, "module_a");
    assert_eq!(mir_b.name, "module_b");
}

/// 验证作用域隔离：模块的导入不会泄漏到另一个模块。
#[test]
fn test_scope_isolation_imports_do_not_leak() {
    let module_a = build_module_with_imports("module_a", &["std.io", "std.math"]);
    let module_b = build_module_with_imports("module_b", &[]);

    let lir_a = LirLowerer::lower_module(&module_a);
    let lir_b = LirLowerer::lower_module(&module_b);

    assert_eq!(lir_a.module_imports.len(), 2);
    assert_eq!(lir_b.module_imports.len(), 0);
}

/// 验证不同 `LIR` 目标通道（`CLR`/`JVM`/`WASM`）都能正确传递导入。
#[test]
fn test_imports_propagate_across_all_lanes() {
    let module = build_module_with_imports("cross_lane", &["nyar", "std"]);

    for lane in [LirTargetLane::Clr, LirTargetLane::Jvm, LirTargetLane::Wasm] {
        let lir = LirLowerer::lower_module_for_lane(&module, lane);
        assert_eq!(lir.module_imports.len(), 2, "通道 {:?} 未正确传递导入", lane);
        assert_eq!(lir.module_imports[0], "nyar");
        assert_eq!(lir.module_imports[1], "std");
    }
}

/// 验证依赖图能检测跨模块的循环依赖。
#[test]
fn test_cross_module_cycle_detection() {
    let mut graph = DependencyGraph::new();

    graph.add_dependency(QualifiedName::from("module_a"), QualifiedName::from("module_b"));
    graph.add_dependency(QualifiedName::from("module_b"), QualifiedName::from("module_c"));
    graph.add_dependency(QualifiedName::from("module_c"), QualifiedName::from("module_a"));

    let cycle = graph.detect_cycle();
    assert!(cycle.is_some());
    let cycle = cycle.unwrap();
    assert!(cycle.len() >= 3);
}

/// 验证依赖图的反向查询：能找到所有依赖某个模块的模块。
#[test]
fn test_reverse_dependency_query() {
    let mut graph = DependencyGraph::new();

    graph.add_dependency(QualifiedName::from("legion.tools"), QualifiedName::from("nyar"));
    graph.add_dependency(QualifiedName::from("other.tools"), QualifiedName::from("nyar"));

    let dependents = graph.dependents(&QualifiedName::from("nyar"));
    assert_eq!(dependents.len(), 2);
    assert!(dependents.contains(&QualifiedName::from("legion.tools")));
    assert!(dependents.contains(&QualifiedName::from("other.tools")));
}

/// 验证拓扑排序结果可用于确定编译顺序：依赖在前，被依赖在后。
#[test]
fn test_topological_sort_for_build_order() {
    let mut graph = DependencyGraph::new();

    graph.add_dependency(QualifiedName::from("legion.tools"), QualifiedName::from("nyar"));
    graph.add_dependency(QualifiedName::from("legion.tools"), QualifiedName::from("std"));
    graph.add_dependency(QualifiedName::from("nyar"), QualifiedName::from("core"));
    graph.add_dependency(QualifiedName::from("std"), QualifiedName::from("core"));

    let sorted = graph.topological_sort().unwrap();

    let core_pos = sorted.iter().position(|n| *n == QualifiedName::from("core")).unwrap();
    let nyar_pos = sorted.iter().position(|n| *n == QualifiedName::from("nyar")).unwrap();
    let std_pos = sorted.iter().position(|n| *n == QualifiedName::from("std")).unwrap();
    let legion_pos = sorted.iter().position(|n| *n == QualifiedName::from("legion.tools")).unwrap();

    assert!(core_pos < nyar_pos);
    assert!(core_pos < std_pos);
    assert!(nyar_pos < legion_pos);
    assert!(std_pos < legion_pos);
}

/// 验证模块名带命名空间的依赖图操作。
#[test]
fn test_qualified_name_dependencies() {
    let mut graph = DependencyGraph::new();

    graph.add_dependency(QualifiedName::from("std::collection::HashMap"), QualifiedName::from("std::hash::Hash"));
    graph.add_dependency(QualifiedName::from("std::collection::HashSet"), QualifiedName::from("std::hash::Hash"));

    let hash_deps = graph.dependents(&QualifiedName::from("std::hash::Hash"));
    assert_eq!(hash_deps.len(), 2);

    let sorted = graph.topological_sort().unwrap();
    let hash_pos = sorted.iter().position(|n| *n == QualifiedName::from("std::hash::Hash")).unwrap();
    let hashmap_pos = sorted.iter().position(|n| *n == QualifiedName::from("std::collection::HashMap")).unwrap();
    let hashset_pos = sorted.iter().position(|n| *n == QualifiedName::from("std::collection::HashSet")).unwrap();

    assert!(hash_pos < hashmap_pos);
    assert!(hash_pos < hashset_pos);
}

/// 验证 `ModuleError::NotFound` 能正确携带搜索路径信息。
#[test]
fn test_module_error_not_found_preserves_search_paths() {
    let name = QualifiedName::from("missing::module");
    let searched = vec![std::path::PathBuf::from("/workspace/projects/missing"), std::path::PathBuf::from("/workspace/projects/module")];

    let error = ModuleError::NotFound { name: name.clone(), searched: searched.clone() };

    let message = format!("{}", error);
    assert!(message.contains("missing::module"));
    assert!(message.contains("/workspace/projects/missing"));
    assert!(message.contains("/workspace/projects/module"));
}

/// 验证 `TopologicalSortError` 能正确显示循环路径。
#[test]
fn test_topological_sort_error_displays_cycle() {
    let cycle = vec![QualifiedName::from("a"), QualifiedName::from("b"), QualifiedName::from("c"), QualifiedName::from("a")];

    let error = TopologicalSortError { cycle };
    let message = format!("{}", error);

    assert!(message.contains("a"));
    assert!(message.contains("b"));
    assert!(message.contains("c"));
    assert!(message.contains("->"));
}
