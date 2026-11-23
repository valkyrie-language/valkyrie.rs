//! 浮点/文本纵切验收：真实 `.v` 工程经 `package_manager_legion::product::build_project`
//! 编译成 Windows PE 并真实启动。
//!
//! 覆盖 x86-64 冻结集中的 `f64`/`f32` 载体：SSE2 算术、比较（含 NaN 无序语义）、
//! `-0.0`、单双精度舍入与溢出（IEEE `inf`），以及混合签名下整数与浮点各自独立的
//! 寄存器序列（Win64：`rcx/rdx/r8/r9` 与 `xmm0..3`）和浮点返回（`xmm0`）。
//!
//! 断言全部通过才返回 0；每个失败分支有独立诊断码，便于定位首个失败能力。

use std::{fs, path::Path, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

fn vendored_projects() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/valkyrie.v/projects")
}

fn workspace_manifest(members: &[String]) -> String {
    let entries = members.iter().map(|member| format!("\"{}\"", member.replace('\\', "/"))).collect::<Vec<_>>().join(",\n        ");
    format!(
        r#"{{
    name: "float-runtime-e2e",
    members: [
        {entries}
    ]
}}
"#
    )
}

fn write_project(root: &std::path::Path, manifest: &str, workspace: &str, source: &str) {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(root.join("legions.von"), workspace).unwrap();
    fs::write(root.join("legion.von"), manifest).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();
}

/// `f64`/`f32` 全能力纵切。失败码按能力分片，返回 0 表示全部成立。
fn float_source() -> String {
    r#"namespace app;

[main]
micro main() -> i64 {
    let a: f64 = 1.5
    let b: f64 = 2.25

    let sum: f64 = a + b
    if sum != 3.75 {
        return 1
    }

    let diff: f64 = b - a
    if !(diff < 1.0) {
        return 2
    }

    let prod: f64 = a * b
    if !(prod > 3.0) {
        return 3
    }

    let quot: f64 = b / a
    if quot != 1.5 {
        return 4
    }

    let neg: f64 = -a
    if !(neg < 0.0) {
        return 5
    }

    # NaN：有序比较全假，`!=` 为真（IEEE 754 无序语义）。
    let nan: f64 = 0.0 / 0.0
    if nan == nan {
        return 6
    }
    if !(nan != nan) {
        return 7
    }
    if nan < 1.0 {
        return 8
    }
    if nan > 1.0 {
        return 9
    }
    if nan <= 1.0 {
        return 10
    }
    if nan >= 1.0 {
        return 11
    }

    # `-0.0` 与 `+0.0` 数值相等；取负保留符号位。
    let zero: f64 = 0.0
    let negative_zero: f64 = -zero
    if negative_zero != zero {
        return 12
    }
    let negative_signal: f64 = 1.0 / negative_zero
    if !(negative_signal < 0.0) {
        return 13
    }

    # 整数 ↔ 浮点：`as` 为向零截断与按目标精度舍入。
    let seven: i64 = 7
    let seven_real: f64 = seven as f64
    if seven_real != 7.0 {
        return 14
    }
    let truncated: i64 = 3.99 as f64 as i64
    if truncated != 3 {
        return 15
    }

    # 混合签名：浮点与整数各自独立的寄存器序列。
    let mixed: f64 = __mixed(1.5, 2, 2.5, 3)
    if mixed != 9.0 {
        return 16
    }

    # `f32`：算术、精度（0.1 + 0.2 在 f32 下恰好舍入到 0.3）与溢出到 inf。
    let a32: f32 = 1.25 as f32
    let b32: f32 = 2.5 as f32
    let sum32: f32 = a32 + b32
    if sum32 != 3.75 as f32 {
        return 17
    }
    let tenth: f32 = 0.1 as f32
    let fifth: f32 = 0.2 as f32
    let third: f32 = 0.3 as f32
    if tenth + fifth != third {
        return 18
    }
    # f64 的同一表达式不相等：双精度保留差异，说明两条宽度没有互相串用。
    if 0.1 + 0.2 == 0.3 {
        return 21
    }
    let huge: f32 = 300000000000000000000000000000000000000.0 as f32
    let overflow: f32 = huge * 10.0 as f32
    if !(overflow > huge) {
        return 19
    }
    if !(a32 < b32) {
        return 20
    }

    return 0
}

# 混合签名：`xmm0`、`rcx`、`xmm1`、`rdx` 各自独立编号。
private micro __mixed(first: f64, second: i64, third: f64, fourth: i64) -> f64 {
    let folded: f64 = first + third
    let whole: i64 = second + fourth
    return folded + whole as f64
}
"#
    .to_string()
}

fn build(prefix: &str, source: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-float-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let manifest = r#"{
    name: "app",
    dependencies: { std: { version: "workspace" } },
    build: [ { target: "x86_64-pc-windows-msvc" } ]
}
"#;
    let projects = vendored_projects();
    let workspace = workspace_manifest(&[
        projects.join("std").display().to_string(),
        projects.join("std.adaptors._").display().to_string(),
        projects.join("unicode._").display().to_string(),
    ]);
    write_project(&root, manifest, &workspace, source);
    let output_dir = root.join("dist").join("float");
    let outcome = build_project(&BuildOptions {
        project_dir: root.clone(),
        target: Some("x86_64-pc-windows-msvc".to_string()),
        output_dir: Some(output_dir),
    });
    (outcome, root)
}

fn executable(outcome: &BuildOutcome) -> PathBuf {
    outcome
        .artifacts
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("exe")))
        .unwrap_or_else(|| panic!("没有产出 .exe：{:?}", outcome.error))
}

fn exit_code(path: &std::path::Path) -> i32 {
    let status = Command::new(path).status().expect("启动 PE");
    status.code().unwrap_or(-1)
}

#[cfg(windows)]
#[test]
fn float_carriers_round_trip_through_sse2_and_abi() {
    let (outcome, root) = build("carriers", &float_source());
    assert!(outcome.ok, "浮点工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 0, "浮点载体、IEEE 语义与混合 ABI 必须全部成立");
    let _ = fs::remove_dir_all(root);
}
