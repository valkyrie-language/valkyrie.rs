//! 文本/控制台纵切验收：真实 `.v` 工程经 `package_manager_legion::product::build_project`
//! 编译成 Windows PE 并真实启动。
//!
//! 文本链覆盖：字面量、`++` 动态构造、跨函数传递与返回、码点长度与字节长度分离
//! （多字节、非 BMP）、内嵌 NUL 的 `from_bytes` 严格入口、`index_of`/`slice`。
//! 断言全部通过才返回 0；每个失败分支有独立诊断码。
//!
//! 控制台链单独验收：`std.io.print_line` 写出到重定向管道，宿主侧按字节捕获，
//! 校验 UTF-8 字节长度与内容（多字节 + 非 BMP）。

use std::{fs, path::Path, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

fn vendored_projects() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/valkyrie.v/projects")
}

fn workspace_manifest(members: &[String]) -> String {
    let entries = members.iter().map(|member| format!("\"{}\"", member.replace('\\', "/"))).collect::<Vec<_>>().join(",\n        ");
    format!(
        r#"{{
    name: "text-runtime-e2e",
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

/// 文本全能力纵切。失败码按能力分片，返回 0 表示全部成立。
fn text_source() -> String {
    r#"namespace app;
using std.text;
using std.collection;
using std.types;

[main]
micro main() -> i64 {
    let greeting: utf8 = "hello"
    if greeting != "hello" {
        return 1
    }

    let combined: utf8 = "foo" ++ "bar"
    if combined != "foobar" {
        return 2
    }
    if combined.byte_length() != 6 {
        return 3
    }

    # 多字节：码点个数与字节长度是两个口径。
    let cjk: utf8 = "中文"
    if cjk.length() != 2 {
        return 4
    }
    if cjk.byte_length() != 6 {
        return 5
    }

    # 非 BMP：单个标量 4 字节，码位完整保留。
    let smile: utf8 = "😀"
    if smile.length() != 1 {
        return 6
    }
    if smile.byte_length() != 4 {
        return 7
    }
    let first: char = smile.character_at(0)
    let code: u32 = first as u32
    if code != 128512 {
        return 8
    }

    # 内嵌 NUL：严格入口 `from_bytes`，NUL 是普通字节，不截断。
    let mut raw: ArrayList<u8> = ArrayList::new(3)
    raw.push(97)
    raw.push(0)
    raw.push(98)
    let decoded: Result<Utf8Text, Utf8DecodeError> = Utf8Text::from_bytes(raw.to_array())
    match decoded {
        case Fine(value):
            if value.byte_length() != 3 {
                return 9
            }
            if value.length() != 3 {
                return 10
            }
            if value.byte_at(1) != 0 {
                return 11
            }
        case Fail(error):
            return 12
    }

    # 跨函数传递与返回。
    let wrapped: utf8 = __wrap("x")
    if wrapped != "[x]" {
        return 13
    }

    # 循环动态构造。
    let mut acc: utf8 = ""
    let mut index: usize = 0
    while index < 5 {
        acc = acc ++ "ab"
        index = index + 1
    }
    if acc.byte_length() != 10 {
        return 14
    }
    if acc != "ababababab" {
        return 15
    }

    # 检索与切片（码点口径）。
    let haystack: utf8 = "hello world"
    let needle: Option<usize> = haystack.index_of("world")
    if needle.is_none() {
        return 16
    }
    if needle.unwrap() != 6 {
        return 17
    }
    let tail: utf8 = haystack.slice(6, 5)
    if tail != "world" {
        return 18
    }

    return 0
}

private micro __wrap(inner: utf8) -> utf8 {
    return "[" ++ inner ++ "]"
}
"#
    .to_string()
}

/// 控制台纵切：`print_line`/`error_line` 写出多字节与非 BMP 文本，宿主按字节捕获，
/// 并验证标准输出（-11）与标准错误（-12）两条句柄通道分别路由。
fn console_source() -> String {
    r#"namespace app;
using std.io;

[main]
micro main() -> i64 {
    print_line("valkyrie:中文😀")
    error("err:中文😀")
    return 7
}
"#
    .to_string()
}

fn build(prefix: &str, source: &str) -> (BuildOutcome, PathBuf) {
    // 二分取证通道：设置该环境变量时用外部 `.v` 源替换内置用例，其余构建参数不变。
    let source: std::borrow::Cow<'_, str> = match std::env::var("VALKYRIE_TEXT_SOURCE_FILE") {
        Ok(path) if !path.trim().is_empty() => std::borrow::Cow::Owned(fs::read_to_string(&path).unwrap_or_else(|error| panic!("读取 {path} 失败：{error}"))),
        _ => std::borrow::Cow::Borrowed(source),
    };
    let source = source.as_ref();
    let root = std::env::temp_dir().join(format!("valkyrie-text-{prefix}-{}", std::process::id()));
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
    let output_dir = root.join("dist").join("text");
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
fn text_literals_concatenation_and_utf8_boundaries() {
    let (outcome, root) = build("text", &text_source());
    assert!(outcome.ok, "文本工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 0, "文本字面量、动态构造、跨函数传递与 UTF-8 边界必须全部成立");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn console_write_line_captured_through_redirected_stdout() {
    let (outcome, root) = build("console", &console_source());
    assert!(outcome.ok, "控制台工程构建失败：{:?}", outcome.error);
    let output = Command::new(executable(&outcome)).output().expect("启动 PE 并捕获管道");
    assert_eq!(output.status.code().unwrap_or(-1), 7, "退出码即诊断通道");
    // 重定向路径：写出必须是 UTF-8 字节（"valkyrie:中文😀\n" = 9+1+6+4+1 = 21 字节）。
    let expected = "valkyrie:中文😀\n";
    assert_eq!(String::from_utf8_lossy(&output.stdout), expected, "控制台重定向写出必须是完整 UTF-8 字节");
    // 句柄分流：`error` 必须走标准错误（-12），不得混入标准输出。
    assert_eq!(String::from_utf8_lossy(&output.stderr), "err:中文😀", "标准错误句柄通道必须独立路由 UTF-8 字节");
    let _ = fs::remove_dir_all(root);
}
