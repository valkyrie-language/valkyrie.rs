//! 多导出真实库三形态同源验收：同一份只写 `[export]`（零注解）的 `.v` 源码，
//! 经正式产品入口 `build_project` 分别产出 `.node` / `.pyd` / 普通 `.dll`，
//! 由真实 Node、CPython 与系统加载器装载并调用，逐一核对宿主实际可见名与
//! 执行清单 `library_exports` 的 `source_name` / `logical_name` / `abi` /
//! `physical_artifact` 同源一致。
//!
//! 命名合同（Boost `29b62823`）：
//! - Node：无注解源码名自动 camelCase 投影（`move_zeroes` → `moveZeroes`）；
//! - CPython 与普通 C ABI DLL：发布源码名；
//! - 投影碰撞（`foo_bar` 与 `fooBar`）：Node 写出前 fail-closed，普通 DLL 不受影响；
//! - 无导出面的库：三条 lane 全部拒绝。

use std::{ffi::c_void, fs, path::{Path, PathBuf}, process::Command};

use package_manager_legion::{
    legion::cmds::run::{ExecutionManifest, LibraryExportIndexEntry, EXECUTION_MANIFEST_SCHEMA_VERSION},
    product::{BuildOptions, BuildOutcome, build_project},
};

const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
);

/// 三形态共用的同一份源码：两个带下划线的公开导出 + 一个私有 helper。
/// `long.add` 直接绑定 x86-64 ISA，不依赖 std；helper 被两个公开函数调用。
fn shared_source() -> &'static str {
    r#"namespace app;

[x86_64_operator("long.add")]
micro i64_add(left: i64, right: i64): i64

micro internal_helper(value: i64) -> i64 {
    return i64_add(value, 1)
}

[export]
micro move_zeroes(value: i64) -> i64 {
    return internal_helper(value)
}

[export]
micro two_sum(left: i64, right: i64) -> i64 {
    return internal_helper(i64_add(left, right))
}
"#
}

/// 同源码去掉全部 `[export]`：导出面为空，三条 lane 都必须拒绝。
fn no_export_source() -> String {
    shared_source().replace("[export]\n", "")
}

/// `foo_bar` 与 `fooBar` 在 Node camelCase 投影下撞同一公开名 `fooBar`。
fn collision_source() -> &'static str {
    r#"namespace app;

[export]
micro foo_bar(value: i64) -> i64 {
    return value
}

[export]
micro fooBar(value: i64) -> i64 {
    return value
}
"#
}

const LIBRARY_MANIFEST: &str = r#"{
    name: "app",
    artifact: "library"
}
"#;

fn write_project(root: &Path, source: &str) -> PathBuf {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "export-naming-e2e",
    members: []
}
"#,
    )
    .unwrap();
    fs::write(root.join("legion.von"), LIBRARY_MANIFEST).unwrap();
    fs::write(source_dir.join("lib.v"), source).unwrap();
    fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
    root.to_path_buf()
}

/// `node` / `python` 是宿主合同占位符，不能作为 manifest 的平台身份；
/// 与 Boost `export_naming_manifest` 同源：manifest 用 `native`，请求目标携带完整标签。
fn build_library(prefix: &str, source: &str, target: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-export-naming-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let project_dir = write_project(&root, source);
    let output_dir = root.join("dist").join("out");
    let outcome = build_project(&BuildOptions {
        project_dir,
        target: Some(target.to_string()),
        output_dir: Some(output_dir),
    });
    (outcome, root)
}

fn artifact_with_extension(outcome: &BuildOutcome, extension: &str) -> PathBuf {
    outcome
        .artifacts
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case(extension)))
        .unwrap_or_else(|| panic!("没有产出 `.{extension}` 产物: {:?}", outcome.artifacts))
}

fn manifest_at(outcome: &BuildOutcome) -> ExecutionManifest {
    let artifact = PathBuf::from(outcome.artifacts.first().expect("库交付至少有一个产物"));
    let dir = artifact.parent().expect("产物在输出目录内");
    ExecutionManifest::read_from_output_dir(dir).expect("读取 run-contracts.txt").expect("库构建必须写出 execution manifest")
}

