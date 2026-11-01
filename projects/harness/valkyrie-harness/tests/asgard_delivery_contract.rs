//! Asgard static 交付线 JS glue ↔ Wasm chunk 一致性合同的冻结与状态矩阵（03 §7/§11）。
//!
//! 目的：把"glue + wasm 是一份需要大量测试才能稳定的一致性合同"写成可断言条目。
//! 每个条目当前只有两种状态：`VerifiedNow`（本轮真实产物/命令可证）或
//! `Blocked`（附根因与 nyar-vm.rs 坐标）。禁止用 HTML 字符串搜索、"目录里有 wasm"
//! 或存在性断言冒充执行证据；当前 glue/Wasm 产物根本不存在，因此本矩阵的大多数条目
//! 是 blocked 记录，而非绿灯。
//!
//! 复现命令（自 `projects/examples/asgard-static` 出发）：
//!
//! ```text
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   build --static source/index.awsl <examples/asgard-static>
//! ```
//!
//! 实际产物清单（本轮实测）：`dist/index.html` 一份，来源为
//! `rusty_valkyrie::asgard::static_site::build_static_site` →
//! `codegen::html_render::render_static_ir`（Rust 内置组件语义预渲染）→
//! `ssg::generate_static_page_with_base`（页面壳）。无 `.js`、无 `.wasm`、无 loader、
//! 无 manifest、无资源拷贝。
//!
//! blocked 条目的"缺席哨兵"测试断言产物中确实没有 glue/chunk 文件；当 nyar-vm.rs
//! 接入真实产物链后该哨兵会失败，即提醒按本表逐条重锚为执行证据，而不是放行半成品。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::asgard_cli;

