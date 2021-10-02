#![doc = include_str!("readme.md")]

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use clap::Args;
use miette::{miette, IntoDiagnostic, Report, Result};
use valkyrie_compiler::CanonicalTarget;

/// `legion bootstrap` 的命令参数。
#[derive(Debug, Clone, Args)]
pub struct BootstrapArgs {
    /// 项目目录，默认当前目录。
    #[arg(value_name = "project-dir", default_value = ".")]
    pub project_dir: PathBuf,
    /// seed 路径，用于自举的已有可运行二进制。
    #[arg(long = "seed")]
    pub seed_path: Option<PathBuf>,
    /// 是否跳过 v1/v2 比对。
    #[arg(long)]
    pub skip_compare: bool,
    /// 目标平台，默认 `clr`。
    #[arg(long, default_value = "clr")]
    pub target: CanonicalTarget,
}

/// 自举阶段枚举。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapStage {
    /// 获取 seed。
    Seed,
    /// 编译 v1（seed 编译编译器源码）。
    V1,
    /// 运行 v1 验证。
    V1Run,
    /// 编译 v2（v1 编译同一份编译器源码）。
    V2,
    /// 比对 v1 和 v2。
    Compare,
}

impl std::fmt::Display for BootstrapStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Seed => write!(f, "seed"),
            Self::V1 => write!(f, "v1"),
            Self::V1Run => write!(f, "v1 运行验证"),
            Self::V2 => write!(f, "v2"),
            Self::Compare => write!(f, "v1/v2 比对"),
        }
    }
}

/// 自举结果。
#[derive(Debug)]
pub struct BootstrapResult {
    /// 成功的阶段。
    pub stages_completed: Vec<BootstrapStage>,
    /// 失败的阶段（如果有）。
    pub failed_stage: Option<(BootstrapStage, Report)>,
    /// v1 产物路径。
    pub v1_path: Option<PathBuf>,
    /// v2 产物路径。
    pub v2_path: Option<PathBuf>,
}

impl BootstrapResult {
    /// 检查是否全部成功。
    pub fn is_success(&self) -> bool {
        self.failed_stage.is_none()
    }

    /// 获取错误信息。
    pub fn error(&self) -> Option<&Report> {
        self.failed_stage.as_ref().map(|(_, report)| report)
    }
}

