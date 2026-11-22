//! 正式产品链验收：真实 `.v` 工程经 `legionBuild` 背后的产品 API 编译成宿主可直接启动的
//! 原生可执行文件。
//!
//! 这里只调用 `package_manager_legion::product::build_project`——`legionBuild` N-API 与
//! TypeScript CLI 走的同一个入口，不绕过产品边界做机器码拼接。

use std::{fs, path::PathBuf, process::Command};

use package_manager_legion::{
    legion::resolve_platform_target,
    product::{BuildOptions, BuildOutcome, build_project},
};

const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
);

/// 跨函数调用 + 条件分支：入口返回值必须是 `42`，否则退出码断言会失败。
const SOURCE: &str = r#"namespace app;

micro choose(flag: bool) -> bool {
    return flag
}

[main]
micro main() -> i64 {
    if app::choose(true) {
        return 42
    }
    return 7
}
"#;

const MANIFEST_TEMPLATE: &str = r#"{
    name: "app",
    build: [
        { target: "%TARGET%" }
    ]
}
"#;

fn write_project(root: &std::path::Path, manifest_target: &str, source: &str) -> PathBuf {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "native-executable-e2e",
    members: []
}
"#,
    )
    .unwrap();
    fs::write(root.join("legion.von"), MANIFEST_TEMPLATE.replace("%TARGET%", manifest_target)).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();
    fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
    root.to_path_buf()
}

fn build_for(prefix: &str, source: &str, target: &str, manifest_target: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-native-exec-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let project_dir = write_project(&root, manifest_target, source);
    let output_dir = project_dir.join("dist").join("native");
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

/// 独立核对 PE 头事实，再直接启动成品读取退出码。
#[cfg(windows)]
fn run_pe_and_read_exit_code(executable: &std::path::Path) -> i32 {
    let bytes = fs::read(executable).expect("read PE artifact");
    assert_eq!(&bytes[..2], b"MZ", "产物必须是 PE 文件");
    let pe_offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    assert_eq!(&bytes[pe_offset..pe_offset + 4], b"PE\0\0", "缺少 PE 签名");
    let machine = u16::from_le_bytes(bytes[pe_offset + 4..pe_offset + 6].try_into().unwrap());
    assert_eq!(machine, 0x8664, "PE 架构必须是 AMD64");
    let optional = pe_offset + 24;
    let entry = u32::from_le_bytes(bytes[optional + 16..optional + 20].try_into().unwrap());
    assert_ne!(entry, 0, "PE AddressOfEntryPoint 必须指向冻结入口");

    Command::new(executable).status().expect("启动 PE 产物").code().expect("PE 进程必须有退出码")
}

#[cfg(windows)]
#[test]
fn builds_runnable_windows_pe_with_expected_exit_code() {
    let (outcome, root) = build_for("explicit", SOURCE, "x86_64-pc-windows-msvc", "x86_64-pc-windows-msvc");
    assert!(outcome.ok, "legion build 失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");

    let contracts_path = executable.parent().unwrap().join("run-contracts.txt");
    let contracts = fs::read_to_string(&contracts_path).expect("原生构建必须产出运行清单");
    assert!(contracts.contains("main"), "运行清单必须登记冻结入口: {contracts}");
    assert!(
        contracts.contains(executable.file_name().unwrap().to_str().unwrap()),
        "运行清单必须指向实际 PE 产物: {contracts}"
    );

    assert_eq!(run_pe_and_read_exit_code(&executable), 42, "退出码必须来自冻结入口的返回值");
    let _ = fs::remove_dir_all(&root);
}

#[cfg(windows)]
#[test]
fn native_placeholder_matches_explicit_windows_target() {
    let native = resolve_platform_target("native").expect("native");
    let explicit = resolve_platform_target("x86_64-pc-windows-msvc").expect("explicit windows");
    assert_eq!(native.platform, explicit.platform, "`native` 必须展开成与显式目标相同的平台描述");
    assert_eq!(native.contract, explicit.contract, "`native` 必须与显式机器目标携带同一产物合同");

    let (outcome, root) = build_for("native", SOURCE, "native", "native");
    assert!(outcome.ok, "legion build --target native 失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");
    assert_eq!(run_pe_and_read_exit_code(&executable), 42, "`native` 与显式目标必须产出等价行为");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn linux_target_emits_sysv_elf_from_the_same_source() {
    let (outcome, root) = build_for("linux", SOURCE, "x86_64-unknown-linux-gnu", "x86_64-unknown-linux-gnu");
    assert!(outcome.ok, "linux legion build 失败: {:?}", outcome.error);
    let executable = PathBuf::from(outcome.artifacts.first().expect("linux artifact"));
    let bytes = fs::read(&executable).expect("read ELF artifact");
    assert_eq!(&bytes[..4], b"\x7fELF", "产物必须是 ELF");
    assert_eq!(bytes[4], 2, "ELF 必须是 64 位");
    assert_eq!(u16::from_le_bytes(bytes[18..20].try_into().unwrap()), 62, "ELF 架构必须是 x86-64");
    let entry = u64::from_le_bytes(bytes[24..32].try_into().unwrap());
    assert_ne!(entry, 0, "ELF e_entry 必须指向冻结入口");
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn binary_without_frozen_entry_fails_at_entry_stage() {
    let (outcome, root) = build_for(
        "missing-entry",
        r#"namespace app;

micro helper(x: i64) -> i64 {
    return x
}
"#,
        "x86_64-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
    );
    assert!(!outcome.ok, "缺少冻结入口的 binary 不得成功: {:?}", outcome.artifacts);
    let error = outcome.error.unwrap_or_default();
    assert!(error.contains("入口") || error.contains("entry"), "错误必须来自入口阶段并指名入口身份，实际: {error}");
    let _ = fs::remove_dir_all(&root);
}