fn export_of(manifest: &ExecutionManifest, source_name: &str) -> LibraryExportIndexEntry {
    manifest
        .library_exports
        .iter()
        .find(|export| export.source_name == source_name)
        .unwrap_or_else(|| panic!("清单缺少源码名 `{source_name}` 的导出: {:?}", manifest.library_exports))
        .clone()
}

/// 清单条目与宿主实际装载事实的逐项对照：公开名、ABI 标签、调用合同、物理模块、
/// 冻结载体签名。
fn assert_export_matches(manifest: &ExecutionManifest, loaded: &Path, source_name: &str, public_name: &str, abi: &str, invocation: &str) {
    let export = export_of(manifest, source_name);
    assert_eq!(export.source_name, source_name, "`source_name` 必须是源码声明名");
    assert_eq!(export.logical_name, public_name, "`logical_name` 必须等于宿主实际可见名 `{public_name}`");
    assert_eq!(export.abi, abi, "`abi` 合同标签必须是 `{abi}`");
    assert_eq!(export.invocation, invocation, "`invocation` 必须是 `{invocation}`");
    assert!(export.signature_available, "清单必须标记载体签名可用");
    assert!(!export.parameters.is_empty(), "清单必须记录冻结形参载体，消费侧不得凭值形状猜");
    assert_eq!(export.parameter_names.len(), export.parameters.len(), "清单形参名必须与载体标签等长，供按名绑定");
    assert!(!export.returns.is_empty(), "清单必须记录冻结返回载体，消费侧不得凭值形状猜");
    assert_eq!(
        Path::new(&export.physical_artifact).file_name().expect("物理产物必须有文件名"),
        loaded.file_name().expect("装载的模块必须有文件名"),
        "`physical_artifact` 必须指向实际装载的模块文件"
    );
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(lp_lib_file_name: *const u16, h_file: *mut c_void, dw_flags: u32) -> *mut c_void;
    fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const i8) -> *mut c_void;
    fn FreeLibrary(h_module: *mut c_void) -> i32;
    fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut c_void) -> i32;
}

/// 查询本身不得改变引用计数，否则「模块是否离开进程」的观测会自己留下引用。
#[cfg(windows)]
const GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT: u32 = 0x0000_0002;

/// 模块当前是否仍驻留进程（按完整路径查询，不增加引用计数）。
#[cfg(windows)]
fn module_present(dll: &Path) -> bool {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dll.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let mut module: *mut c_void = std::ptr::null_mut();
    let found = unsafe {
        GetModuleHandleExW(GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT, wide.as_ptr(), &mut module)
    };
    found != 0 && !module.is_null()
}

/// 一次装载：同一模块句柄解析全部导出（Windows 按装载次数计引用）。
#[cfg(windows)]
fn module_of(dll: &Path) -> *mut c_void {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dll.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let module = unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    assert!(!module.is_null(), "系统加载器无法装载 {}", dll.display());
    module
}

#[cfg(windows)]
fn symbol_of(module: *mut c_void, symbol: &str) -> *mut c_void {
    let name = format!("{symbol}\0");
    unsafe { GetProcAddress(module, name.as_ptr() as *const i8) }
}

/// 与装载成对释放一次，并验证模块确实离开进程：遗留引用会让下一轮装载复用旧模块。
#[cfg(windows)]
fn release(module: *mut c_void, dll: &Path) {
    assert!(module_present(dll), "释放前模块必须驻留：{}", dll.display());
    let released = unsafe { FreeLibrary(module) };
    assert_ne!(released, 0, "库必须可被系统加载器卸载");
    assert!(!module_present(dll), "成对释放后模块必须离开进程：{}", dll.display());
}

/// Node 面：`require()` 后只暴露 camelCase 投影名，源码名与私有 helper 一律不可见。
const NODE_SCRIPT: &str = r#"
const addon = require(process.env.MODULE_PATH);
const expect = (label, actual, want) => {
    if (actual !== want) throw new Error(label + ": got " + String(actual) + " want " + String(want));
};
expect("typeof moveZeroes", typeof addon.moveZeroes, "function");
expect("typeof twoSum", typeof addon.twoSum, "function");
for (const leaked of ["move_zeroes", "two_sum", "internal_helper", "internalHelper"]) {
    if (leaked in addon) throw new Error("source/private name leaked to the Node surface: " + leaked);
}
expect("moveZeroes(41n)", addon.moveZeroes(41n), 42n);
expect("moveZeroes(i64::MAX-1)", addon.moveZeroes(9223372036854775806n), 9223372036854775807n);
expect("twoSum(40n,1n)", addon.twoSum(40n, 1n), 42n);
expect("twoSum(-9n,8n)", addon.twoSum(-9n, 8n), 0n);
process.stdout.write("ok export naming node");
"#;

