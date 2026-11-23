//! 数组运行时验收：真实 `.v` 工程经 `package_manager_legion::product::build_project`
//! （`legionBuild` 背后的同一产品入口）编译成 Windows PE 与 `.node`，覆盖 x86-64
//! 堆数组的 16 字节头（`[0]=元素个数`、`[8]=分配字节数`、元素从 `+16` 起）。
//!
//! 与 `native_executable_e2e.rs` / `host_extension_e2e.rs` 同源脚手架：只调用产品
//! API，不绕过产品边界拼接机器码。

use std::{fs, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

/// 宿主边界只闭合 `i64`；数组元素也走 8 字节标量槽。
const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
);

/// 数组 operator 直接声明为 x86-64 ISA 绑定，不依赖 std / adaptor 包：
/// machine backend 把助记符冻结成 [`array_runtime`] 的分配与边界检查序列。
const ARRAY_OPERATORS: &str = r#"
[x86_64_operator("array.new")]
micro array_new(count: i64): [i64]

[x86_64_operator("array.length")]
micro array_length(values: [i64]): i64

[x86_64_operator("array.get")]
micro array_get(values: [i64], index: i64): i64

[x86_64_operator("array.set")]
micro array_set(values: [i64], index: i64, value: i64): unit
"#;

/// 入口：字面量构造 `[i64]`，读首元素与长度，退出码 = `length + first` = 6。
///
/// 无 std 的最小工程里 `+` 没有来源，这里把它显式绑定到 x86-64 `long.add`。
fn executable_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[x86_64_operator("long.add")]
micro i64_add(left: i64, right: i64): i64

imply i64 {{
    infix `+`(self, rhs: Self): Self {{
        i64_add(self, rhs)
    }}
}}

[main]
micro main() -> i64 {{
    let values: [i64] = [3, 1, 2]
    let first: i64 = array_get(values, 0)
    let count: i64 = array_length(values)
    return count + first
}}
"#
    )
}

/// 导出：原地把 `values[0]` 改成 7 并返回写入后的值，宿主可见写回。
fn library_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[export]
micro mutate_first(values: [i64]) -> i64 {{
    array_set(values, 0, 7)
    return array_get(values, 0)
}}
"#
    )
}

/// 导出：两个 `[i64]` 形参各改一个元素，用于验证实参复制与别名写回顺序。
fn alias_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[export]
micro mirror_pair(left: [i64], right: [i64]) -> i64 {{
    array_set(left, 0, 11)
    array_set(right, 1, 22)
    return array_get(left, 0)
}}

[export]
micro mutate_first(values: [i64]) -> i64 {{
    array_set(values, 0, 7)
    return array_get(values, 0)
}}

[export]
micro shift_add_before(offset: i64, values: [i64]) -> i64 {{
    array_set(values, 0, offset)
    return array_get(values, 0)
}}
"#
    )
}

const EXECUTABLE_MANIFEST: &str = r#"{
    name: "app",
    build: [
        { target: "%TARGET%" }
    ]
}
"#;

const LIBRARY_MANIFEST: &str = r#"{
    name: "app",
    artifact: "library"
}
"#;

fn write_project(root: &std::path::Path, manifest: &str, source: &str) -> PathBuf {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "array-runtime-e2e",
    members: []
}
"#,
    )
    .unwrap();
    fs::write(root.join("legion.von"), manifest).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();
    fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
    root.to_path_buf()
}

fn build_for(prefix: &str, source: &str, target: &str, manifest: &str) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-array-runtime-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let project_dir = write_project(&root, manifest, source);
    let output_dir = project_dir.join("dist").join("array");
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

#[cfg(windows)]
fn run_pe_and_read_exit_code(executable: &std::path::Path) -> i32 {
    let bytes = fs::read(executable).expect("read PE artifact");
    assert_eq!(&bytes[..2], b"MZ", "产物必须是 PE 文件");
    let pe_offset = u32::from_le_bytes(bytes[0x3c..0x40].try_into().unwrap()) as usize;
    assert_eq!(&bytes[pe_offset..pe_offset + 4], b"PE\0\0", "缺少 PE 签名");
    let machine = u16::from_le_bytes(bytes[pe_offset + 4..pe_offset + 6].try_into().unwrap());
    assert_eq!(machine, 0x8664, "PE 架构必须是 AMD64");
    Command::new(executable).status().expect("启动 PE 产物").code().expect("PE 进程必须有退出码")
}

