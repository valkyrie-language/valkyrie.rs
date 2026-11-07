//! 准入子集内扩展示例的 Node 宿主 DOM 行为证据：每个示例先真实
//! `asgard build --static` 出完整交付链，再在 Node 宿主以最小 DOM 实现加载真实
//! `boot.js` + wasm chunk + 组件胶水，按场景断言结构与事件→状态→DOM 行为。
//! 不启动浏览器，不用产物存在性或 HTML 字符串冒充行为证据。

use std::{path::Path, process::Command};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

const ENTRY: &str = "source/index.awsl";

fn build_example(name: &str, dest: &Path) -> std::path::PathBuf {
    let project = materialize_example_workspace(name, dest);
    let dist = project.join("dist");
    // 输出目录显式指定，不依赖 CLI 默认的 `.cache/asgard`。
    let args = ["build", "--static", ENTRY, project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")];
    let owned: Vec<&std::ffi::OsStr> = args.iter().map(std::ffi::OsStr::new).collect();
    let output = asgard_cli::run(&owned).expect("无法启动 asgard CLI");
    assert!(
        output.status.success(),
        "示例 `{name}` 静态构建应成功\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    project
}

/// 产物树清单：CSS 名取入口页 stem（`index.css`），chunk 名取工程名。
fn assert_artifact_tree(name: &str, dist: &Path) {
    let expected = [
        format!("{name}.mjs"),
        format!("{name}.wasm"),
        format!("{name}.wat"),
        "boot.js".into(),
        "c/index.js".into(),
        "index.css".into(),
        "index.html".into(),
        "manifest.json".into(),
    ];
    let mut actual = Vec::new();
    collect_files(dist, dist, &mut actual);
    actual.sort();
    assert_eq!(actual, expected, "示例 `{name}` 的产物树应为完整交付链");
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in std::fs::read_dir(dir).expect("读取产物目录失败") {
        let path = entry.expect("目录项").path();
        if path.is_dir() {
            collect_files(root, &path, out);
        }
        else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

/// 通用岛宿主探针：DOM shim + fetch 读盘 + 真实 boot 链装载，随后把控制权交给
/// 示例场景脚本（`SCENARIO` 占位）。场景以 `emit(key, value)` 回报证据行。
fn run_scenario_probe(dist: &Path, scenario: &str) -> Vec<(String, String)> {
    run_scenario_probe_multi(dist, 1, scenario)
}

/// 同 `run_scenario_probe`，但按 `islands` 数量创建挂载点（多岛证据用）。
fn run_scenario_probe_multi(dist: &Path, islands: usize, scenario: &str) -> Vec<(String, String)> {
    const PROBE_PREFIX: &str = r##"
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const root = process.argv[1];
const emit = (key, value) => console.log(key + '=' + value);

class VNode {
  constructor(tag) {
    this.tagName = tag ? tag.toUpperCase() : undefined;
    this.attributes = new Map();
    this.childNodes = [];
    this.parentNode = null;
    this.listeners = new Map();
    this.value = '';
    this.nodeType = 1;
  }
  static text(data) { const n = new VNode(undefined); n.nodeType = 3; n.text = String(data); return n; }
  static comment(data) { const n = new VNode(undefined); n.nodeType = 8; n.text = String(data); return n; }
  get firstChild() { return this.childNodes[0] || null; }
  get children() { return this.childNodes.filter((n) => n.nodeType === 1); }
  get classList() {
    const self = this;
    const tokens = () => (self.getAttribute('class') || '').split(/\s+/).filter(Boolean);
    return {
      add(t) { self.setAttribute('class', [...new Set([...tokens(), t])].join(' ')); },
      remove(t) { self.setAttribute('class', tokens().filter((x) => x !== t).join(' ')); },
      toggle(t) { tokens().includes(t) ? this.remove(t) : this.add(t); },
    };
  }
  get textContent() {
    if (this.nodeType !== 1) return this.text;
    return this.childNodes.map((c) => c.textContent).join('');
  }
  set textContent(value) {
    if (this.nodeType !== 1) { this.text = String(value); return; }
    this.childNodes = value === '' ? [] : [VNode.text(value)];
    for (const c of this.childNodes) c.parentNode = this;
  }
  setAttribute(n, v) { this.attributes.set(n, String(v)); }
  getAttribute(n) { return this.attributes.has(n) ? this.attributes.get(n) : null; }
  removeAttribute(n) { this.attributes.delete(n); }
  appendChild(c) { c.parentNode = this; this.childNodes.push(c); return c; }
  removeChild(c) { const i = this.childNodes.indexOf(c); if (i >= 0) this.childNodes.splice(i, 1); c.parentNode = null; return c; }
  insertBefore(c, b) { c.parentNode = this; const i = b ? this.childNodes.indexOf(b) : -1; if (i >= 0) this.childNodes.splice(i, 0, c); else this.childNodes.push(c); return c; }
  replaceChild(c, o) { const i = this.childNodes.indexOf(o); if (i >= 0) { this.childNodes.splice(i, 1, c); c.parentNode = this; o.parentNode = null; } return o; }
  contains(node) { for (let cur = node; cur; cur = cur.parentNode) if (cur === this) return true; return false; }
  addEventListener(n, fn) { const l = this.listeners.get(n) || []; l.push(fn); this.listeners.set(n, l); }
  removeEventListener(n, fn) { this.listeners.set(n, (this.listeners.get(n) || []).filter((x) => x !== fn)); }
  dispatchEvent(e) { for (const fn of [...(this.listeners.get(e.type) || [])]) fn.call(this, e); return true; }
  matches(sel) {
    const attr = sel.match(/^\[([a-zA-Z-]+)(?:="([^"]*)")?\]$/);
    if (attr) { const v = this.getAttribute(attr[1]); if (v === null) return false; return attr[2] === undefined || v === attr[2]; }
    if (sel.startsWith('.')) return (this.getAttribute('class') || '').split(/\s+/).includes(sel.slice(1));
    if (sel === '*') return true;
    return this.tagName === sel.toUpperCase();
  }
  querySelectorAll(sel) {
    const out = [];
    const visit = (node) => { for (const c of node.childNodes) if (c.nodeType === 1) { if (c.matches(sel)) out.push(c); visit(c); } };
    visit(this);
    return out;
  }
}

const documentHandle = new VNode('#document');
documentHandle.createElement = (t) => new VNode(t);
documentHandle.createTextNode = (t) => VNode.text(t);
documentHandle.createComment = (t) => VNode.comment(t);
documentHandle.head = new VNode('head');
const realAppend = documentHandle.head.appendChild.bind(documentHandle.head);
documentHandle.head.appendChild = (script) => {
  const file = path.join(root, String(script.src));
  if (!fs.existsSync(file)) throw new Error('script 404: ' + script.src);
  vm.runInThisContext(fs.readFileSync(file, 'utf8'), { filename: file });
  if (typeof script.onload === 'function') script.onload();
  return realAppend(script);
};

const islandHosts = [];
for (let islandIndex = 0; islandIndex < ISLAND_COUNT; islandIndex++) {
  const host = new VNode('div');
  host.setAttribute('data-island', 'hydrated');
  host.setAttribute('data-component', 'index');
  host.appendChild(VNode.text('prerender-placeholder'));
  documentHandle.appendChild(host);
  islandHosts.push(host);
}

globalThis.document = documentHandle;
globalThis.CustomEvent = class { constructor(t, i) { this.type = t; this.detail = i && i.detail; } };
globalThis.fetch = function (url) {
  const text = String(url);
  if (text.startsWith('/')) return Promise.reject(new Error('absolute request: ' + text));
  const file = path.join(root, text);
  if (!fs.existsSync(file)) return Promise.reject(new Error('fetch 404: ' + url));
  const bytes = fs.readFileSync(file);
  return Promise.resolve({ ok: true, status: 200, arrayBuffer: async () => bytes, json: async () => JSON.parse(bytes.toString('utf8')), text: async () => bytes.toString('utf8') });
};

vm.runInThisContext(fs.readFileSync(path.join(root, 'boot.js'), 'utf8'), { filename: path.join(root, 'boot.js') });

const find = (pred) => {
  const out = [];
  const visit = (node) => {
    for (const c of node.childNodes) {
      if (c.nodeType === 1) {
        if (pred(c)) out.push(c);
        visit(c);
      }
    }
  };
  visit(documentHandle);
  return out;
};
const byClass = (cls) => find((n) => (n.getAttribute('class') || '').split(/\s+/).includes(cls));
const one = (cls) => {
  const found = byClass(cls);
  if (found.length !== 1) throw new Error('expect exactly one .' + cls + ', got ' + found.length);
  return found[0];
};
const byText = (text) => find((n) => n.tagName === 'BUTTON' && n.textContent === text)[0];
const click = (el) => el.dispatchEvent({ type: 'click' });
const typeText = (el, value) => { el.value = value; el.dispatchEvent({ type: 'input' }); };

let watchdog = setTimeout(() => {
  console.error('probe timeout; isLoaded=' + (globalThis.__voa && globalThis.__voa.isLoaded()));
  process.exit(1);
}, 15000);

const poll = setInterval(() => {
  const ready = globalThis.__voa && globalThis.__voa.isLoaded() && islandHosts.every((host) => host.__voaMounted);
  if (!ready) return;
  clearInterval(poll);
  try {
"##;
    const PROBE_SUFFIX: &str = r##"
    emit('probe.ready', 'true');
    clearTimeout(watchdog);
  } catch (error) {
    console.error('scenario failed:', error && error.stack ? error.stack : error);
    process.exitCode = 1;
  }
}, 20);
"##;
    let script = format!("const ISLAND_COUNT = {islands};\n{PROBE_PREFIX}{scenario}{PROBE_SUFFIX}");
    let output =
        Command::new("node").arg("-e").arg(&script).arg(dist.to_str().expect("UTF-8 路径")).output().expect("Node 必须可用（岛宿主探针）");
    assert!(
        output.status.success(),
        "场景探针失败:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_once('='))
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn kv<'a>(pairs: &'a [(String, String)], key: &str) -> &'a str {
    pairs.iter().find_map(|(k, v)| (k == key).then_some(v.as_str())).unwrap_or_else(|| panic!("探针缺少 `{key}` 行: {pairs:?}"))
}

/// 表单与输入绑定：两个 `utf8` 输入各自双向同步，Reset 把表单恢复到默认值。
#[test]
fn form_example_two_way_inputs_and_reset() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-form", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-form", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    emit('initial.name', one('name-input').value);
    emit('initial.city', one('city-input').value);
    emit('initial.echoName', one('echo-name').textContent);
    emit('initial.echoCity', one('echo-city').textContent);
    typeText(one('name-input'), 'Grace');
    emit('typed.echoName', one('echo-name').textContent);
    typeText(one('city-input'), 'Rome');
    emit('typed.echoCity', one('echo-city').textContent);
    emit('typed.echoNameStays', one('echo-name').textContent);
    click(byText('Reset'));
    emit('afterReset.name', one('name-input').value);
    emit('afterReset.city', one('city-input').value);
    emit('afterReset.echoName', one('echo-name').textContent);
    emit('afterReset.echoCity', one('echo-city').textContent);
"#,
    );
    assert_eq!(kv(&pairs, "initial.name"), "Ada");
    assert_eq!(kv(&pairs, "initial.city"), "Paris");
    assert_eq!(kv(&pairs, "initial.echoName"), "Name: Ada");
    assert_eq!(kv(&pairs, "initial.echoCity"), "City: Paris");
    assert_eq!(kv(&pairs, "typed.echoName"), "Name: Grace");
    assert_eq!(kv(&pairs, "typed.echoCity"), "City: Rome");
    assert_eq!(kv(&pairs, "typed.echoNameStays"), "Name: Grace");
    assert_eq!(kv(&pairs, "afterReset.name"), "Ada");
    assert_eq!(kv(&pairs, "afterReset.city"), "Paris");
    assert_eq!(kv(&pairs, "afterReset.echoName"), "Name: Ada");
    assert_eq!(kv(&pairs, "afterReset.echoCity"), "City: Paris");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 事件派生状态：advance 在两个字面状态间驱动状态机，shout 重新赋值派生文本。
#[test]
fn derived_example_event_driven_state() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-derived", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-derived", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    emit('initial.status', one('status-label').textContent);
    emit('initial.message', one('message-label').textContent);
    click(byText('Advance'));
    emit('afterFirst.status', one('status-label').textContent);
    click(byText('Advance'));
    emit('afterSecond.status', one('status-label').textContent);
    click(byText('Shout'));
    emit('afterShout.message', one('message-label').textContent);
    emit('afterShout.statusStays', one('status-label').textContent);
"#,
    );
    assert_eq!(kv(&pairs, "initial.status"), "Status: red");
    assert_eq!(kv(&pairs, "initial.message"), "hello");
    assert_eq!(kv(&pairs, "afterFirst.status"), "Status: green");
    assert_eq!(kv(&pairs, "afterSecond.status"), "Status: red");
    assert_eq!(kv(&pairs, "afterShout.message"), "louder");
    assert_eq!(kv(&pairs, "afterShout.statusStays"), "Status: red");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 重建/卸载路径：重建后信号复位、组件 DOM 不双份、一次点击只翻转一次
///（重复订阅会在一次点击里来回翻转两次，读数回到原位，可据此识别）。
#[test]
fn lifecycle_example_dispose_and_rebuild() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-lifecycle", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-lifecycle", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    emit('initial.mark', one('mark-line').textContent);
    click(byText('Mark'));
    emit('afterMark.mark', one('mark-line').textContent);
    const staleButton = byText('Mark');
    click(byText('Rebuild island'));
    emit('afterRebuild.mark', one('mark-line').textContent);
    emit('afterRebuild.lines', String(byClass('mark-line').length));
    click(byText('Mark'));
    emit('afterRebuild.singleFire', one('mark-line').textContent);
    click(staleButton);
    emit('afterRebuild.detachedClick', one('mark-line').textContent);
"#,
    );
    assert_eq!(kv(&pairs, "initial.mark"), "Mark: off");
    assert_eq!(kv(&pairs, "afterMark.mark"), "Mark: on");
    assert_eq!(kv(&pairs, "afterRebuild.mark"), "Mark: off", "重建应复位信号");
    assert_eq!(kv(&pairs, "afterRebuild.lines"), "1", "重建后组件 DOM 不得双份");
    assert_eq!(kv(&pairs, "afterRebuild.singleFire"), "Mark: on", "一次点击只应翻转一次（重复订阅会翻回原位）");
    assert_eq!(kv(&pairs, "afterRebuild.detachedClick"), "Mark: on", "已卸载按钮的旧监听不得再触发");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 条件渲染：`<if :condition>` + `<else>` 经 SSG 取初始分支渲染，hydration 后
/// Show/Hide 点击经 `rx_bind_if` 挂载/卸载对应分支，Refresh 只更新标题绑定。
#[test]
fn conditional_example_if_else_branches_follow_state() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-conditional", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-conditional", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    emit('initial.headline', one('headline').textContent);
    emit('initial.detailCount', String(byClass('detail').length));
    emit('initial.collapsedCount', String(byClass('collapsed').length));
    click(byText('Hide'));
    emit('afterHide.detailCount', String(byClass('detail').length));
    emit('afterHide.collapsed', byClass('collapsed').length ? byClass('collapsed')[0].textContent : '<missing>');
    click(byText('Show'));
    emit('afterShow.detail', byClass('detail').length ? byClass('detail')[0].textContent : '<missing>');
    click(byText('Refresh'));
    emit('afterRefresh.headline', one('headline').textContent);
    emit('afterRefresh.detailStays', byClass('detail').length ? byClass('detail')[0].textContent : '<missing>');
"#,
    );
    assert_eq!(kv(&pairs, "initial.headline"), "Status Board — online");
    assert_eq!(kv(&pairs, "initial.detailCount"), "1", "初始条件为真应渲染 then 分支");
    assert_eq!(kv(&pairs, "initial.collapsedCount"), "0");
    assert_eq!(kv(&pairs, "afterHide.detailCount"), "0", "Hide 应卸载 then 分支");
    assert_eq!(kv(&pairs, "afterHide.collapsed"), "details hidden", "Hide 后应挂载 else 分支");
    assert_eq!(kv(&pairs, "afterShow.detail"), "expanded body", "Show 应重新挂载 then 分支");
    assert_eq!(kv(&pairs, "afterRefresh.headline"), "Status Board — refreshed");
    assert_eq!(kv(&pairs, "afterRefresh.detailStays"), "expanded body", "Refresh 不应影响条件分支");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 多岛共存：同一组件在两个挂载点各自实例化。DOM 子树互相隔离，
/// 而信号状态在 wasm 模块内全局共享——任一岛的点击会同步更新两岛文本。
#[test]
fn multi_island_instances_share_state_but_not_dom() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static", &dist);
    let pairs = run_scenario_probe_multi(
        &dist,
        2,
        r#"
    emit('islands.mounted', String(islandHosts.filter((h) => h.__voaMounted).length));
    const heroOf = (host) => host.querySelectorAll('.hero')[0];
    const labelOf = (host) => host.querySelectorAll('.state-label')[0].textContent;
    const buttonOf = (host, text) => host.querySelectorAll('button').filter((b) => b.textContent === text)[0];
    emit('islands.domDistinct', String(heroOf(islandHosts[0]) !== heroOf(islandHosts[1])));
    emit('initial.a', labelOf(islandHosts[0]));
    emit('initial.b', labelOf(islandHosts[1]));
    click(buttonOf(islandHosts[0], 'Turn on'));
    emit('afterAOn.a', labelOf(islandHosts[0]));
    emit('afterAOn.b', labelOf(islandHosts[1]));
    click(buttonOf(islandHosts[1], 'Turn off'));
    emit('afterBOff.a', labelOf(islandHosts[0]));
    emit('afterBOff.b', labelOf(islandHosts[1]));
"#,
    );
    assert_eq!(kv(&pairs, "islands.mounted"), "2", "两个挂载点都应完成水合");
    assert_eq!(kv(&pairs, "islands.domDistinct"), "true", "两岛 DOM 子树必须是不同实例");
    assert_eq!(kv(&pairs, "initial.a"), "Label: off");
    assert_eq!(kv(&pairs, "initial.b"), "Label: off");
    assert_eq!(kv(&pairs, "afterAOn.a"), "Label: on", "A 岛点击更新 A 岛");
    assert_eq!(kv(&pairs, "afterAOn.b"), "Label: on", "信号状态模块级共享：A 岛点击同步更新 B 岛");
    assert_eq!(kv(&pairs, "afterBOff.a"), "Label: off");
    assert_eq!(kv(&pairs, "afterBOff.b"), "Label: off");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 嵌套布局与样式类复用、双交互区独立：Row/Column 嵌套结构、多 token 类名、
/// nav 开合与便签编辑互不干扰。
#[test]
fn nested_example_layout_regions_and_class_reuse() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-nested", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-nested", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    const page = one('page');
    const content = find((n) => (n.getAttribute('class') || '').split(/\s+/).includes('content'));
    const toggle = one('nav-toggle');
    const toggleTokens = (toggle.getAttribute('class') || '').split(/\s+/).filter(Boolean);
    emit('structure.pageChildren', String(page.children.length));
    emit('structure.contentRegions', String(content.length ? content[0].children.length : -1));
    emit('structure.toggleHasBase', String(toggleTokens.includes('nav-toggle')));
    emit('structure.toggleHasPrimary', String(toggleTokens.includes('primary')));
    emit('initial.nav', one('nav-state').textContent);
    emit('initial.note', one('note-echo').textContent);
    click(byText('Menu'));
    emit('afterToggle.nav', one('nav-state').textContent);
    emit('afterToggle.noteStays', one('note-echo').textContent);
    typeText(one('note-input'), 'second');
    emit('typed.note', one('note-echo').textContent);
    emit('typed.navStays', one('nav-state').textContent);
    click(byText('Clear'));
    emit('afterClear.note', one('note-echo').textContent);
    emit('afterClear.input', one('note-input').value);
    click(byText('Menu'));
    emit('afterSecond.nav', one('nav-state').textContent);
"#,
    );
    assert_eq!(kv(&pairs, "structure.pageChildren"), "2", "Column 应嵌套两个 Row");
    assert_eq!(kv(&pairs, "structure.contentRegions"), "2", "content Row 应嵌套 sidebar 与 main 两个 Column");
    assert_eq!(kv(&pairs, "structure.toggleHasBase"), "true", "样式类复用：nav-toggle 应保留原类名");
    assert_eq!(kv(&pairs, "structure.toggleHasPrimary"), "true", "样式类复用：nav-toggle 应同时携带 primary");
    assert_eq!(kv(&pairs, "initial.nav"), "Nav: open");
    assert_eq!(kv(&pairs, "initial.note"), "Note: first");
    assert_eq!(kv(&pairs, "afterToggle.nav"), "Nav: closed");
    assert_eq!(kv(&pairs, "afterToggle.noteStays"), "Note: first");
    assert_eq!(kv(&pairs, "typed.note"), "Note: second");
    assert_eq!(kv(&pairs, "typed.navStays"), "Nav: closed");
    assert_eq!(kv(&pairs, "afterClear.note"), "Note: ");
    assert_eq!(kv(&pairs, "afterClear.input"), "");
    assert_eq!(kv(&pairs, "afterSecond.nav"), "Nav: open");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// WebView 示例的真实宿主行为：`Refresh` 事件把状态文本推进到刷新后的值，
