//! 托管 lane 布局验收：同一份 `.v` 工程分别编译成 CLR 程序集与 JVM `class`，
//! 由真实 `dotnet`/`java` 装载、构造、读写并返回退出码。
//!
//! 使用手写泛型 `Slot<T>` 隔离 `Apply<T>` 冻结布局本身（不引入 std 的无关形状）：
//! - `Slot<i64>`：`i64` 值字段 + `bool` 标记字段，构造 → 读标记 → 覆写 → 读回；
//! - `Slot<bool>`：`bool` 值字段 + `bool` 标记字段，真/假往返与覆写。
//!
//! 正向：构造、字段读、`mut self` 覆写、跨 `Apply` 实例的类型替换；
//! 负向：字段初值不得被构造函数默认值覆盖（初始 `false` 必须可区分）。

use std::{fs, path::Path, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
    "[primitive(\"std::primitive::usize\")] structure usize { }\n",
    "[primitive(\"std::primitive::bool\")] structure bool { }\n",
);

fn write_project(root: &std::path::Path, manifest: &str, workspace: &str, source: &str) {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(root.join("legions.von"), workspace).unwrap();
    fs::write(root.join("legion.von"), manifest).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();
    fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
}

/// `Apply<i64>` 与 `Apply<bool>` 的构造、字段读、覆写与初值保真。
fn layout_source() -> String {
    r#"namespace app;

class Slot<T> {
    value: T
    marker: bool
}

imply Slot<T> {
    micro new(value: T, marker: bool): Self {
        return Slot { value: value, marker: marker }
    }

    micro get(self): T {
        return self.value
    }

    micro marked(self): bool {
        return self.marker
    }

    micro replace(mut self, value: T): unit {
        self.value = value
    }
}

[main]
micro main() -> i32 {
    return __probe() as i32
}

micro __probe() -> i64 {
    # Apply<i64>：值字段与 bool 标记字段按冻结偏移读写。
    let mut number: Slot<i64> = Slot::new(41, true)
    if number.marked() {
    }
    else {
        return 1
    }

    # Apply<bool>：初始 false 必须保真，不能被构造默认值吞掉。
    let mut flag: Slot<bool> = Slot::new(false, true)
    if flag.marked() {
    }
    else {
        return 2
    }
    if flag.get() {
        return 3
    }

    flag.replace(true)
    if flag.get() {
    }
    else {
        return 4
    }

    number.replace(42)
    return number.get()
}
"#
    .to_string()
}

fn build(prefix: &str, target: &str, manifest_build: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-managed-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let manifest = format!(
        r#"{{
    name: "app",
    build: [ {manifest_build} ]
}}
"#
    );
    write_project(&root, &manifest, r#"{ name: "managed-layout-e2e", members: [] }"#, &layout_source());
    let output_dir = root.join("dist").join(prefix);
    let outcome = build_project(&BuildOptions {
        project_dir: root.clone(),
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
        .unwrap_or_else(|| panic!("没有产出 .{extension}：{:?}", outcome.artifacts))
}

/// CLR 入口镜像既可能是 `.dll`（库/框架依赖）也可能是 `.exe`（应用交付），
/// 两者都由 `dotnet exec` 装载；按产物事实选择，不猜文件名。
fn clr_assembly(outcome: &BuildOutcome) -> PathBuf {
    outcome
        .artifacts
        .iter()
        .map(PathBuf::from)
        .find(|path| path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "dll" | "exe")))
        .unwrap_or_else(|| panic!("没有产出 CLR 程序集：{:?}", outcome.artifacts))
}

fn exit_code(command: &mut Command) -> i32 {
    command.status().expect("启动托管运行时").code().unwrap_or(-1)
}

fn stdout_of(command: &mut Command) -> String {
    let output = command.output().expect("启动托管运行时");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(windows)]
#[test]
fn clr_apply_layouts_load_construct_read_write_and_miss() {
    let (outcome, root) = build("clr", "clr", r#"{ target: "clr", msil: true }"#);
    assert!(outcome.ok, "CLR 工程构建失败：{:?}", outcome.error);
    let assembly = clr_assembly(&outcome);
    // CLR entry 合同只接受 `void`/`int32`：退出码就是冻结入口的返回值。
    let code = exit_code(Command::new("dotnet").arg("exec").arg(&assembly));
    assert_eq!(code, 42, "CLR 冻结布局必须支撑构造/读/覆写/初值保真");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn jvm_apply_layouts_load_construct_read_write_and_miss() {
    let (outcome, root) = build("jvm", "jvm-openjdk-unknown-managed", r#"{ target: "jvm-openjdk-unknown-managed" }"#);
    assert!(outcome.ok, "JVM 工程构建失败：{:?}", outcome.error);
    let jar = artifact_with_extension(&outcome, "jar");
    // JVM launcher 按 managed ABI 把返回值写到 stdout（`main` 本身返回 void）。
    let stdout = stdout_of(Command::new("java").arg("-jar").arg(&jar));
    assert!(stdout.contains("42"), "JVM 冻结布局必须支撑构造/读/覆写/初值保真，stdout={stdout:?}");
    let _ = fs::remove_dir_all(root);
}