/// 执行自举流程。
pub fn run(args: &BootstrapArgs) -> Result<BootstrapResult> {
    let mut result = BootstrapResult { stages_completed: Vec::new(), failed_stage: None, v1_path: None, v2_path: None };
    let target_str = args.target.to_string();
    // `WASI` 目标的规范字符串形如 `wasm32-unknown-wasi-wasi`，以 `wasm` 开头但包含 `wasi`。
    // 因此用 `contains("wasi")` 检测 `WASI` 目标，`contains("wasm")` 检测所有 `WASM` 系目标。
    let is_wasm_target = target_str.contains("wasm") || target_str.contains("wasi");

    // 阶段 1: 获取 seed。
    let seed_path = match resolve_seed(&args.seed_path, &args.project_dir) {
        Ok(path) => {
            result.stages_completed.push(BootstrapStage::Seed);
            println!("seed: {}", path.display());
            path
        }
        Err(e) => {
            result.failed_stage = Some((BootstrapStage::Seed, e));
            return Ok(result);
        }
    };

    // 阶段 2: 用 seed 编译编译器源码，得到 v1。
    let v1_output_dir = args.project_dir.join("dist").join("v1");
    let source_file = get_compiler_source_path(&args.project_dir);
    match compile_with_seed(&seed_path, &args.project_dir, &v1_output_dir, "v1", &target_str, &source_file) {
        Ok(_) => {
            result.stages_completed.push(BootstrapStage::V1);
            println!("v1: compiled");
        }
        Err(e) => {
            result.failed_stage = Some((BootstrapStage::V1, e));
            return Ok(result);
        }
    }

    // 阶段 3: 运行 v1 验证。
    let v1_artifact_name = get_artifact_name(&args.project_dir);
    let v1_artifact = v1_output_dir.join(artifact_filename(&v1_artifact_name, &target_str));

    // 获取编译器源文件路径（用于 WASM 目标的 v1 运行输入）。
    // 诚实自举要求 v1 实际读取源文件内容，而非使用固定输入值。
    // （source_file 已在阶段 2 计算，此处复用）

    // 对于 WASM 目标，运行 v1 会产生一个输出文件（v1_output.wasm），用于后续比对。
    let v1_output_path = if is_wasm_target { Some(v1_output_dir.join("v1_output.wasm")) } else { None };

    match run_and_verify(&v1_artifact, &target_str, &seed_path, v1_output_path.as_deref(), &source_file) {
        Ok(()) => {
            result.stages_completed.push(BootstrapStage::V1Run);
            println!("v1: verified");
        }
        Err(e) => {
            result.failed_stage = Some((BootstrapStage::V1Run, e));
            return Ok(result);
        }
    }

    // 阶段 4: 用 v1 编译同一份编译器源码，得到 v2。
    // 这是诚实自举的核心：v1 必须是编译器，能编译产生 v2。
    // 对于 WASM 目标：v1 读取源文件，产生 v2_output.wasm（输出依赖于源文件内容）。
    let v2_output_dir = args.project_dir.join("dist").join("v2");
    match compile_with_seed(&v1_artifact, &args.project_dir, &v2_output_dir, "v2", &target_str, &source_file) {
        Ok(_) => {
            result.stages_completed.push(BootstrapStage::V2);
            println!("v2: compiled");
        }
        Err(e) => {
            result.failed_stage = Some((BootstrapStage::V2, e));
            return Ok(result);
        }
    }

    // 阶段 5: 比对 v1 和 v2。
    // 两次编译同一份源码，产物应字节一致（或语义等价）。
    // 对于 WASM 目标，比对的是运行 v1 两次产生的输出文件，而非 v1.wasm 和 v2.wasm 本身。
    if !args.skip_compare {
        let (compare_v1, compare_v2) = if is_wasm_target {
            // WASM 目标：比对 v1_output.wasm 和 v2_output.wasm（运行 v1 两次的输出）
            let v2_output = v2_output_dir.join("v2_output.wasm");
            (v1_output_path.unwrap(), v2_output)
        }
        else {
            // CLR 目标：比对 v1.exe 和 v2.exe
            let v2_artifact = v2_output_dir.join(artifact_filename(&v1_artifact_name, &target_str));
            (v1_artifact.clone(), v2_artifact)
        };

        match compare_artifacts(&compare_v1, &compare_v2) {
            Ok(()) => {
                result.stages_completed.push(BootstrapStage::Compare);
                println!("v1/v2 比对: 一致");
            }
            Err(e) => {
                result.failed_stage = Some((BootstrapStage::Compare, e));
                return Ok(result);
            }
        }
    }
    else {
        println!("v1/v2 比对: skipped");
    }

    result.v1_path = Some(v1_artifact.clone());
    result.v2_path = Some(v2_output_dir.join(artifact_filename(&v1_artifact_name, &target_str)));
    Ok(result)
}

