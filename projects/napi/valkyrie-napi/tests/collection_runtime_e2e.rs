//! 聚合/集合纵切验收：真实 `.v` 工程经 `package_manager_legion::product::build_project`
//! 编译成 Windows PE 并真实启动。
//!
//! 第一层刻意不用 std：手写 `class` 覆盖 `StructNew` / `FieldGet` / `FieldSet` 与
//! `mut self` 同实例改写。第二层用真实 std 的 `ArrayList<i64>` 覆盖追加、扩容、
//! 覆写、清空后复用与 `Option<T>` 载荷；第三层用 `HashSet<i64>` 覆盖重复插入、
//! 碰撞、扩容与删除后再插入。

use std::{fs, path::Path, path::PathBuf, process::Command};

use package_manager_legion::product::{BuildOptions, BuildOutcome, build_project};

const PRIMITIVES: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
    "[primitive(\"std::primitive::usize\")] structure usize { }\n",
);

const ADDITION: &str = r#"
[x86_64_operator("long.add")]
micro i64_add(left: i64, right: i64): i64

imply i64 {
    infix `+`(self, rhs: Self): Self {
        i64_add(self, rhs)
    }
}
"#;

/// 手写引用聚合：构造、字段读写、`mut self` 修改同一实例。
fn class_source() -> String {
    format!(
        r#"namespace app;
{ADDITION}
class Counter {{
    value: i64
}}

imply Counter {{
    micro new(value: i64): Self {{
        return Counter {{ value: value }}
    }}

    micro bump(mut self, delta: i64): unit {{
        self.value = self.value + delta
    }}

    micro current(self): i64 {{
        return self.value
    }}
}}

[main]
micro main() -> i64 {{
    let mut counter: Counter = Counter::new(5)
    counter.bump(7)
    return counter.current()
}}
"#
    )
}

/// 真实 std：`ArrayList<i64>` 追加 → 扩容 → 覆写 → 清空后复用。
fn array_list_source() -> String {
    r#"namespace app;
using std.collection;

[main]
micro main() -> i64 {
    let mut list: ArrayList<i64> = ArrayList::new(2)
    list.push(10)
    list.push(20)
    push_grown(list)
    let third: i64 = list.at(3)
    list.clear()
    list.push(7)
    return third + list.at(1)
}

micro push_grown(mut list: ArrayList<i64>) -> unit {
    list.push(30)
}
"#
    .to_string()
}

/// 真实 std：`HashSet<i64>` 重复插入、碰撞、扩容、删除后再插入。
fn hash_set_source() -> String {
    r#"namespace app;
using std.collection;

[main]
micro main() -> i64 {
    let mut set: HashSet<i64> = HashSet::new()
    if set.insert(7) == false {
        return 1
    }

    if set.insert(7) {
        return 2
    }

    set.insert(15)
    set.insert(23)
    set.insert(31)
    if set.remove(15) == false {
        return 3
    }

    if set.insert(15) == false {
        return 4
    }

    if set.contains(15) == false {
        return 5
    }

    return 42
}
"#
    .to_string()
}

fn vendored_projects() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/valkyrie.v/projects")
}

fn workspace_manifest(members: &[String]) -> String {
    let entries = members.iter().map(|member| format!("\"{}\"", member.replace('\\', "/"))).collect::<Vec<_>>().join(",\n        ");
    format!(
        r#"{{
    name: "collection-runtime-e2e",
    members: [
        {entries}
    ]
}}
"#
    )
}

fn write_project(root: &std::path::Path, manifest: &str, workspace: &str, source: &str, with_std: bool) {
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(root.join("legions.von"), workspace).unwrap();
    fs::write(root.join("legion.von"), manifest).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();
    // std 自带 primitive 声明；本地夹具只在无 std 的场景补身份，避免同一声明两个来源。
    if !with_std {
        fs::write(source_dir.join("core_primitive.v"), PRIMITIVES).unwrap();
    }
}

fn build(prefix: &str, source: &str, with_std: bool) -> (BuildOutcome, PathBuf) {
    let root = std::env::temp_dir().join(format!("valkyrie-collection-{prefix}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let manifest = if with_std {
        r#"{
    name: "app",
    dependencies: { std: { version: "workspace" } },
    build: [ { target: "x86_64-pc-windows-msvc" } ]
}
"#
    } else {
        r#"{
    name: "app",
    build: [ { target: "x86_64-pc-windows-msvc" } ]
}
"#
    };
    let workspace = if with_std {
        let projects = vendored_projects();
        workspace_manifest(&[
            projects.join("std").display().to_string(),
            projects.join("std.adaptors._").display().to_string(),
            projects.join("unicode._").display().to_string(),
        ])
    } else {
        workspace_manifest(&[])
    };
    write_project(&root, manifest, &workspace, source, with_std);
    let output_dir = root.join("dist").join("collection");
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
fn class_fields_round_trip_through_struct_ops() {
    let (outcome, root) = build("class", &class_source(), false);
    assert!(outcome.ok, "class 工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 12, "5 + 7 必须落在同一实例");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn array_list_push_grow_clear_and_reuse() {
    let (outcome, root) = build("array-list", &array_list_source(), true);
    assert!(outcome.ok, "ArrayList 工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 37, "扩容后第三项 30 + 清空复用后首项 7");
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn hash_set_duplicate_collision_grow_remove_and_reinsert() {
    let (outcome, root) = build("hash-set", &hash_set_source(), true);
    assert!(outcome.ok, "HashSet 工程构建失败：{:?}", outcome.error);
    // 所有断言通过才返回 42；每个失败分支有独立诊断码。
    assert_eq!(exit_code(&executable(&outcome)), 42, "HashSet 行为必须与 std 语义一致");
    let _ = fs::remove_dir_all(root);
}

/// 定位用最小纵切：`ArrayList<Option<i64>>` 的 sum 元素载体。
fn option_array_source() -> String {
    r#"namespace app;
using std.collection;
using std.types;

[main]
micro main() -> i64 {
    let mut list: ArrayList<Option<i64>> = ArrayList::new(2)
    list.push(Some(11))
    list.push(None)
    list.push(Some(22))
    let second: Option<i64> = list.at(2)
    if second.is_none() == false {
        return 1
    }
    let third: Option<i64> = list.at(3)
    if third.is_some() == false {
        return 2
    }
    if third.unwrap() == 22 {
        return 0
    }
    return 3
}
"#
    .to_string()
}

#[cfg(windows)]
#[test]
fn array_list_option_elements_round_trip() {
    let (outcome, root) = build("option-list", &option_array_source(), true);
    assert!(outcome.ok, "ArrayList<Option> 工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 0, "sum 元素必须按值往返");
    let _ = fs::remove_dir_all(root);
}

/// 定位用最小纵切：单次 HashSet 插入。
fn hash_set_minimal_source() -> String {
    r#"namespace app;
using std.collection;

[main]
micro main() -> i64 {
    let mut set: HashSet<i64> = HashSet::new()
    if set.insert(7) == false {
        return 1
    }
    if set.contains(7) == false {
        return 2
    }
    if set.insert(7) {
        return 3
    }
    return 0
}
"#
    .to_string()
}

#[cfg(windows)]
#[test]
fn hash_set_single_insert_and_duplicate() {
    let (outcome, root) = build("hash-set-min", &hash_set_minimal_source(), true);
    assert!(outcome.ok, "HashSet 最小工程构建失败：{:?}", outcome.error);
    assert_eq!(exit_code(&executable(&outcome)), 0, "单次插入/重复插入必须自洽");
    let _ = fs::remove_dir_all(root);
}
