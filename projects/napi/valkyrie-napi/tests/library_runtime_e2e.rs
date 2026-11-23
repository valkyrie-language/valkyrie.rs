//! 普通共享库交付门：`artifact: "library"` 的 x86-64 目标产出带 C ABI 导出面的
//! 共享镜像，独立消费者经系统加载器（`LoadLibraryExW` + `GetProcAddress`）调用，
//! 验证符号解析、跨边界调用与 `FreeLibrary` 卸载，并重复装载确认无一次性泄漏。
//!
//! 首批导出面只开放已验证的标量形状（`i64`/`f64`/`bool`）；数组与文本跨边界
//! 需要不透明句柄或显式序列投影合同，不在本门开放。

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

micro mix_internal(a: i64, b: f64, c: i64) -> f64 {
    return b + (a + c) as f64
}

[export]
micro c_mix(a: i64, b: f64, c: i64) -> f64 {
    return b + (a + c) as f64
}

[export]
micro c_mix_call(a: i64, b: f64, c: i64) -> f64 {
    return app::mix_internal(a, b, c)
}

[export]
micro c_mix4(a: i64, b: f64, c: i64, d: f64) -> f64 {
    return b + d + (a + c) as f64
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
    /// 按模块名/路径查询已装载模块；`UNCHANGED_REFCOUNT` 不为查询增加引用计数。
    fn GetModuleHandleExW(flags: u32, module_name: *const u16, module: *mut *mut c_void) -> i32;
}

/// `GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT`：查询本身不得改变引用计数，
/// 否则「模块是否离开进程」的观测会自己留下引用。
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

/// 装载一次、按同一模块句柄解析全部导出符号；符号缺失即测试失败。
#[cfg(windows)]
fn load_module(dll: &Path) -> *mut c_void {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dll.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let module = unsafe { LoadLibraryExW(wide.as_ptr(), std::ptr::null_mut(), 0) };
    assert!(!module.is_null(), "系统加载器无法装载 {}", dll.display());
    module
}

#[cfg(windows)]
fn symbol(module: *mut c_void, name: &str) -> *mut c_void {
    let owner = format!("{name}\0");
    let address = unsafe { GetProcAddress(module, owner.as_ptr() as *const i8) };
    assert!(!address.is_null(), "导出符号 `{name}` 不在库的导出面中");
    address
}

/// 一轮「装载 → 解析全部导出 → 调用 → 成对释放 → 验证模块离开进程」。
///
/// Windows 按装载次数计引用：一次 `LoadLibraryExW` 只对应一次 `FreeLibrary`，
/// 同一模块句柄被多个符号复用时不得重复装载或重复释放。
#[cfg(windows)]
fn exercise_round(dll: &Path) {
    assert!(!module_present(dll), "本轮开始前模块不得驻留：{}", dll.display());
    let module = load_module(dll);
    let add: extern "system" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(symbol(module, "c_add")) };
    assert_eq!(add(40, 2), 42, "C ABI `i64` 加法必须跨边界成立");
    assert_eq!(add(i64::MIN, 0), i64::MIN, "`i64` 全域载体不受宿主收窄");
    // 入站混合签名：整数与浮点按各自寄存器序列独立编号（Win64 C）。
    let mix: extern "system" fn(i64, f64, i64) -> f64 = unsafe { std::mem::transmute(symbol(module, "c_mix")) };
    assert_eq!(mix(2, 3.5, 5), 10.5, "混合整数/浮点 ABI 参数位置必须与 Win64 C 一致");
    // 出站混合签名：guest 内部调用的寄存器分配必须与入站一致。
    let mix_call: extern "system" fn(i64, f64, i64) -> f64 = unsafe { std::mem::transmute(symbol(module, "c_mix_call")) };
    assert_eq!(mix_call(7, 1.25, 3), 11.25, "guest 内部混合签名调用必须与外部入站使用同一 ABI");
    // 第 4 个位置是浮点（XMM1）：位置编号与寄存器类别分离，整数与浮点各自编号。
    let mix4: extern "system" fn(i64, f64, i64, f64) -> f64 = unsafe { std::mem::transmute(symbol(module, "c_mix4")) };
    assert_eq!(mix4(10, 0.5, 20, 0.25), 30.75, "第 4 个形参必须落在浮点寄存器类别的第 2 个位置（XMM1）");
    let clamp: extern "system" fn(i64, i64, i64) -> i64 = unsafe { std::mem::transmute(symbol(module, "c_clamp")) };
    assert_eq!(clamp(-5, 0, 10), 0, "分支与下界钳制");
    assert_eq!(clamp(5, 0, 10), 5, "区间内原值返回");
    assert_eq!(clamp(50, 0, 10), 10, "上界钳制");
    let negative: extern "system" fn(i64) -> bool = unsafe { std::mem::transmute(symbol(module, "c_is_negative")) };
    assert!(negative(-1) && !negative(0), "`bool` 返回载体必须归一为 0/1");

    assert!(module_present(dll), "装载期间模块必须驻留进程：{}", dll.display());
    let released = unsafe { FreeLibrary(module) };
    assert_ne!(released, 0, "库必须可被系统加载器卸载");
    assert!(!module_present(dll), "成对释放后模块必须离开进程：{}", dll.display());
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
    // 两轮各自「装载一次 → 解析全部导出 → 释放一次 → 验证模块离开 → 重新装载」：
    // 第二轮必须重新装载成功，证明没有一次性状态泄漏，也没有遗留引用。
    exercise_round(&dll);
    exercise_round(&dll);
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