/// 解析 seed 路径。
///
/// 优先级：
/// 1. 用户指定的 --seed 参数
/// 2. 使用当前二进制作为 seed（self-seed）
///
/// 注意：seed 始终是当前 `legion.exe`（编译器自身），不是之前的 v1 产物。
/// v1 产物是编译结果，不具备编译能力（除非 CLR 后端已能 lowering 完整编译器）。
fn resolve_seed(seed_path: &Option<PathBuf>, _project_dir: &Path) -> Result<PathBuf> {
    if let Some(path) = seed_path {
        if !path.exists() {
            return Err(miette!("seed 文件不存在: {}", path.display()));
        }
        return Ok(strip_verbatim_prefix(path));
    }

    // 使用当前二进制 self-seed。
    let current_exe = std::env::current_exe().into_diagnostic().map_err(|error| error.wrap_err("获取当前可执行文件路径失败"))?;
    let current_exe = fs::canonicalize(current_exe).into_diagnostic().map_err(|error| error.wrap_err("解析 seed 路径失败"))?;

    // 移除 `\\?\` 前缀，否则 `Command::new` 在 Windows 上会 panic。
    Ok(strip_verbatim_prefix(&current_exe))
}

/// 查找本机 `wasmtime` CLI。
///
/// `WASI` 目标不生成 `.mjs` 启动壳，而是直接通过本机 `wasmtime` 运行 `WASM` 模块。
fn find_wasmtime_cli() -> Result<PathBuf> {
    let path = env::var_os("PATH").ok_or_else(|| miette!("未设置 PATH，无法查找本机 `wasmtime`"))?;
    let extensions = executable_extensions();
    for dir in env::split_paths(&path) {
        for candidate in candidate_command_paths(&dir, "wasmtime", &extensions) {
            if candidate.is_file() {
                return Ok(strip_verbatim_prefix(&candidate));
            }
        }
    }

    Err(miette!("未在 PATH 中找到本机 `wasmtime`，请先安装并确保可直接执行"))
}

/// 移除 Windows extended-length path 前缀 `\\?\`。
fn strip_verbatim_prefix(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    }
    else {
        path.to_path_buf()
    }
}

