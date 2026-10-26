//! 调用 `@nyar/valkyrie` 提供的 `legion` / `asgard` 命令（经 Node 与 `valkyrie-napi`）。

use std::{
    ffi::OsStr,
    path::PathBuf,
    process::{Command, Output},
};

/// `legion` 可执行解析结果：程序路径与固定前缀参数（例如 `node` + `legion.mjs`）。
pub struct LegionInvocation {
    /// 要执行的程序（`node` 或 PATH 中的 `legion`）。
    pub program: PathBuf,
    /// 置于用户参数之前的固定前缀（`.mjs` 入口时为脚本路径）。
    pub prefix: Vec<String>,
}

/// 解析 `@nyar/valkyrie` `legion` 入口。
///
/// 优先级：`VALKYRIE_LEGION` → 工作区 `node_modules/@nyar/valkyrie` → 兄弟仓
/// `nyar-vm.rs/projects/nyar.ts/emulators/valkyrie` → PATH 中的 `legion`。
pub fn resolve_legion() -> LegionInvocation {
    if let Ok(path) = std::env::var("VALKYRIE_LEGION") {
        return invocation_for_path(PathBuf::from(path));
    }

    let harness_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = workspace_root();
    for candidate in [
        harness_dir.join("node_modules/@nyar/valkyrie/bin/legion.mjs"),
        root.join("node_modules/@nyar/valkyrie/bin/legion.mjs"),
        root.join("../nyar-vm.rs/projects/nyar.ts/emulators/valkyrie/bin/legion.mjs"),
    ] {
        if candidate.is_file() {
            return invocation_for_path(candidate);
        }
    }

    LegionInvocation { program: PathBuf::from("legion"), prefix: Vec::new() }
}

/// 运行 `legion`，返回完整子进程输出。
pub fn run(args: &[&OsStr]) -> std::io::Result<Output> {
    let invocation = resolve_legion();
    let mut command = Command::new(&invocation.program);
    for arg in &invocation.prefix {
        command.arg(arg);
    }
    command.args(args).output()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn invocation_for_path(path: PathBuf) -> LegionInvocation {
    if path.extension().and_then(|ext| ext.to_str()) == Some("mjs") {
        LegionInvocation { program: PathBuf::from("node"), prefix: vec![path.to_string_lossy().into_owned()] }
    }
    else {
        LegionInvocation { program: path, prefix: Vec::new() }
    }
}
