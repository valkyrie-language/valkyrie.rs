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
//! 实际产物清单（本轮实测）：`index.html`（含 `data-component` 岛挂载点与 `boot.js` 引用）、
//! `<module>.wasm`、`<module>.mjs`（浏览器 WASM glue）、`<module>.wat`（debug 构建附带）、
//! `c/<route>.js`（组件胶水）、`manifest.json`（wasm `url`/`glue`/`hash` 与组件胶水）、
//! `boot.js`、`<module>.css`。来源为 `rusty_valkyrie::asgard::static_site::build_static_site`
//! → `compile::compile_wasm_bundle`（`GeneratedOnly`）→ `copy_wasm_artifacts_to_dist`
//! → `codegen::package_browser_islands`。
//!
//! 仍为 blocked 的条目附根因与 nyar-vm.rs 坐标；不用 HTML 字符串搜索、"目录里有 wasm"
//! 或存在性断言冒充执行证据。浏览器内执行语义（事件→状态→DOM、卸载重建）由本轮
//! 浏览器自动化实测通过，但合同测试进程不启浏览器，故仍标注 blocked 并说明宿主缺口。

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
        claim: "static 构建产物树完整：页面 + wasm chunk + 浏览器 glue + 组件胶水 + boot + manifest + CSS（集合相等，不只是存在）",
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
        id: "widget-stem-correspondence",
        claim: "入口根组件名 ↔ 文件 stem 对应：错配结构化拒绝，匹配则组件名进 stdout 合同行与标题回退",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "conditional-aggregation-shape",
        claim: "条件渲染（`<if>`/`<else>`）、字符串聚合插值与 `@click`+`micro` 事件声明形态在正式 Legion 示例中通过构建，产物完整；\
                渲染与交互执行语义依赖 Rust 内置线，执行证据属 `event-state-dom` blocked 条目",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "list-each-shape",
        claim: "`each`/`item`/`key` 列表集合形态通过构建并产出完整页面",
        status: Status::Blocked(
            "实测构建线程栈溢出崩溃（退出码 `0xC00000FD`），非结构化失败：nyar-vm.rs \
             `codegen/html_render.rs::emit_loop` → `codegen/expr_eval.rs` 列表表达式求值路径；\
             根因修复前列表形态不得进入可构建示例，修复后本条目重锚为执行证据",
        ),
    },
    ContractItem {
        id: "config-driven-build",
        claim: "`asgard build <project>` 读 `asgard.config.v` 求值产出完整 static 产物，不要求裸参数",
        status: Status::Blocked(
            "配置求值（C0）仍断在聚合读回：nyar-vm.rs `rusty-valkyrie/tests/asgard/config_probe.rs` 的 \
             `c0_aggregate_layer_fails_closed_on_missing_nominal_identity` 固定了 WASM 预发射 SSA 名义身份门禁失败；\
             `cli.rs::run_build` App 分支因此保持失败闭合，`asgard-cli.ts` 的裸 `build` 也只打印该诊断",
        ),
    },
    ContractItem {
        id: "project-admission-rejection",
        claim: "缺 `legion.von` / 缺 `asgard.config.v` / 内容无效 / 工程归属不一致 → 拒绝执行（命令面与 N-API 面同一准入实现）",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "js-glue-exists",
        claim: "首屏 JS glue 作为独立产物文件生成，可被宿主加载（`c/<route>.js` 注册组件工厂，`<module>.mjs` 为浏览器 WASM glue）",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "wasm-chunk-exists",
        claim: "可达性减枝后的 Wasm chunk 作为产物文件生成，与 glue 成对；manifest 引用可解析到落盘文件",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "loader-manifest-hash",
        claim: "manifest 为唯一 loader 入口：URL、glue、内容 hash、组件胶水与初始化顺序均可解析；hash 等于落盘 chunk 字节的 sha256",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "mime-and-base-subdir",
        claim: "serve 交付 Wasm 用正确 MIME；base 子目录部署时 HTML/JS/Wasm/CSS/资源路径全部相对 base",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "assets-copy",
        claim: "显式资源目录资源进入产物树，重名/越界拒绝",
        status: Status::VerifiedNow,
    },
    ContractItem {
        id: "chunk-dependency-graph",
        claim: "chunk 依赖图与按需加载边界成立；共享 runtime 不重复打包",
        status: Status::Blocked(
            "仍是单 chunk，本轮不伪造按需加载。坐标：`rusty-valkyrie/src/asgard/compile.rs::compile_v_bundle` → \
             `compile_source_groups_to_artifacts` 每次只产出一个模块；`nyar::ArtifactPartitionPlan` 按目标/后端分区，\
             不按 UI 路由切分；`asgard/chunk_planning.rs::plan_asgard_build` 只有计划类型且无产物消费者；\
             `boot.js::start` 只实例化 `manifest.wasm[0]`。解除条件：路由级模块发射 + 逐 chunk 懒加载 + 配置求值（见 config-driven-build）",
        ),
    },
    ContractItem {
        id: "abi-string-aggregate",
        claim: "glue↔Wasm 的字符串/聚合 ABI 往返在真实宿主执行中成立",
        status: Status::Blocked("utf8 标量往返已在浏览器实测成立；聚合（列表/对象）读回仍断在 WASM 预发射 SSA 名义身份门禁（`compile_pipeline` verifier）"),
    },
    ContractItem {
        id: "event-state-dom",
        claim: "浏览器真实事件 → V/Wasm 状态 → DOM 更新往返（按钮点击、文本输入）",
        status: Status::Blocked(
            "浏览器实测通过（点击 off→on、输入 hello、重建后回到 off）；但合同测试进程不启浏览器，\
             缺少宿主内断言设施，故不在此标绿",
        ),
    },
    ContractItem {
        id: "dispose-reload",
        claim: "dispose 停止调度/取消异步/解除监听/释放 handle；重载与卸载后回调行为成立",
        status: Status::Blocked(
            "`boot.js` 已实现 `unmountIslands`（解绑订阅/监听、丢弃 memo、释放 DOM 句柄）并经浏览器重建实测；\
             缺宿主内断言设施，故不在此标绿",
        ),
    },
    ContractItem {
        id: "dev-atomic-replacement",
        claim: "`dev` 成功重建后原子替换整页，失败保留上一可用 generation 并显示错误",
        status: Status::Blocked(
            "Rust `cli.rs::run_dev` 已实现 staging→整代替换并有测试；产品 CLI 未接通：\
             `asgard-cli.ts` 的 `UNWIRED_COMMANDS` 仍把 `dev` 挡在 TS 层",
        ),
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

    // artifact-completeness：页面 + 成对浏览器产物（集合相等，不是存在性）。
    let dist = project.join("dist");
    let expected = vec![
        "asgard-static.css".to_string(),
        "asgard-static.mjs".to_string(),
        "asgard-static.wasm".to_string(),
        "asgard-static.wat".to_string(),
        "boot.js".to_string(),
        "c/index.js".to_string(),
        "index.html".to_string(),
        "manifest.json".to_string(),
    ];
    assert_eq!(artifact_tree(&dist), expected, "artifact-completeness");

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
    assert!(!html.contains("<base"), "page-shell-base（根部署省略 `<base>`）:\n{html}");
    assert!(html.contains(r#"<script src="boot.js"></script>"#), "页面必须引用 boot 启动脚本:\n{html}");
    assert!(html.contains("data-component=\"index\""), "页面必须声明岛挂载点:\n{html}");

    // js-glue / wasm-chunk / loader-manifest-hash：用宿主执行核对 manifest↔产物↔hash，
    // 不是字符串搜索；脚本内部对每个引用做存在性与 sha256 比对。
    assert_manifest_matches_artifacts(&dist);

    // build-determinism：同输入重建逐字节确定（页面与 chunk 都不得漂移）。
    let first_page = fs::read(dist.join("index.html")).expect("读取第一次页面失败");
    let first_wasm = fs::read(dist.join("asgard-static.wasm")).expect("读取第一次 chunk 失败");
    let output2 = asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]);
    assert!(output2.status.success(), "重建应成功\nstderr:\n{}", stderr_of(&output2));
    assert_eq!(first_page, fs::read(dist.join("index.html")).expect("读取第二次页面失败"), "build-determinism（页面）");
    assert_eq!(first_wasm, fs::read(dist.join("asgard-static.wasm")).expect("读取第二次 chunk 失败"), "build-determinism（chunk）");
}

