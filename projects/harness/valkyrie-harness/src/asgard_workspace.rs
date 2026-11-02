//! Asgard 示例的临时工作区装配。
//!
//! 示例工程在 `legion.von` 声明 `asgard.render` 合同族 workspace 依赖后，SSG OVM 链路
//! （`compile_ssg_ovm_bundle`）要求 plan 解析能闭合到真实 workspace 成员：
//! 裸拷贝示例进临时目录会因「no workspace member was found」结构化失败。
//! 本模块按 nyar 侧 `tests/asgard/support` 已验证的同一形态，把示例副本与
//! 兄弟仓 `valkyrie.v` 的 `asgard.render` / `asgard.render.host.ovm` 包
//! （以绝对路径注册为成员，单一事实来源，不 vendor 复制）组装成自洽工作区。

use std::{
    fs,
    path::{Path, PathBuf},
};

/// 兄弟仓 `valkyrie.v` 根目录（与 `valkyrie.rs` 同级；与 `asgard_cli` 的兄弟仓模式一致）。
fn sibling_valkyrie_v_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../valkyrie.v")
}

/// 兄弟仓 `valkyrie.v` 内某包的绝对路径（正斜杠形态，供 workspace 成员引用）。
fn valkyrie_v_package(parts: &[&str]) -> String {
    let path = parts.iter().fold(sibling_valkyrie_v_root(), |path, part| path.join(part));
    path.to_string_lossy().replace('\\', "/")
}

/// 递归复制目录（构建只写临时目录，不落仓库）。
pub fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("创建目标目录失败");
    for entry in fs::read_dir(from).expect("读取目录失败") {
        let entry = entry.expect("目录项");
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target);
        }
        else {
            fs::copy(&path, &target).expect("复制文件失败");
        }
    }
}

/// 把 `projects/examples/<example_name>` 复制进 `dest` 并写出临时工作区 `legions.von`，
/// 返回示例工程副本根。工作区成员：示例副本 + 兄弟仓 `valkyrie.v` 的
/// `asgard.render` 与 `asgard.render.host.ovm` 包（绝对路径引用）。
pub fn materialize_example_workspace(example_name: &str, dest: &Path) -> PathBuf {
    let example_source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples").join(example_name);
    assert!(example_source.is_dir(), "示例工程缺失: {}", example_source.display());
    for package in ["asgard.render", "asgard.render.host.ovm"] {
        let source = sibling_valkyrie_v_root().join("projects/asgard._/projects").join(package);
        assert!(source.is_dir(), "asgard 合同包缺失（兄弟仓 valkyrie.v）: {}", source.display());
    }

    let example_target = dest.join(example_name);
    copy_dir(&example_source, &example_target);

    let members = [
        format!("    \"{}\"", example_target.to_string_lossy().replace('\\', "/")),
        format!("    \"{}\"", valkyrie_v_package(&["projects", "asgard._", "projects", "asgard.render"])),
        format!("    \"{}\"", valkyrie_v_package(&["projects", "asgard._", "projects", "asgard.render.host.ovm"])),
    ]
    .join(",\n");
    fs::write(dest.join("legions.von"), format!("{{\n  name: \"asgard-example-harness\",\n  members: [\n{members}\n  ]\n}}\n"))
        .expect("写入临时工作区 legions.von 失败");

    example_target
}