/// 标题与副标题保持预渲染内容，重复点击不会破坏已更新的状态。
#[test]
fn webview_example_refresh_updates_status_only() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-webview", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-webview", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    emit('initial.title', one('desk-title').textContent);
    emit('initial.subtitle', one('desk-subtitle').textContent);
    emit('initial.status', one('desk-status').textContent);
    click(byText('Refresh'));
    emit('afterRefresh.status', one('desk-status').textContent);
    emit('afterRefresh.title', one('desk-title').textContent);
    emit('afterRefresh.subtitle', one('desk-subtitle').textContent);
    click(byText('Refresh'));
    emit('afterSecondRefresh.status', one('desk-status').textContent);
"#,
    );
    assert_eq!(kv(&pairs, "initial.title"), "Valkyrie Desk");
    assert_eq!(kv(&pairs, "initial.subtitle"), "WebView desktop host example");
    assert_eq!(kv(&pairs, "initial.status"), "All services online");
    assert_eq!(kv(&pairs, "afterRefresh.status"), "Refreshed just now", "点击应更新状态文本");
    assert_eq!(kv(&pairs, "afterRefresh.title"), "Valkyrie Desk", "刷新不应改写标题");
    assert_eq!(kv(&pairs, "afterRefresh.subtitle"), "WebView desktop host example", "刷新不应改写副标题");
    assert_eq!(kv(&pairs, "afterSecondRefresh.status"), "Refreshed just now", "重复点击应保持已更新状态");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 派生值示例的组件结构：根布局下是“状态文本、推进按钮、消息文本、喊话按钮”四元顺序，