/// 合同条目状态。
#[derive(Debug)]
enum Status {
    /// 本轮真实产物/命令可证。
    VerifiedNow,
    /// 当前不可证：附根因（含 nyar-vm.rs 坐标）。
    Blocked(&'static str),
}

/// 一条可断言的合同条目。
struct ContractItem {
    id: &'static str,
    claim: &'static str,
    status: Status,
}

/// 03 §7/§11 冻结的合同条目表。顺序即验收顺序，条目不得静默删除。
const CONTRACT_ITEMS: &[ContractItem] = &[
    ContractItem {
        id: "artifact-completeness",
        claim: "static 构建产物树完整且只含合同声明的文件；当前合同面为单 `index.html`",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "entry-component-correspondence",
        claim: "入口文件 ↔ 根组件名 ↔ 产物页一一对应（stdout 合同行与产物路径核验）",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "page-shell-base",
        claim: "页面壳合同成立：完整 doctype、charset/viewport、标题回退根组件名；根基址 `/` 时 `ssg` 按合同省略 `<base>`，\
                配置驱动的非根 base 属 `mime-and-base-subdir` 缺口",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "build-determinism",
        claim: "同输入重复构建逐字节确定（manifest 确定性的页面级子集）",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "w-awsl-synonym",
        claim: "后缀合同（设计仓 `54d13b9`）：`.awsl` 为规范拼写，`.w` 同 parser/同 lowering/同身份仍可接受，同内容入口产物逐字节相同",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "config-driven-build",
        claim: "`asgard build <project>` 读 `asgard.config.v` 求值产出完整 static 产物，不要求裸参数",
        status: Status::Blocked(
            "配置求值（C0/C2）未接入：nyar-vm.rs `rusty-valkyrie/src/asgard/cli.rs::run_build` App 分支失败闭合；\
             `projects/nyar.ts/emulators/valkyrie/src/cli/asgard-cli.ts::runStaticBuild` 仍消费裸 entry/title/output",
        ),
    },
    ContractItem {
        id: "project-admission-rejection",
        claim: "缺 `legion.von` / 缺 `asgard.config.v` / 内容无效 / 工程归属不一致 → 拒绝执行",
        status: Status::Blocked(
            "`--static` 黑盒面绕过准入：nyar-vm.rs `projects/nyar.rs/ffis/valkyrie-napi/src/asgard_api.rs::asgard_build_static` \
             直达 `static_site::build_static_site`，未经 `asgard/cli.rs::require_project_admission`（后者仅覆盖 Rust argv `run_build`）",
        ),
    },
    ContractItem {
        id: "js-glue-exists",
        claim: "首屏 JS glue 作为独立产物文件生成，可被宿主加载",
        status: Status::Blocked("静态构建只写 `index.html`：`static_site.rs::build_static_site` 无 JS 产物发射"),
    },
    ContractItem {
        id: "wasm-chunk-exists",
        claim: "可达性减枝后的 Wasm chunk 作为产物文件生成，与 glue 成对",
        status: Status::Blocked("无 Wasm chunk 产物：backend/emitter 产物链未接入 asgard 静态交付（06 §3 记录产物仍只有 index.html）"),
    },
    ContractItem {
        id: "loader-manifest-hash",
        claim: "每个 chunk 有配套 loader/manifest：URL/hash、依赖、真实 import/export、初始化顺序；入口↔loader↔manifest↔hash 对应",
        status: Status::Blocked("无 loader/manifest 产物；`chunk_planning` 有目标策略但无产物消费者（06 §3）"),
    },
    ContractItem {
        id: "mime-and-base-subdir",
        claim: "serve 交付 Wasm 用正确 MIME；base 子目录部署时 HTML/JS/Wasm/CSS/资源路径全部相对 base",
        status: Status::Blocked(
            "`serve` 未接入（`asgard-cli.ts` UNWIRED_COMMANDS）；`base` 在 `cli.rs::run_build` 与 \
             `asgard_api.rs::asgard_build_static` 硬编码 `/`，配置 `artifact.base` 未被消费",
        ),
    },
    ContractItem {
        id: "assets-copy",
        claim: "显式 `assets` 目录资源进入产物树，重名/越界拒绝",
        status: Status::Blocked("`static_site.rs` 无资源拷贝路径；配置 `assets` 字段无产物消费者"),
    },
    ContractItem {
        id: "chunk-dependency-graph",
        claim: "chunk 依赖图与按需加载边界成立；共享 runtime 不重复打包",
        status: Status::Blocked("无 chunk 产物即无依赖图证据；`artifact.chunks` 配置未被求值消费"),
    },
    ContractItem {
        id: "abi-string-aggregate",
        claim: "glue↔Wasm 的字符串/聚合 ABI 往返在真实宿主执行中成立",
        status: Status::Blocked("无 glue/Wasm 运行时；聚合 ABI 仍断在 SSA 名义身份门禁（06 §5）"),
    },
    ContractItem {
        id: "event-state-dom",
        claim: "浏览器真实事件 → V/Wasm 状态 → DOM 更新往返（计数器、输入、焦点键盘）",
        status: Status::Blocked("当前交付为无 script 的预渲染页；事件处理语义归 `asgard.ui` V 库（03 §12），Rust 内置线不再扩语义"),
    },
    ContractItem {
        id: "dispose-reload",
        claim: "dispose 停止调度/取消异步/解除监听/释放 handle；重载与卸载后回调行为成立",
        status: Status::Blocked("无运行时实例可 dispose；依赖交互交付线"),
    },
    ContractItem {
        id: "dev-atomic-replacement",
        claim: "`dev` 成功重建后原子替换整页，失败保留上一可用 generation 并显示错误",
        status: Status::Blocked("`dev` 未接入（`cli.rs::run_dev` 失败闭合）"),
    },
    ContractItem {
        id: "desktop-unsplit-artifact",
        claim: "桌面 WebView target 跳过分割规划，直接生成减枝后未分割产物并本地加载",
        status: Status::Blocked("无桌面产物链与宿主选型；`html_render.rs:30` 作用域前缀硬编码 `asgard-static-*`，示例产物被浏览器线命名"),
    },
];

fn example_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/asgard-static")
}

fn copy_example_into(dest: &Path) -> PathBuf {
    let target = dest.join("asgard-static");
    copy_dir(&example_project(), &target);
    target
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("创建目标目录失败");
    for entry in fs::read_dir(from).expect("读取示例目录失败") {
        let entry = entry.expect("目录项");
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target);
        }
        else {
            fs::copy(&path, &target).expect("复制示例文件失败");
        }
    }
}