/// 使用指定的 seed 编译项目。
///
/// 对于原生 seed（`legion.exe`）：直接运行 `seed build <project_dir> --target <target> --output <output_dir>`。
/// 对于 WASM seed（`.wasm` 文件）：
///   - `wasm` 目标：通过 `node <seed>.mjs <source_file> <output>` 运行微编译器
///   - `wasi` 目标：当前只保留“直接运行本机 `wasmtime` 验证产物”的路径，
///     旧 `wasi_host` 源文件驱动自举链已移除，因此不再支持 `wasm seed -> 再编译`
///   微编译器读取源文件内容，产生一个依赖于源文件内容的 WASM 输出文件。
///   这是 WASM/WASI 自举的"编译"等价物。
fn compile_with_seed(seed: &Path, project_dir: &Path, output_dir: &Path, stage_name: &str, target: &str, source_file: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).into_diagnostic().map_err(|error| error.wrap_err(format!("创建输出目录失败 {}", output_dir.display())))?;

    let normalized_seed = strip_verbatim_prefix(seed);
    let normalized_project_dir = strip_verbatim_prefix(project_dir);
    let normalized_output_dir = strip_verbatim_prefix(output_dir);

    // 检测 seed 是否为 WASM 文件（v1 产物）。
    let is_wasm_seed = normalized_seed.extension().is_some_and(|ext| ext == "wasm");
    // `WASI` 目标字符串形如 `wasm32-unknown-wasi-wasi`，用 `contains("wasi")` 检测。
    let is_wasi_target = target.contains("wasi");

    if is_wasm_seed {
        if !source_file.exists() {
            return Err(miette!(
                r#"{stage_name} 源文件不存在: {source_file}
诚实自举要求 v1 实际读取源文件内容"#,
                stage_name = stage_name,
                source_file = source_file.display()
            ));
        }

        // 对于 V2 阶段，输出文件名为 v2_output.wasm；对于 V1 阶段，由调用方处理。
        let output_file =
            if stage_name == "v2" { normalized_output_dir.join("v2_output.wasm") } else { normalized_output_dir.join("v1_output.wasm") };
        let mut cmd = if is_wasi_target {
            return Err(miette!(
                r#"{stage_name} 当前不支持 `WASI` wasm seed 自举
旧 `valkyrie-wasi-host` 源文件驱动链已移除；
现在只保留本机 `wasmtime` 直接运行产物的验证路径。"#,
                stage_name = stage_name
            ));
        }
        else {
            // WASM 目标：通过 node 运行 .mjs 启动壳。
            let launcher_path = normalized_seed.with_extension("mjs");
            if !launcher_path.exists() {
                return Err(miette!(
                    "{stage_name} WASM seed 缺少启动壳: {launcher_path}\n\
                     请确认 seed 编译时已生成 .mjs 启动壳",
                    stage_name = stage_name,
                    launcher_path = launcher_path.display()
                ));
            }
            let mut cmd = Command::new("node");
            cmd.arg(&launcher_path).arg(source_file).arg(&output_file);
            cmd
        };

        println!("{}: executing {:?}", stage_name, cmd);

        let output = cmd.output().into_diagnostic().map_err(|error| error.wrap_err(format!("{stage_name} WASM 运行失败")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(miette!(
                r#"{} WASM 运行失败
stdout: {}
stderr: {}"#,
                stage_name,
                stdout.trim(),
                stderr.trim()
            ));
        }

        // 验证产物确实被生成了。
        if !output_file.exists() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(miette!(
                r#"{stage_name} WASM 运行退出码为 0 但未产出产物 {path}
seed（{seed}）可能未正确实现输出逻辑。
stdout: {stdout}
stderr: {stderr}"#,
                stage_name = stage_name,
                path = output_file.display(),
                seed = seed.display(),
                stdout = stdout.trim(),
                stderr = stderr.trim(),
            ));
        }

        println!("  {} 输出: {} ({} 字节)", stage_name, output_file.display(), fs::metadata(&output_file).map(|m| m.len()).unwrap_or(0));

        return Ok(());
    }

    // 原生 seed：直接运行 build 命令。
    let artifact_name = get_artifact_name(project_dir);
    let expected_artifact = output_dir.join(artifact_filename(&artifact_name, target));

    // 将 stderr 重定向到文件，避免 `Command::output()` 在 Windows 上
    // 因子进程写入 stderr 管道而 panic（Windows pipe 读取的已知问题）。
    let stderr_path = output_dir.join(format!("{stage_name}_stderr.log"));
    let stderr_file = std::fs::File::create(&stderr_path)
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("创建 stderr 日志文件失败 {}", stderr_path.display())))?;

    let mut cmd = Command::new(&normalized_seed);
    cmd.arg("build").arg(&normalized_project_dir).arg("--target").arg(target).arg("--output").arg(&normalized_output_dir);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::from(stderr_file));

    println!("{}: executing {:?}", stage_name, cmd);

    let output = cmd.output().into_diagnostic().map_err(|error| error.wrap_err(format!("{stage_name} 编译失败")))?;

    if !output.status.success() {
        let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(miette!(
            r#"{} 编译失败
stdout: {}
stderr: {}"#,
            stage_name,
            stdout.trim(),
            stderr.trim()
        ));
    }

    // 验证产物确实被生成了。
    if !expected_artifact.exists() {
        let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(miette!(
            r#"{stage_name} 编译退出码为 0 但未产出产物 {path}
seed（{seed}）可能不具备 `legion build` 命令行能力。
stdout: {stdout}
stderr: {stderr}"#,
            stage_name = stage_name,
            path = expected_artifact.display(),
            seed = seed.display(),
            stdout = stdout.trim(),
            stderr = stderr.trim(),
        ));
    }

    Ok(())
}

