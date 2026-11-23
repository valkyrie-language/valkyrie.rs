//! 普通共享库交付门：`artifact: "library"` 的 x86-64 目标产出带 C ABI 导出面的
//! 共享镜像，独立消费者经系统加载器（`LoadLibraryExW` + `GetProcAddress`）调用，
//! 验证符号解析、跨边界调用与 `FreeLibrary` 卸载，并重复装载确认无一次性泄漏。
//!
//! 首批导出面只开放已验证的标量形状（`i64`/`f64`/`bool`）；数组与文本跨边界
//! 需要不透明句柄或显式序列投影合同，不在本门开放。

use std::cell::RefCell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

fn vendored_projects() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/valkyrie.v/projects")
}

fn workspace_manifest(members: &[String]) -> String {
    let entries = members.iter().map(|member| format!("\"{}\"", member.replace('\\', "/"))).collect::<Vec<_>>().join(",\n        ");
    format!(
        r#"{{
    name: "library-runtime-e2e",
    members: [
        {entries}
    ]
}}
"#
    )
}

/// 首批 C ABI 导出：标量算术、混合 ABI、分支与布尔返回。
fn library_source() -> &'static str {
    r#"namespace app;

[export]
micro c_add(a: i64, b: i64) -> i64 {
    return a + b
}

[export]
micro c_mix(a: i64, b: f64, c: i64) -> f64 {
    return b + (a + c) as f64
}

[export]
micro c_clamp(value: i64, low: i64, high: i64) -> i64 {
    if value < low {
        return low
    }
    if value > high {
        return high
    }
    return value
}

[export]
micro c_is_negative(value: i64) -> bool {
    return value < 0
}
"#
}

const LIBRARY_MANIFEST: &str = r#"{
    name: "app",
    artifact: "library",
    dependencies: { std: { version: "workspace" } }
}
"#;

fn write_project(root: &Path) -> PathBuf {
    let source_dir = root.join("source");
    std::fs::create_dir_all(&source_dir).unwrap();
    let projects = vendored_projects();
    let workspace = workspace_manifest(&[
        projects.join("std").display().to_string(),
        projects.join("std.adaptors._").display().to_string(),
        projects.join("unicode._").display().to_string(),
    ]);
    std::fs::write(root.join("legions.von"), workspace).unwrap();
    std::fs::write(root.join("legion.von"), LIBRARY_MANIFEST).unwrap();
    std::fs::write(source_dir.join("main.v"), library_source()).unwrap();
    root.to_path_buf()
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn LoadLibraryExW(lp_lib_file_name: *const u16, h_file: *mut c_void, dw_flags: u32) -> *mut c_void;
    fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const i8) -> *mut c_void;
    fn FreeLibrary(h_module: *mut c_void) -> i32;
}

// 测试线程内已装载的模块句柄；卸载由 `unload_all` 逐一验证。
#[cfg(windows)]
thread_local! {
    static LOADED_MODULES: RefCell<Vec<*mut c_void>> = const { RefCell::new(Vec::new()) };
}

/// 经系统加载器装载库并按名解析 C ABI 符号；符号缺失即测试失败。
#[cfg(windows)]
fn resolve(dll: &Path, symbol: &str) -> *mut c_void {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dll.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let (module, address) = unsafe {
        let module = LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), 0);
        assert!(!module.is_null(), "系统加载器无法装载 {}", dll.display());
        let name = format!("{symbol}\0");
        let address = GetProcAddress(module, name.as_ptr() as *const i8);
        (module, address)
    };
    assert!(!address.is_null(), "导出符号 `{symbol}` 不在库的导出面中");
    LOADED_MODULES.with(|modules| {
        if !modules.borrow().contains(&module) {
            modules.borrow_mut().push(module);
        }
    });
    address
}

#[cfg(windows)]
fn unload_all() {
    let modules = LOADED_MODULES.with(|held| held.take());
    assert!(!modules.is_empty(), "没有已装载的模块可卸载");
    for module in modules {
        let ok = unsafe { FreeLibrary(module) };
        assert_ne!(ok, 0, "库必须可被系统加载器卸载");
    }
}

#[cfg(windows)]
fn load_once(dll: &Path) {
    let add: extern "system" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(resolve(dll, "c_add")) };
    assert_eq!(add(40, 2), 42, "C ABI `i64` 加法必须跨边界成立");
    assert_eq!(add(i64::MIN, 0), i64::MIN, "`i64` 全域载体不受宿主收窄");
    let mix: extern "system" fn(i64, f64, i64) -> f64 = unsafe { std::mem::transmute(resolve(dll, "c_mix")) };
    assert_eq!(mix(2, 3.5, 5), 10.5, "混合整数/浮点 ABI 参数位置必须与 Win64 C 一致");
    let clamp: extern "system" fn(i64, i64, i64) -> i64 = unsafe { std::mem::transmute(resolve(dll, "c_clamp")) };
    assert_eq!(clamp(-5, 0, 10), 0, "分支与下界钳制");
    assert_eq!(clamp(5, 0, 10), 5, "区间内原值返回");
    assert_eq!(clamp(50, 0, 10), 10, "上界钳制");
    let negative: extern "system" fn(i64) -> bool = unsafe { std::mem::transmute(resolve(dll, "c_is_negative")) };
    assert!(negative(-1) && !negative(0), "`bool` 返回载体必须归一为 0/1");
}

#[cfg(windows)]
fn build_library(prefix: &str, source: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-library-{prefix}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let project_dir = write_project(&root);
    std::fs::write(project_dir.join("source").join("main.v"), source).unwrap();
    let outcome = build_project(&BuildOptions {
        project_dir,
        target: Some("x86_64-pc-windows-msvc".to_string()),
        output_dir: Some(root.join("dist")),
    });
    (outcome, root)
}

#[cfg(windows)]
fn dll_artifact(outcome: &BuildOutcome) -> PathBuf {
    outcome
        .artifacts
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("dll")))
        .unwrap_or_else(|| panic!("没有产出 `.dll` 共享库: {:?}", outcome.artifacts))
}

#[cfg(windows)]
#[test]
fn shared_library_exports_call_and_unload_through_system_loader() {
    let (outcome, root) = build_library("loader", library_source());
    assert!(outcome.ok, "共享库构建失败：{:?}", outcome.error);
    // 交付身份：产物入口符号是首个导出名，且没有 CLI 运行合同（库用途语义）。
    assert_eq!(outcome.entry_symbol.as_deref(), Some("c_add"), "库交付必须报告导出身份");
    let dll = dll_artifact(&outcome);
    // 装载→调用→卸载，重复两轮：第二轮必须重新解析成功，证明没有一次性状态泄漏。
    load_once(&dll);
    unload_all();
    load_once(&dll);
    unload_all();
    let _ = std::fs::remove_dir_all(root);
}

/// 无导出面的库必须在管道门被拒绝（库交付合同，不产出空导出 DLL）。
#[cfg(windows)]
#[test]
fn library_without_exports_fails_closed() {
    let (outcome, root) = build_library("noexport", "namespace app;\nmicro helper(a: i64) -> i64 {\n    return a + 1\n}\n");
    assert!(!outcome.ok, "无导出面的库交付必须被拒绝");
    let _ = std::fs::remove_dir_all(root);
}
