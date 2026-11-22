//! 宿主扩展验收：`artifact: "library"` 的 `.v` 工程经正式产品构建产出 `.node` / `.pyd`，
//! 再由真实 Node / CPython 装载、调用并观察错误传播。
//!
//! 与 `native_executable_e2e.rs` 同源：只调用 `package_manager_legion::product::build_project`，
//! 即 `legionBuild` N-API 与 TypeScript CLI 背后的同一入口。

use std::{fs, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

/// 数值原始类型：运算符族不在本用例面内，宿主边界只闭合 `i64` 实参与
/// `i64` / `bool` 返回，算子编码已由 native 可执行用例覆盖。
const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
);

/// 两个导出覆盖 `i64` 与 `bool` 返回：宿主按冻结 ABI 编组，不接受字符串等未闭合载体。
const LIBRARY_SOURCE: &str = r#"namespace app;

[export]
micro first(left: i64, right: i64) -> i64 {
    return left
}

[export]
micro always_true() -> bool {
    return true
}
"#;

/// 清单只声明产物形态；宿主合同与平台由 `--target node` / `--target python`
/// 经统一目标解析服务冻结，清单里的 `build` 只接受平台身份。
const LIBRARY_MANIFEST: &str = r#"{
    name: "app",
    artifact: "library"
}
"#;

fn write_library_project(root: &std::path::Path) -> PathBuf {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "host-extension-e2e",
    members: []
}
"#,
    )
    .unwrap();
    fs::write(root.join("legion.von"), LIBRARY_MANIFEST).unwrap();
    fs::write(source_dir.join("lib.v"), LIBRARY_SOURCE).unwrap();
    fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
    root.to_path_buf()
}

fn build_library(prefix: &str, target: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-host-extension-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let project_dir = write_library_project(&root);
    let output_dir = project_dir.join("dist").join("extension");
    let outcome = build_project(&BuildOptions {
        project_dir: project_dir.clone(),
        target: Some(target.to_string()),
        output_dir: Some(output_dir.clone()),
    });
    (outcome, output_dir)
}

fn artifact_with_extension(outcome: &BuildOutcome, extension: &str) -> PathBuf {
    outcome
        .artifacts
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case(extension)))
        .unwrap_or_else(|| panic!("没有产出 `.{extension}` 产物: {:?}", outcome.artifacts))
}

const NODE_SCRIPT: &str = r#"
const addon = require(process.env.MODULE_PATH);
if (addon.first(2, 3) !== 2) throw new Error("first(2,3) !== 2");
if (addon.first(-9, 3) !== -9) throw new Error("first(-9,3) !== -9");
if (addon.always_true() !== true) throw new Error("always_true() must be true");
let type_error = false;
try {
    addon.first("x", 1);
} catch (error) {
    type_error = error instanceof TypeError;
}
if (!type_error) throw new Error("non-integer argument must raise TypeError");
"#;

#[cfg(windows)]
#[test]
fn node_addon_loads_calls_and_propagates_errors() {
    let (outcome, output_dir) = build_library("node", "node");
    assert!(outcome.ok, "node 扩展构建失败: {:?}", outcome.error);
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
    let _ = fs::remove_dir_all(output_dir);
}

#[cfg(windows)]
#[test]
fn cpython_extension_loads_calls_and_propagates_errors() {
    let (outcome, output_dir) = build_library("python", "python");
    assert!(outcome.ok, "CPython 扩展构建失败: {:?}", outcome.error);
    let extension = artifact_with_extension(&outcome, "pyd");

    let script = concat!(
        "import os, sys\n",
        "sys.path.insert(0, os.environ['MODULE_DIR'])\n",
        "import app\n",
        "assert app.first(2, 3) == 2, app.first(2, 3)\n",
        "assert app.first(-9, 3) == -9\n",
        "assert app.always_true() is True\n",
        "try:\n",
        "    app.first('x', 1)\n",
        "except TypeError:\n",
        "    pass\n",
        "else:\n",
        "    raise AssertionError('non-integer argument must raise TypeError')\n",
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
    let _ = fs::remove_dir_all(output_dir);
}