/// 运行并验证产物。
///
/// 对于 `CLR` 目标，直接执行 `.exe` 并检查退出码。
/// 对于 `WASM` 目标，通过 `node <artifact>.mjs <source_file> <output>` 运行，
///   微编译器读取源文件内容，产生输出文件用于后续比对。
/// 对于 `WASI` 目标，通过本机 `wasmtime <artifact>.wasm` 运行。
fn run_and_verify(artifact_path: &Path, target: &str, _seed: &Path, output_path: Option<&Path>, source_file: &Path) -> Result<()> {
    if !artifact_path.exists() {
        return Err(miette!("产物文件不存在: {}", artifact_path.display()));
    }

    let normalized_artifact = strip_verbatim_prefix(artifact_path);

    // `WASI` 目标字符串形如 `wasm32-unknown-wasi-wasi`，用 `contains("wasi")` 检测。
    // 注意：必须先检查 `wasi` 再检查 `wasm`，因为 `WASI` 字符串也包含 `wasm`。
    if target.contains("wasi") {
        // WASI 目标：直接通过本机 wasmtime 运行 wasm 模块。
        let wasmtime = find_wasmtime_cli()?;

        if !source_file.exists() {
            return Err(miette!("源文件不存在: {}", source_file.display()));
        }

        let mut cmd = Command::new(&wasmtime);
        cmd.arg(&normalized_artifact);

        println!("verify: executing {:?}", cmd);
        let output = cmd.output().into_diagnostic().map_err(|error| error.wrap_err("执行本机 wasmtime 失败"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(miette!(
                r#"WASI 产物运行失败
stdout: {}
stderr: {}"#,
                stdout.trim(),
                stderr.trim()
            ));
        }

        return Ok(());
    }

    if target.contains("wasm") {
        // WASM 目标（非 WASI）：通过 node 运行 .mjs 启动壳。
        let launcher_path = normalized_artifact.with_extension("mjs");
        if !launcher_path.exists() {
            return Err(miette!("WASM 产物缺少启动壳: {}", launcher_path.display()));
        }

        if !source_file.exists() {
            return Err(miette!("源文件不存在: {}", source_file.display()));
        }

        let mut cmd = Command::new("node");
        // 诚实自举：传递源文件路径，而非固定输入值。
        // 微编译器通过 read_source_byte 导入读取源文件内容。
        cmd.arg(&launcher_path).arg(source_file);

        // 如果有输出路径参数，传递给启动壳用于收集输出字节。
        if let Some(output) = output_path {
            cmd.arg(output);
        }

        println!("verify: executing {:?}", cmd);
        let output = cmd.output().into_diagnostic().map_err(|error| error.wrap_err("执行 node 启动壳失败"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(miette!(
                r#"WASM 产物运行失败
stdout: {}
stderr: {}"#,
                stdout.trim(),
                stderr.trim()
            ));
        }

        // 如果有输出路径，验证输出文件已生成。
        if let Some(output) = output_path {
            if !output.exists() {
                return Err(miette!("WASM 运行成功但未产出输出文件: {}", output.display()));
            }
            println!("  WASM 输出: {} ({} 字节)", output.display(), fs::metadata(output).map(|m| m.len()).unwrap_or(0));
        }

        return Ok(());
    }

    // CLR 目标：直接执行 .exe。
    let path_str = normalized_artifact.to_str().ok_or_else(|| miette!("路径包含非 UTF-8 字符"))?;
    let status = run_cli_program(path_str, &[]).map_err(|error| error.wrap_err("执行产物失败"))?;

    if !status.success() {
        return Err(miette!("产物运行返回非零退出码: {} (code={})", status, status.code().unwrap_or(-1)));
    }

    Ok(())
}

/// 运行 CLI 程序并返回退出状态。
fn run_cli_program(program: &str, args: &[&str]) -> Result<ExitStatus> {
    let output =
        Command::new(program).args(args).output().into_diagnostic().map_err(|error| error.wrap_err(format!("无法执行 {}", program)))?;

    Ok(output.status)
}

fn candidate_command_paths(dir: &Path, command: &str, extensions: &[String]) -> Vec<PathBuf> {
    let base = dir.join(command);
    if Path::new(command).extension().is_some() {
        return vec![base];
    }

    let mut candidates = Vec::with_capacity(1 + extensions.len());
    candidates.push(base.clone());
    for ext in extensions {
        candidates.push(dir.join(format!("{command}{ext}")));
    }
    candidates
}

fn executable_extensions() -> Vec<String> {
    if cfg!(windows) {
        env::var("PATHEXT")
            .ok()
            .map(|value| value.split(';').filter(|item| !item.is_empty()).map(|item| item.to_ascii_lowercase()).collect())
            .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string(), ".com".to_string()])
    }
    else {
        Vec::new()
    }
}