/// 真构建 + 真启动：字面量数组首元素与长度必须按 16 字节头读出。
#[cfg(windows)]
#[test]
fn literal_array_reads_first_element_and_length() {
    let (outcome, root) = build_for("exe", &executable_source(), "x86_64-pc-windows-msvc", &EXECUTABLE_MANIFEST.replace("%TARGET%", "x86_64-pc-windows-msvc"));
    assert!(outcome.ok, "数组可执行文件构建失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");
    assert_eq!(run_pe_and_read_exit_code(&executable), 6, "退出码必须是 length(3) + first(3)");
    let _ = fs::remove_dir_all(root);
}

/// `.node` 真装载：返回值与宿主数组写回都必须是 7n / `[7n,1n,2n]`，并连续调用 1000 次。
#[cfg(windows)]
#[test]
fn node_export_mutates_first_element_in_place() {
    let (outcome, root) = build_for("node", &library_source(), "node", LIBRARY_MANIFEST);
    assert!(outcome.ok, "node 扩展构建失败: {:?}", outcome.error);
    let addon = artifact_with_extension(&outcome, "node");

    let script = r#"
const addon = require(process.env.MODULE_PATH);
const values = [3n, 1n, 2n];
const result = addon.mutateFirst(values);
if (result !== 7n) throw new Error("mutateFirst must return 7n, got " + String(result));
if (typeof result !== "bigint") throw new Error("i64 return must be bigint");
if (values.length !== 3 || values[0] !== 7n || values[1] !== 1n || values[2] !== 2n) {
    throw new Error("host array must be written back as [7n,1n,2n], got " + String(values));
}
// 元素同样走 bigint：i64 两端与 2^53+1 必须逐位保留。
const wide = [-9223372036854775808n, 9007199254740993n, 9223372036854775807n];
if (addon.mutateFirst(wide) !== 7n) throw new Error("wide array return must be 7n");
if (wide[0] !== 7n || wide[1] !== 9007199254740993n || wide[2] !== 9223372036854775807n) {
    throw new Error("wide elements must round-trip, got " + String(wide));
}
for (let i = 0; i < 1000; i += 1) {
    const xs = [3n, 1n, 2n];
    const value = addon.mutateFirst(xs);
    if (value !== 7n || xs[0] !== 7n || xs[1] !== 1n || xs[2] !== 2n) {
        throw new Error("iteration " + i + " failed: value=" + String(value) + " array=" + String(xs));
    }
}
process.stdout.write("ok " + String(result) + " " + String(values));
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(script)
        .env("MODULE_PATH", &addon)
        .output()
        .expect("需要 node 宿主装载 .node 产物");
    assert!(
        output.status.success(),
        "node 装载/调用失败: status={:?} stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("node array stdout: {}", String::from_utf8_lossy(&output.stdout).trim());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok 7 7,1,2", "node 输出必须确认返回值与写回");
    let _ = fs::remove_dir_all(root);
}