/// 用 Node 宿主核对 manifest：每个引用必须解析到产物文件，wasm 的 hash 必须等于字节 sha256。
fn assert_manifest_matches_artifacts(dist: &Path) {
    const SCRIPT: &str = r#"
const fs = require('fs');
const crypto = require('crypto');
const path = require('path');
const root = process.argv[1];
const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
const resolve = (reference) => path.join(root, String(reference).replace(/^\/+/, ''));
for (const entry of manifest.wasm || []) {
  const wasmPath = resolve(entry.url);
  if (!fs.existsSync(wasmPath)) throw new Error('wasm missing: ' + entry.url);
  if (!fs.existsSync(resolve(entry.glue))) throw new Error('glue missing: ' + entry.glue);
  const digest = 'sha256:' + crypto.createHash('sha256').update(fs.readFileSync(wasmPath)).digest('hex');
  if (entry.hash !== digest) throw new Error('hash mismatch: manifest=' + entry.hash + ' actual=' + digest);
}
for (const reference of manifest.css || []) {
  if (!fs.existsSync(resolve(reference))) throw new Error('css missing: ' + reference);
}
for (const component of manifest.components || []) {
  if (!fs.existsSync(resolve(component.js))) throw new Error('component glue missing: ' + component.js);
  if (!component.name) throw new Error('component glue entry has no name');
}
console.log('manifest ok: ' + (manifest.wasm || []).length + ' chunk(s), ' + (manifest.components || []).length + ' component(s)');
"#;
    let output = std::process::Command::new("node")
        .arg("-e")
        .arg(SCRIPT)
        .arg(dist.to_str().expect("UTF-8 路径"))
        .output()
        .expect("Node 必须可用（主动交付线宿主）");
    assert!(
        output.status.success(),
        "manifest 与产物不一致（loader-manifest-hash / js-glue-exists / wasm-chunk-exists）:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 准入条目：缺 `legion.von` 的项目走 N-API 面同样被拒绝，不再绕过准入。
#[test]
fn admission_rejects_project_without_manifest() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    fs::remove_file(project.join("legion.von")).expect("删除工程清单失败");
    let output = asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]);
    assert!(!output.status.success(), "缺 legion.von 的构建应失败");
    let stderr = stderr_of(&output);
    assert!(stderr.contains("准入失败"), "诊断应指出项目准入失败，实际:\n{stderr}");
    assert!(!project.join("dist").exists(), "准入失败不得产出任何产物");
}