/// 事件驱动更新不改变组件结构。
#[test]
fn derived_example_keeps_component_structure_across_state_updates() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-derived", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-derived", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    const root = one('derived');
    emit('structure.tags', root.children.map((n) => n.tagName).join(','));
    emit('structure.texts', root.children.map((n) => n.textContent.trim()).join('|'));
    click(byText('Advance'));
    emit('afterAdvance.tags', root.children.map((n) => n.tagName).join(','));
    emit('afterAdvance.texts', root.children.map((n) => n.textContent.trim()).join('|'));
"#,
    );
    assert_eq!(kv(&pairs, "structure.tags"), "SPAN,BUTTON,SPAN,BUTTON", "根布局应保留四元组件顺序");
    assert_eq!(kv(&pairs, "structure.texts"), "Status: red|Advance|hello|Shout");
    assert_eq!(kv(&pairs, "afterAdvance.tags"), "SPAN,BUTTON,SPAN,BUTTON", "状态更新不应改变组件结构");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 表单示例的组件结构：两个输入框与两个回显文本按声明顺序排列，Reset 不增删节点。
#[test]
fn form_example_keeps_input_and_echo_structure() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-form", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-form", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    const root = one('form');
    emit('structure.tags', root.children.map((n) => n.tagName).join(','));
    emit('structure.inputs', String(root.children.filter((n) => n.tagName === 'INPUT').length));
    click(byText('Reset'));
    emit('afterReset.tags', root.children.map((n) => n.tagName).join(','));