/// 数组别名与部分编组失败：每个形参独立复制，失败不写回、不遗留可用副本。
#[cfg(windows)]
#[test]
fn node_array_alias_and_failed_marshalling_contract() {
    let (outcome, root) = build_for("alias", &alias_source(), "node", LIBRARY_MANIFEST);
    assert!(outcome.ok, "node 扩展构建失败: {:?}", outcome.error);
    let addon = artifact_with_extension(&outcome, "node");

    let script = r#"
const addon = require(process.env.MODULE_PATH);
// 同一 JS 数组传两次：两个形参各得一份独立副本，写回按形参顺序进行，后者整段覆盖前者。
const shared = [1n, 2n];
const value = addon.mirrorPair(shared, shared);
if (value !== 11n) throw new Error("mirrorPair must return 11n, got " + String(value));
if (shared[0] !== 1n || shared[1] !== 22n) {
    throw new Error("independent copies must write back in parameter order, got " + String(shared));
}

// 标量在数组之前：数组帧槽仍必须落在帧内，且返回值/写回都走 bigint。
const mixed = [1n, 2n];
if (addon.shiftAddBefore(42n, mixed) !== 42n) throw new Error("scalar-before-array return must be 42n");
if (mixed[0] !== 42n || mixed[1] !== 2n) throw new Error("scalar-before-array writeback must be [42n,2n], got " + String(mixed));

// 部分编组失败：第二个元素不是 bigint，必须在调用 guest 前抛 TypeError，且不写回。
const bad = [5n, 6n, 7];
let typeError = null;
try {
    addon.mutateFirst(bad);
} catch (error) {
    typeError = error;
}
if (!(typeError instanceof TypeError)) throw new Error("non-bigint element must raise TypeError, got " + typeError);
if (bad[0] !== 5n || bad[1] !== 6n || bad[2] !== 7) throw new Error("failed marshalling must not write back: " + String(bad));

// 超出 i64 域：RangeError，同样不写回半成品。
const outOfRange = [1n, 9223372036854775808n];
let rangeError = null;
try {
    addon.mutateFirst(outOfRange);
} catch (error) {
    rangeError = error;
}
if (!(rangeError instanceof RangeError)) throw new Error("out-of-i64 element must raise RangeError, got " + rangeError);
if (outOfRange[0] !== 1n || outOfRange[1] !== 9223372036854775808n) throw new Error("range failure must not write back");

// 反复失败后再成功：镜像副本已被释放，后续调用仍必须正确。
for (let i = 0; i < 2000; i += 1) {
    try { addon.mutateFirst([1n, 2n, 3]); } catch (error) { if (!(error instanceof TypeError)) throw error; }
}
const after = [3n, 1n, 2n];
if (addon.mutateFirst(after) !== 7n || after[0] !== 7n) throw new Error("array mirror must stay reusable after failures");
process.stdout.write("ok alias [1,22] failures-clean");
"#;
    let output = Command::new("node")
        .arg("-e")
        .arg(script)
        .env("MODULE_PATH", &addon)
        .output()
        .expect("需要 node 宿主装载 .node 产物");
    assert!(
        output.status.success(),
        "node 装载/调用失败: status={:?} stdout={} stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("node alias stdout: {}", String::from_utf8_lossy(&output.stdout).trim());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "ok alias [1,22] failures-clean");
    let _ = fs::remove_dir_all(root);
}

/// 零长数组：分配只写头部，长度必须是 0。
fn zero_length_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[main]
micro main() -> i64 {{
    let values: [i64] = array_new(0)
    return array_length(values)
}}
"#
    )
}

/// 未初始化槽按零值读取，最后一位写入后必须按同一偏移读回。
fn last_element_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[x86_64_operator("long.add")]
micro i64_add(left: i64, right: i64): i64

imply i64 {{
    infix `+`(self, rhs: Self): Self {{
        i64_add(self, rhs)
    }}
}}

[main]
micro main() -> i64 {{
    let values: [i64] = array_new(3)
    let zero: i64 = array_get(values, 0)
    array_set(values, 2, 9)
    return zero + array_get(values, 2)
}}
"#
    )
}

/// 越界读取必须走显式失败出口，而不是读到相邻内存。
fn out_of_bounds_source() -> String {
    format!(
        r#"namespace app;
{ARRAY_OPERATORS}
[main]
micro main() -> i64 {{
    let values: [i64] = array_new(2)
    return array_get(values, 2)
}}
"#
    )
}

#[cfg(windows)]
#[test]
fn zero_length_array_reads_zero_length() {
    let manifest = EXECUTABLE_MANIFEST.replace("%TARGET%", "x86_64-pc-windows-msvc");
    let (outcome, root) = build_for("zero", &zero_length_source(), "x86_64-pc-windows-msvc", &manifest);
    assert!(outcome.ok, "零长数组构建失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");
    assert_eq!(run_pe_and_read_exit_code(&executable), 0, "零长数组长度必须是 0");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn last_element_round_trips_after_zero_initialization() {
    let manifest = EXECUTABLE_MANIFEST.replace("%TARGET%", "x86_64-pc-windows-msvc");
    let (outcome, root) = build_for("last", &last_element_source(), "x86_64-pc-windows-msvc", &manifest);
    assert!(outcome.ok, "末位读写构建失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");
    assert_eq!(run_pe_and_read_exit_code(&executable), 9, "未初始化槽读 0 + 末位写入 9");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn out_of_bounds_takes_the_defined_failure_exit() {
    let manifest = EXECUTABLE_MANIFEST.replace("%TARGET%", "x86_64-pc-windows-msvc");
    let (outcome, root) = build_for("oob", &out_of_bounds_source(), "x86_64-pc-windows-msvc", &manifest);
    assert!(outcome.ok, "越界用例构建失败: {:?}", outcome.error);
    let executable = artifact_with_extension(&outcome, "exe");
    assert_ne!(run_pe_and_read_exit_code(&executable), 0, "越界不得作为正常返回");
    let _ = fs::remove_dir_all(root);
}