/// 第 5 个形参超出 Win64 寄存器传参上限（整数 4 + 浮点 4，栈上传参未实现）：
/// 必须在 lowering 阶段结构化拒绝并指名函数与上限，不能静默错位传参。
#[cfg(windows)]
#[test]
fn library_with_five_parameters_reports_the_register_limit() {
    let source = r#"namespace app;

[export]
micro five(a: i64, b: f64, c: i64, d: f64, e: i64) -> i64 {
    return a
}
"#;
    let (outcome, root) = build_library("five-params", source);
    assert!(!outcome.ok, "五个形参的导出必须在 lowering 阶段被拒绝: {:?}", outcome.artifacts);
    let error = outcome.error.unwrap_or_default();
    assert!(error.contains("five"), "诊断必须指名被拒函数: {error}");
    assert!(error.contains("5 个形参"), "诊断必须说明形参数量: {error}");
    assert!(error.contains("上限"), "诊断必须给出寄存器传参上限: {error}");
    let _ = std::fs::remove_dir_all(root);
}

/// Linux 真机门：由系统加载器 `dlopen`/`dlsym` 装载 `.so` 并调用 C ABI 导出。
///
/// 只在真实 Linux runner 上编译执行；Windows 上的 `LoadLibraryExW` 门
/// （`shared_library_exports_call_and_unload_through_system_loader`）不是它的替代证据。
#[cfg(target_os = "linux")]
mod linux_shared_object {
    use super::*;
    use std::ffi::{CString, c_void};
    use std::os::unix::ffi::OsStrExt;

    #[link(name = "dl")]
    unsafe extern "C" {
        fn dlopen(filename: *const i8, flag: i32) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const i8) -> *mut c_void;
        fn dlclose(handle: *mut c_void) -> i32;
    }

    /// `RTLD_NOW`：装载时解析全部符号，缺失即装载失败。
    const RTLD_NOW: i32 = 2;

    #[test]
    fn linux_so_loads_and_calls_through_system_loader() {
        let root = std::env::temp_dir().join(format!("valkyrie-library-linux-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let project_dir = write_project(&root);
        let outcome = build_project(&BuildOptions {
            project_dir,
            target: Some("x86_64-unknown-linux-gnu".to_string()),
            output_dir: Some(root.join("dist")),
        });
        assert!(outcome.ok, "Linux 共享库构建失败：{:?}", outcome.error);
        let artifact = outcome
            .artifacts
            .iter()
            .map(PathBuf::from)
            .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("so")))
            .unwrap_or_else(|| panic!("没有产出 `.so` 共享库: {:?}", outcome.artifacts));

        let path = CString::new(artifact.as_os_str().as_bytes()).unwrap();
        let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW) };
        assert!(!handle.is_null(), "系统加载器无法装载 {}", artifact.display());
        let name = CString::new("c_add").unwrap();
        let address = unsafe { dlsym(handle, name.as_ptr()) };
        assert!(!address.is_null(), "`.so` 必须导出 C ABI 符号 `c_add`");
        let add: extern "C" fn(i64, i64) -> i64 = unsafe { std::mem::transmute(address) };
        assert_eq!(add(40, 2), 42, "`.so` 跨边界调用必须成立");
        assert_eq!(unsafe { dlclose(handle) }, 0, "共享库必须可被系统加载器卸载");
        let _ = std::fs::remove_dir_all(&root);
    }
}