"#,
    );
    assert_eq!(kv(&pairs, "structure.tags"), "SPAN,INPUT,INPUT,SPAN,SPAN,BUTTON", "表单应保留标题、两个输入、两个回显与重置按钮");
    assert_eq!(kv(&pairs, "structure.inputs"), "2", "表单应恰有两个输入框");
    assert_eq!(kv(&pairs, "afterReset.tags"), "SPAN,INPUT,INPUT,SPAN,SPAN,BUTTON", "重置不应增删组件节点");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}

/// 生命周期示例的组件结构：标记文本与两个按钮，重建岛前后节点顺序稳定。
#[test]
fn lifecycle_example_keeps_mark_and_button_structure() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example("asgard-static-lifecycle", temp.path());
    let dist = project.join("dist");
    assert_artifact_tree("asgard-static-lifecycle", &dist);
    let pairs = run_scenario_probe(
        &dist,
        r#"
    const root = one('lifecycle');
    emit('structure.tags', root.children.map((n) => n.tagName).join(','));
    emit('structure.hasInput', String(root.children.some((n) => n.tagName === 'INPUT')));
    click(byText('Rebuild island'));
    emit('probe.ready', 'true');
"#,
    );
    assert_eq!(kv(&pairs, "structure.tags"), "SPAN,BUTTON,BUTTON", "生命周期示例应保留标记文本和两个按钮");
    assert_eq!(kv(&pairs, "structure.hasInput"), "false", "生命周期示例不应包含输入框");
    assert_eq!(kv(&pairs, "probe.ready"), "true");
}
