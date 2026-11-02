//! 调用 `@nyar/valkyrie` 提供的 `asgard` 命令（经 Node 与 `valkyrie-napi`）。
//!
//! 静态交付线（`build --static`）目前只在兄弟仓 CLI 落地：发布包 `@nyar/valkyrie`
//! 的 `asgard` 把 `build` 透传 Rust argv 桩并以 1 退出，其 native 亦未导出
//! `asgardBuildStatic`（发布包 dist 与 native 导出的探测结论见任务记录）。
//! 因此 [`resolve_asgard`] 把兄弟仓 `bin/asgard.mjs` 排在发布包之前；
//! `VALKYRIE_ASGARD` 可覆盖一切。

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::OnceLock,
};

/// `asgard` 可执行解析结果：程序路径与固定前缀参数（例如 `node` + `asgard.mjs`）。
pub struct AsgardInvocation {
    /// 要执行的程序（`node` 或 PATH 中的 `asgard`）。
    pub program: PathBuf,
    /// 置于用户参数之前的固定前缀（`.mjs` 入口时为脚本路径）。
    pub prefix: Vec<String>,
}

/// 兄弟仓 `nyar-vm.rs` 根目录（与 `valkyrie.rs` 同级）。
fn sibling_repo_root() -> PathBuf {
    workspace_root().join("../nyar-vm.rs")
}

/// 解析 `asgard` 入口。
///
/// 优先级：`VALKYRIE_ASGARD` → 兄弟仓 `projects/nyar.ts/emulators/valkyrie`
/// （静态交付线唯一落点，见模块级说明）→ 工作区 `node_modules/@nyar/valkyrie` →
/// PATH 中的 `asgard`。
pub fn resolve_asgard() -> AsgardInvocation {
    if let Ok(path) = std::env::var("VALKYRIE_ASGARD") {
        return invocation_for_path(PathBuf::from(path));
    }

    let sibling_bin = sibling_repo_root().join("projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs");
    if sibling_bin.is_file() {
        ensure_sibling_dist(&sibling_bin);
        return invocation_for_path(sibling_bin);
    }

    let harness_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = workspace_root();
    for candidate in [harness_dir.join("node_modules/@nyar/valkyrie/bin/asgard.mjs"), root.join("node_modules/@nyar/valkyrie/bin/asgard.mjs")] {
        if candidate.is_file() {
            return invocation_for_path(candidate);
        }
    }

    AsgardInvocation { program: PathBuf::from("asgard"), prefix: Vec::new() }
}

/// 运行 `asgard`，返回完整子进程输出。
pub fn run(args: &[&OsStr]) -> std::io::Result<Output> {
    let invocation = resolve_asgard();
    let mut command = Command::new(&invocation.program);
    for arg in &invocation.prefix {
        command.arg(arg);
    }
    apply_native_addon_override(&mut command, invocation.prefix.first());
    command.args(args).output()
}

/// 本机构建 addon 的显式通道（`VALKYRIE_NATIVE`，loader 文档化的覆盖入口）。
///
/// 走兄弟仓 CLI 时 loader 会按 `node_modules` 链接 → 根级遗留位置 → 正式
/// `platforms/<short>/lib/` 的顺序解析 addon：开发树根级遗留的陈旧二进制会遮蔽
/// 正式 lib/ 通道，使验收实际跑在过期编译器上。此处显式指向本机构建的 lib/
/// addon，与 loader 的正式本地通道同源；调用方已设 `VALKYRIE_NATIVE` 时不覆盖。
pub fn apply_native_addon_override(command: &mut Command, resolved_prefix: Option<&String>) {
    if std::env::var_os("VALKYRIE_NATIVE").is_some() {
        return;
    }
    let sibling = sibling_repo_root();
    let is_sibling_cli = resolved_prefix.is_some_and(|prefix| PathBuf::from(prefix).starts_with(&sibling));
    if !is_sibling_cli {
        return;
    }
    let Some((short, triple)) = host_platform_ids()
    else {
        return;
    };
    let binary =
        sibling.join("projects/nyar.ts/platforms").join(format!("valkyrie-{short}")).join("lib").join(format!("valkyrie.{triple}.node"));
    if binary.is_file() {
        command.env("VALKYRIE_NATIVE", binary.as_os_str());
    }
}

/// 当前宿主 OS/arch 对应的 npm 平台短名与 addon triple（与 loader 的映射一致）。
fn host_platform_ids() -> Option<(&'static str, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some(("win32-x64", "win32-x64-msvc")),
        ("linux", "x86_64") => Some(("linux-x64", "linux-x64-gnu")),
        ("macos", "x86_64") => Some(("darwin-x64", "darwin-x64")),
        ("macos", "aarch64") => Some(("darwin-arm64", "darwin-arm64")),
        _ => None,
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn invocation_for_path(path: PathBuf) -> AsgardInvocation {
    if path.extension().and_then(|ext| ext.to_str()) == Some("mjs") {
        AsgardInvocation { program: PathBuf::from("node"), prefix: vec![path.to_string_lossy().into_owned()] }
    }
    else {
        AsgardInvocation { program: path, prefix: Vec::new() }
    }
}

/// 兄弟仓 `dist/` 是 gitignore 的本地产物；缺失时用其自身构建脚本重建
/// （纯本地 TS 类型剥离，无网络）。native addon 需要完整 cargo release 构建，
/// 不由测试触发，缺失时 CLI 会打印含构建指引的清晰错误。
///
/// `OnceLock` 保证并行测试下重建至多发生一次，避免并发 `rm -rf dist` 竞态。
fn ensure_sibling_dist(sibling_bin: &Path) {
    static READY: OnceLock<()> = OnceLock::new();
    READY.get_or_init(|| {
        let package_dir = sibling_bin.parent().and_then(Path::parent).expect("bin/asgard.mjs 应位于包根之下");
        if package_dir.join("dist").join("cli").join("asgard-cli.js").is_file() {
            return;
        }
        let output = Command::new("node")
            .arg("scripts/build-npm-package-js.mjs")
            .arg("projects/nyar.ts/emulators/valkyrie")
            .current_dir(sibling_repo_root())
            .output()
            .expect("无法运行兄弟仓构建脚本 scripts/build-npm-package-js.mjs");
        assert!(output.status.success(), "兄弟仓 dist 重建失败:\n{}", String::from_utf8_lossy(&output.stderr));
    });
}