/// 比对两个产物文件的字节一致性。
///
/// 诚实自举要求 v1 和 v2 是同一份源码两次编译的产物，应字节一致。
/// 若不一致，输出首个差异位置和上下文。
fn compare_artifacts(v1_path: &Path, v2_path: &Path) -> Result<()> {
    if !v1_path.exists() {
        return Err(miette!("v1 产物不存在: {}", v1_path.display()));
    }
    if !v2_path.exists() {
        return Err(miette!("v2 产物不存在: {}", v2_path.display()));
    }

    let v1_bytes = fs::read(v1_path).into_diagnostic().map_err(|error| error.wrap_err("读取 v1 失败"))?;
    let v2_bytes = fs::read(v2_path).into_diagnostic().map_err(|error| error.wrap_err("读取 v2 失败"))?;

    if v1_bytes == v2_bytes {
        println!("  v1/v2 字节一致 ({} bytes)", v1_bytes.len());
        return Ok(());
    }

    // 二进制不一致，报告差异统计。
    let min_len = v1_bytes.len().min(v2_bytes.len());
    let mut diff_pos = None;
    for i in 0..min_len {
        if v1_bytes[i] != v2_bytes[i] {
            diff_pos = Some(i);
            break;
        }
    }

    match diff_pos {
        Some(pos) => Err(miette!(
            r#"v1/v2 比对失败: 二进制不一致
首次差异位置: 0x{pos:X} (v1=0x{v1:02X}, v2=0x{v2:02X})
v1 长度: {v1_len}, v2 长度: {v2_len}"#,
            pos = pos,
            v1 = v1_bytes[pos],
            v2 = v2_bytes[pos],
            v1_len = v1_bytes.len(),
            v2_len = v2_bytes.len()
        )),
        None => Err(miette!("v1/v2 比对失败: 长度不一致 (v1: {}, v2: {})", v1_bytes.len(), v2_bytes.len())),
    }
}

/// 根据目标平台返回产物文件名。
fn artifact_filename(artifact_name: &str, target: &str) -> String {
    // `WASM` 和 `WASI` 目标都生成 `.wasm` 文件。
    if target.contains("wasm") || target.contains("wasi") {
        format!("{}.wasm", artifact_name)
    }
    else {
        format!("{}.exe", artifact_name)
    }
}

/// 从项目目录或 legion.von 中提取产物名称。
fn get_artifact_name(project_dir: &Path) -> String {
    // 尝试从 legion.von 中读取项目名称。
    let manifest_path = project_dir.join("legion.von");
    if manifest_path.exists() {
        if let Ok(content) = fs::read_to_string(&manifest_path) {
            if let Ok(value) = von_parser::from_str::<von_parser::VonValue>(&content) {
                if let Some(object) = value.as_object() {
                    if let Some(name) = object.get("name") {
                        if let Some(name_str) = name.as_str() {
                            return name_str.to_string();
                        }
                    }
                }
            }
        }
    }

    // 回退到目录名。
    project_dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "unknown".to_string())
}

/// 获取编译器源文件路径。
///
/// 诚实自举要求 v1 实际读取源文件内容，而非使用固定输入值。
/// 源文件位于 `<project_dir>/source/main.v`。
fn get_compiler_source_path(project_dir: &Path) -> PathBuf {
    project_dir.join("source").join("main.v")
}