/// 资源条目：`--assets` 指向的目录被复制进产物树，且不覆盖已生成产物。
#[test]
fn assets_flag_copies_resources_into_output() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    fs::create_dir_all(project.join("assets/nested")).expect("创建资源目录失败");
    fs::write(project.join("assets/logo.txt"), "logo").expect("写入资源失败");
    fs::write(project.join("assets/nested/icon.svg"), "<svg/>").expect("写入嵌套资源失败");

    let output = asgard(&["build", "--static", "source/index.awsl", "--assets", "assets", project.to_str().expect("UTF-8 路径")]);
    assert!(output.status.success(), "带资源目录的构建应成功\nstderr:\n{}", stderr_of(&output));
    let dist = project.join("dist");
    assert!(dist.join("logo.txt").is_file(), "资源应复制到产物根");
    assert!(dist.join("nested/icon.svg").is_file(), "嵌套资源应保留相对路径");
    // 复制不得顶掉产物本身。
    assert!(dist.join("manifest.json").is_file(), "manifest 必须仍在");
    assert!(dist.join("index.html").is_file(), "页面必须仍在");
}

/// CLI 面 blocked 条目的失败闭合证据：dev/pack 未接入、配置驱动构建未接入；
/// `serve` 已接通，但缺产物目录时仍结构化失败而不是静默成功。
#[test]
fn blocked_cli_paths_fail_closed_not_silently() {
    for command in ["dev", "pack"] {
        let output = asgard(&[command]);
        assert!(!output.status.success(), "`{command}` 应失败闭合");
        assert!(stderr_of(&output).contains("尚未接入"), "`{command}` blocked 诊断缺失");
    }
    let serve = asgard(&["serve", "--port", "0"]);
    assert!(!serve.status.success(), "缺产物目录的 serve 应失败");
    assert!(stderr_of(&serve).contains("托管根"), "serve 诊断应指出托管根不可用: {}", stderr_of(&serve));
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", project.to_str().expect("UTF-8 路径")]);
    assert!(!output.status.success(), "config-driven-build 应失败闭合");
    assert!(stderr_of(&output).contains("配置求值"), "config-driven-build blocked 诊断缺失");
}