#[cfg(windows)]
#[test]
fn node_extension_publishes_camel_case_projection_with_matching_manifest() {
    let (outcome, root) = build_library("node", shared_source(), "node");
    assert!(outcome.ok, "Node 扩展构建失败: {:?}", outcome.error);
    let addon = artifact_with_extension(&outcome, "node");

    let output = Command::new("node")
        .arg("-e")
        .arg(NODE_SCRIPT)
        .env("MODULE_PATH", &addon)
        .output()
        .expect("需要 node 宿主装载 .node 产物");
    assert!(
        output.status.success(),
        "node 装载/调用失败: status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok export naming node");

    let manifest = manifest_at(&outcome);
    assert_eq!(manifest.schema_version, EXECUTION_MANIFEST_SCHEMA_VERSION, "清单 schema 必须与工具链生产者同代（导出身份合同自 2 起）");
    assert_export_matches(&manifest, &addon, "move_zeroes", "moveZeroes", "node-api", "node require");
    assert_export_matches(&manifest, &addon, "two_sum", "twoSum", "node-api", "node require");
    assert_eq!(manifest.library_exports.len(), 2, "清单只登记公开导出，私有 helper 不得入册");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn cpython_extension_publishes_source_names_with_matching_manifest() {
    let (outcome, root) = build_library("python", shared_source(), "python");
    assert!(outcome.ok, "CPython 扩展构建失败: {:?}", outcome.error);
    let extension = artifact_with_extension(&outcome, "pyd");

    let script = concat!(
        "import os, sys\n",
        "sys.path.insert(0, os.environ['MODULE_DIR'])\n",
        "import app\n",
        "assert callable(app.move_zeroes) and callable(app.two_sum)\n",
        "assert app.move_zeroes(41) == 42, app.move_zeroes(41)\n",
        "assert app.two_sum(40, 1) == 42\n",
        "assert app.two_sum(-9, 8) == 0\n",
        "for leaked in ('moveZeroes', 'twoSum', 'internal_helper', 'internalHelper'):\n",
        "    assert not hasattr(app, leaked), leaked\n",
    );
    let output = Command::new("python")
        .arg("-c")
        .arg(script)
        .env("MODULE_DIR", extension.parent().unwrap())
        .output()
        .expect("需要 python 宿主装载 .pyd 产物");
    assert!(
        output.status.success(),
        "python 装载/调用失败: status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let manifest = manifest_at(&outcome);
    assert_eq!(manifest.schema_version, EXECUTION_MANIFEST_SCHEMA_VERSION);
    assert_export_matches(&manifest, &extension, "move_zeroes", "move_zeroes", "cpython", "python import");
    assert_export_matches(&manifest, &extension, "two_sum", "two_sum", "cpython", "python import");
    assert_eq!(manifest.library_exports.len(), 2);
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn plain_dll_publishes_source_names_with_matching_manifest() {
    let (outcome, root) = build_library("dll", shared_source(), "x86_64-pc-windows-msvc");
    assert!(outcome.ok, "普通共享库构建失败: {:?}", outcome.error);
    let dll = artifact_with_extension(&outcome, "dll");

    // 一次装载：源码名可解析并跨边界调用，camelCase 投影名与私有 helper 不存在。
    let module = module_of(&dll);
    let move_zeroes: extern "system" fn(i64) -> i64 = unsafe { std::mem::transmute(symbol_of(module, "move_zeroes")) };
    assert!(!symbol_of(module, "move_zeroes").is_null(), "C ABI 必须导出源码名 `move_zeroes`");
    assert_eq!(move_zeroes(41), 42, "`move_zeroes` 跨边界结果必须正确");
    let two_sum: extern "system" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(symbol_of(module, "two_sum")) };
    assert!(!symbol_of(module, "two_sum").is_null(), "C ABI 必须导出源码名 `two_sum`");
    assert_eq!(two_sum(40, 1), 42, "`two_sum` 跨边界结果必须正确");
    for absent in ["moveZeroes", "twoSum", "internal_helper", "internalHelper"] {
        assert!(symbol_of(module, absent).is_null(), "普通 DLL 导出面不得混入 `{absent}`");
    }
    release(module, &dll);

    // 重装：成对释放后模块已离开进程，第二轮必须重新装载并重新解析成功。
    let module = module_of(&dll);
    let move_zeroes: extern "system" fn(i64) -> i64 = unsafe { std::mem::transmute(symbol_of(module, "move_zeroes")) };
    assert_eq!(move_zeroes(1), 2, "重新装载后导出必须仍可调用");
    release(module, &dll);

    let manifest = manifest_at(&outcome);
    assert_eq!(manifest.schema_version, EXECUTION_MANIFEST_SCHEMA_VERSION);
    assert_export_matches(&manifest, &dll, "move_zeroes", "move_zeroes", "c-abi", "LoadLibraryW + GetProcAddress");
    assert_export_matches(&manifest, &dll, "two_sum", "two_sum", "c-abi", "LoadLibraryW + GetProcAddress");
    assert_eq!(manifest.library_exports.len(), 2);
    let _ = fs::remove_dir_all(root);
}

/// 无导出反例：同源码去掉全部 `[export]`，三条 lane 的构建都必须失败并说明导出面为空。
#[cfg(windows)]
#[test]
fn library_without_exports_fails_on_all_three_forms() {
    let source = no_export_source();
    assert!(!source.contains("[export]"), "反例源码必须不含任何 `[export]`");
    for target in ["node", "python", "x86_64-pc-windows-msvc"] {
        let (outcome, root) = build_library(&format!("noexport-{target}"), &source, target);
        assert!(!outcome.ok, "`{target}` 无导出面的库交付必须被拒绝: {:?}", outcome.artifacts);
        let error = outcome.error.unwrap_or_default();
        assert!(error.contains("导出") || error.contains("export"), "`{target}` 失败必须说明导出面为空: {error}");
        let _ = fs::remove_dir_all(root);
    }
}

/// 碰撞反例：`foo_bar` 与 `fooBar` 同投影 `fooBar` → Node 写出前 fail-closed 且诊断
/// 同时列出两个源码名；同一工程普通 DLL 按源码名发布，两个 C 符号都可解析。
#[cfg(windows)]
#[test]
fn projection_collision_fails_closed_for_node_but_plain_dll_builds() {
    let (outcome, root) = build_library("collision-node", collision_source(), "node");
    assert!(!outcome.ok, "Node 投影碰撞必须写出前拒绝: {:?}", outcome.artifacts);
    let error = outcome.error.unwrap_or_default();
    assert!(error.contains("foo_bar"), "碰撞诊断必须列出源码名 `foo_bar`: {error}");
    assert!(error.contains("fooBar"), "碰撞诊断必须列出源码名 `fooBar`: {error}");
    let _ = fs::remove_dir_all(root);

    let (outcome, root) = build_library("collision-dll", collision_source(), "x86_64-pc-windows-msvc");
    assert!(outcome.ok, "同一源码的普通 DLL 按源码名合同不碰撞: {:?}", outcome.error);
    let dll = artifact_with_extension(&outcome, "dll");
    let module = module_of(&dll);
    let foo_bar: extern "system" fn(i64) -> i64 = unsafe { std::mem::transmute(symbol_of(module, "foo_bar")) };
    assert!(!symbol_of(module, "foo_bar").is_null(), "C ABI 必须导出 `foo_bar`");
    let foobar: extern "system" fn(i64) -> i64 = unsafe { std::mem::transmute(symbol_of(module, "fooBar")) };
    assert!(!symbol_of(module, "fooBar").is_null(), "C ABI 必须导出 `fooBar`");
    assert_eq!(foo_bar(1), 1);
    assert_eq!(foobar(2), 2);
    release(module, &dll);
    let _ = fs::remove_dir_all(root);
}
