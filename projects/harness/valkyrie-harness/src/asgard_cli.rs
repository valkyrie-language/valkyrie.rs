//! 调用 `@nyar/valkyrie` 提供的 `asgard` 命令（经 Node 与 `valkyrie-napi`）。
//!
//! 静态交付线 CLI 已迁入本仓 `projects/ts/emulators/valkyrie/bin/asgard.mjs`
//! （兄弟仓 `nyar-vm.rs` 的旧落点已随 `odin-vm` 拆仓移除）。发布包
//! `@nyar/valkyrie` 的 `asgard` 把 `build` 透传 Rust argv 桩并以 1 退出，
//! 因此 [`resolve_asgard`] 把仓内 emulator CLI 排在发布包之前；
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

/// 本仓 emulator 包根（静态交付线 CLI 落点）。
fn emulator_package_root() -> PathBuf {
    workspace_root().join("projects/ts/emulators/valkyrie")
}

/// 解析 `asgard` 入口。
///
/// 优先级：`VALKYRIE_ASGARD` → 仓内 `projects/ts/emulators/valkyrie`
/// （静态交付线唯一落点，见模块级说明）→ 工作区 `node_modules/@nyar/valkyrie` →
/// PATH 中的 `asgard`。
pub fn resolve_asgard() -> AsgardInvocation {
    if let Ok(path) = std::env::var("VALKYRIE_ASGARD") {
        return invocation_for_path(PathBuf::from(path));
    }

    let repo_bin = emulator_package_root().join("bin/asgard.mjs");
    if repo_bin.is_file() {
        ensure_emulator_dist(&repo_bin);
        return invocation_for_path(repo_bin);
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
/// 走仓内 CLI 时 loader 会按平台 optionalDependency 链接 → `platforms/<short>/lib/`
/// 的顺序解析 addon；此处显式指向本仓正式 `platforms/<short>/lib/` 二进制，
/// 使验收必然跑在刚构建的 addon 上，不被任何陈旧链接遮蔽。
/// 调用方已设 `VALKYRIE_NATIVE` 时不覆盖。
pub fn apply_native_addon_override(command: &mut Command, resolved_prefix: Option<&String>) {
    if std::env::var_os("VALKYRIE_NATIVE").is_some() {
        return;
    }
    let root = workspace_root();
    let is_repo_cli = resolved_prefix.is_some_and(|prefix| PathBuf::from(prefix).starts_with(&root));
    if !is_repo_cli {
        return;
    }
    let Some((short, triple)) = host_platform_ids()
    else {
        return;
    };
    let binary = root
        .join("projects/ts/platforms")
        .join(format!("valkyrie-{short}"))
        .join("lib")
        .join(format!("valkyrie.{triple}.node"));
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

/// emulator `dist/` 是 gitignore 的本地产物；缺失时用本仓构建脚本重建
/// （纯本地 TS 类型剥离，无网络）。native addon 需要完整 cargo release 构建，
/// 不由测试触发，缺失时 CLI 会打印含构建指引的清晰错误。
///
/// `OnceLock` 保证并行测试下重建至多发生一次，避免并发 `rm -rf dist` 竞态。
fn ensure_emulator_dist(repo_bin: &Path) {
    static READY: OnceLock<()> = OnceLock::new();
    READY.get_or_init(|| {
        let package_dir = repo_bin.parent().and_then(Path::parent).expect("bin/asgard.mjs 应位于包根之下");
        if package_dir.join("dist").join("cli").join("asgard-cli.js").is_file() {
            return;
        }
        let root = workspace_root();
        let output = Command::new("node")
            .arg("scripts/build-npm-package-js.mjs")
            .arg("projects/ts/emulators/valkyrie")
            .current_dir(&root)
            .output()
            .expect("无法运行本仓构建脚本 scripts/build-npm-package-js.mjs");
        assert!(output.status.success(), "emulator dist 重建失败:\n{}", String::from_utf8_lossy(&output.stderr));
    });
}