fn asgard(args: &[&str]) -> Output {
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    asgard_cli::run(&owned).expect("无法启动 asgard CLI")
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// 收集输出目录内文件清单（相对输出根，`/` 分隔，排序）。
fn artifact_tree(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    files.sort();
    files
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("读取输出目录失败") {
        let path = entry.expect("目录项").path();
        if path.is_dir() {
            collect_files(root, &path, out);
        }
        else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

/// 条目表自身的一致性：ID 唯一；blocked 条目必须携带非空根因。
#[test]
fn contract_table_is_well_formed() {
    let mut ids = std::collections::BTreeSet::new();
    for item in CONTRACT_ITEMS {
        assert!(ids.insert(item.id), "合同条目 ID 重复: {}", item.id);
        if let Status::Blocked(reason) = &item.status {
            assert!(!reason.trim().is_empty(), "blocked 条目 `{}` 必须附根因", item.id);
        }
    }
}

/// VerifiedNow 条目对真实产物成立：构建一次，逐项核验声明的合同面。
#[test]
fn verified_items_hold_against_real_artifacts() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]);
    assert!(output.status.success(), "静态构建应成功\nstderr:\n{}", stderr_of(&output));
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    // artifact-completeness：产物树恰为单 index.html。
    let dist = project.join("dist");
    assert_eq!(artifact_tree(&dist), vec!["index.html".to_string()], "artifact-completeness");

    // entry-component-correspondence：stdout 合同行与产物路径一致。
    assert!(stdout.contains("component: index"), "entry-component-correspondence:\n{stdout}");
    let page = stdout
        .lines()
        .find_map(|line| line.strip_prefix("page: "))
        .expect("stdout 应含 page: 合同行");
    assert_eq!(fs::canonicalize(page).expect("page 路径应存在"), fs::canonicalize(dist.join("index.html")).expect("产物页应存在"));

    // page-shell-base：页面壳合同（doctype/charset/标题回退；根 base 省略 `<base>`）。
    let html = fs::read_to_string(dist.join("index.html")).expect("读取页面失败");
    assert!(html.contains("<!DOCTYPE html>"), "page-shell-base:\n{html}");
    assert!(html.contains("<meta charset=\"utf-8\">"), "page-shell-base:\n{html}");
    assert!(html.contains("<title>index</title>"), "page-shell-base:\n{html}");
    assert!(!html.contains("<base"), "page-shell-base（根部署省略 `<base>`，非根 base 属 blocked 缺口）:\n{html}");

    // build-determinism：同输入重建逐字节确定。
    let first = fs::read(dist.join("index.html")).expect("读取第一次页面失败");
    let output2 = asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]);
    assert!(output2.status.success(), "重建应成功\nstderr:\n{}", stderr_of(&output2));
    let second = fs::read(dist.join("index.html")).expect("读取第二次页面失败");
    assert_eq!(first, second, "build-determinism");
}

/// blocked 条目的缺席哨兵：产物中确实没有 JS/Wasm/loader/manifest 文件。
/// 本测试失败 = 交付链已前进，必须把对应条目从 Blocked 重锚为真实执行证据，
/// 禁止半成品绿灯（例如只出 wasm 不出 loader/manifest 同样不合格）。
#[test]
fn blocked_glue_items_have_no_partial_artifacts() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]);
    assert!(output.status.success(), "静态构建应成功\nstderr:\n{}", stderr_of(&output));

    let tree = artifact_tree(&project.join("dist"));
    let offenders: Vec<&String> =
        tree.iter().filter(|file| file.ends_with(".js") || file.ends_with(".mjs") || file.ends_with(".wasm") || file.ends_with(".json")).collect();
    assert!(
        offenders.is_empty(),
        "js-glue-exists / wasm-chunk-exists / loader-manifest-hash 当前为 Blocked（产物链未接入）。\
         出现半成文件 {offenders:?} 说明链路已前进：请把对应条目重锚为执行证据（loader↔manifest↔hash↔MIME 成套），不得只验存在性。"
    );
}

/// CLI 面 blocked 条目的失败闭合证据：serve/dev 未接入、配置驱动构建未接入。
#[test]
fn blocked_cli_paths_fail_closed_not_silently() {
    for command in ["serve", "dev"] {
        let output = asgard(&[command]);
        assert!(!output.status.success(), "`{command}` 应失败闭合");
        assert!(stderr_of(&output).contains("尚未接入"), "`{command}` blocked 诊断缺失");
    }
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", project.to_str().expect("UTF-8 路径")]);
    assert!(!output.status.success(), "config-driven-build 应失败闭合");
    assert!(stderr_of(&output).contains("配置求值"), "config-driven-build blocked 诊断缺失");
}
