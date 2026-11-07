//! `projects/examples/asgard-static` 的岛交互纵向证据：真实 `asgard build --static`
//! 构建 → Node 宿主以最小 DOM 实现加载真实 `boot.js` + wasm chunk + 组件胶水 →
//! DOM 结构断言 → 真实 `click`/`input` 事件 → V/Wasm 信号状态 → DOM 更新断言 →
//! 卸载重建断言。全程在本进程内执行真实产物字节，不启动浏览器，不用 HTML 字符串
//! 搜索冒充交互证据。
//!
//! DOM shim 只实现 boot 运行时实际消费的 API 面（元素/属性/文本/监听/选择器），
//! 不模拟布局与样式；事件派发同步执行监听，与运行时的同步信号刷新路径对齐。

use std::{path::Path, process::Command};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

const ENTRY: &str = "source/index.awsl";

fn build_example(dest: &Path) -> std::path::PathBuf {
    let project = materialize_example_workspace("asgard-static", dest);
    let dist = project.join("dist");
    // 输出目录显式指定，不依赖 CLI 默认的 `.cache/asgard`。
    let args = ["build", "--static", ENTRY, project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")];
    let owned: Vec<&std::ffi::OsStr> = args.iter().map(std::ffi::OsStr::new).collect();
    let output = asgard_cli::run(&owned).expect("无法启动 asgard CLI");
    assert!(
        output.status.success(),
        "静态构建应成功\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    project
}

/// 在 Node 宿主内执行真实 boot 链：fetch 读盘、DOM shim、wasm 实例化、岛挂载、
/// 事件派发；以 `key=value` 行回报结构与行为证据。任何缺失引用/加载失败/超时
/// 都以非零退出使测试失败，不产生静默通过。
fn run_interaction_probe(dist: &Path) -> Vec<(String, String)> {
    const SCRIPT: &str = r##"
const fs = require('fs');
const path = require('path');
const vm = require('vm');

const root = process.argv[1];
const emit = (key, value) => console.log(key + '=' + value);

// ── 最小 DOM：boot 运行时实际消费的 API 面 ──────────────────────────
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
  static text(data) {
    const node = new VNode(undefined);
    node.nodeType = 3;
    node.text = String(data);
    return node;
  }
  static comment(data) {
    const node = new VNode(undefined);
    node.nodeType = 8;
    node.text = String(data);
    return node;
  }
  get nodeType() { return this._type; }
  set nodeType(value) { this._type = value; }
  get text() { return this._text || ''; }
  set text(value) { this._text = String(value); }
  get firstChild() { return this.childNodes[0] || null; }
  get children() { return this.childNodes.filter((n) => n.nodeType === 1); }
  get classList() {
    const self = this;
    const tokens = () => (self.getAttribute('class') || '').split(/\s+/).filter(Boolean);
    return {
      add(token) { self.setAttribute('class', [...new Set([...tokens(), token])].join(' ')); },
      remove(token) { self.setAttribute('class', tokens().filter((t) => t !== token).join(' ')); },
      toggle(token) { tokens().includes(token) ? this.remove(token) : this.add(token); },
    };
  }
  get textContent() {
    if (this.nodeType !== 1) return this.text;
    return this.childNodes.map((child) => child.textContent).join('');
  }
  set textContent(value) {
    // 与 DOM 一致：文本/注释节点写 `data`，元素节点替换全部子节点。
    if (this.nodeType !== 1) {
      this.text = String(value);
      return;
    }
    this.childNodes = value === '' ? [] : [VNode.text(value)];
    for (const child of this.childNodes) child.parentNode = this;
  }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.has(name) ? this.attributes.get(name) : null; }
  removeAttribute(name) { this.attributes.delete(name); }
  appendChild(child) { child.parentNode = this; this.childNodes.push(child); return child; }
  removeChild(child) {
    const index = this.childNodes.indexOf(child);
    if (index >= 0) this.childNodes.splice(index, 1);
    child.parentNode = null;
    return child;
  }
  insertBefore(child, before) {
    child.parentNode = this;
    const index = before ? this.childNodes.indexOf(before) : -1;
    if (index >= 0) this.childNodes.splice(index, 0, child);
    else this.childNodes.push(child);
    return child;
  }
  replaceChild(child, old) {
    const index = this.childNodes.indexOf(old);
    if (index >= 0) {
      this.childNodes.splice(index, 1, child);
      child.parentNode = this;
      old.parentNode = null;
    }
    return old;
  }
  contains(node) {
    for (let current = node; current; current = current.parentNode) {
      if (current === this) return true;
    }
    return false;
  }
  addEventListener(name, fn) {
    const list = this.listeners.get(name) || [];
    list.push(fn);
    this.listeners.set(name, list);
  }
  removeEventListener(name, fn) {
    const list = (this.listeners.get(name) || []).filter((item) => item !== fn);
    this.listeners.set(name, list);
  }
  dispatchEvent(event) {
    const list = this.listeners.get(event.type) || [];
    for (const fn of [...list]) fn.call(this, event);
    return true;
  }
  matches(selector) {
    const attr = selector.match(/^\[([a-zA-Z-]+)(?:="([^"]*)")?\]$/);
    if (attr) {
      const value = this.getAttribute(attr[1]);
      if (value === null) return false;
      return attr[2] === undefined || value === attr[2];
    }
    if (selector.startsWith('.')) return (this.getAttribute('class') || '').split(/\s+/).includes(selector.slice(1));
    if (selector === '*') return true;
    return this.tagName === selector.toUpperCase();
  }
  querySelectorAll(selector) {
    const found = [];
    const visit = (node) => {
      for (const child of node.childNodes) {
        if (child.nodeType === 1) {
          if (child.matches(selector)) found.push(child);
          visit(child);
        }
      }
    };
    visit(this);
    return found;
  }
}

const documentHandle = new VNode('#document');
documentHandle.createElement = (tag) => new VNode(tag);
documentHandle.createTextNode = (text) => VNode.text(text);
documentHandle.createComment = (text) => VNode.comment(text);

const loadedScripts = [];
documentHandle.head = new VNode('head');
const realHeadAppend = documentHandle.head.appendChild.bind(documentHandle.head);
documentHandle.head.appendChild = (script) => {
  loadedScripts.push(String(script.src));
  const file = path.join(root, String(script.src));
  if (!fs.existsSync(file)) throw new Error('script 404: ' + script.src);
  vm.runInThisContext(fs.readFileSync(file, 'utf8'), { filename: file });
  if (typeof script.onload === 'function') script.onload();
  return realHeadAppend(script);
};

// 页面初始 DOM：岛挂载点（含预渲染占位，hydration 应清空后由 wasm 重建）。
const islandHost = new VNode('div');
islandHost.setAttribute('data-island', 'hydrated');
islandHost.setAttribute('data-component', 'index');
islandHost.setAttribute('data-module', 'asgard-static');
islandHost.appendChild(VNode.text('prerender-placeholder'));
documentHandle.appendChild(islandHost);

globalThis.document = documentHandle;
globalThis.CustomEvent = class CustomEvent {
  constructor(type, init) { this.type = type; this.detail = init && init.detail; }
};
const requests = [];
globalThis.fetch = function (url) {
  const text = String(url);
  if (text.startsWith('/')) return Promise.reject(new Error('absolute request: ' + text));
  requests.push(text);
  const file = path.join(root, text);
  if (!fs.existsSync(file)) return Promise.reject(new Error('fetch 404: ' + url));
  const bytes = fs.readFileSync(file);
  return Promise.resolve({
    ok: true,
    status: 200,
    arrayBuffer: async () => bytes,
    json: async () => JSON.parse(bytes.toString('utf8')),
    text: async () => bytes.toString('utf8'),
  });
};

const bootFile = path.join(root, 'boot.js');
vm.runInThisContext(fs.readFileSync(bootFile, 'utf8'), { filename: bootFile });

const find = (pred) => {
  const found = [];
  const visit = (node) => {
    for (const child of node.childNodes) {
      if (child.nodeType === 1) {
        if (pred(child)) found.push(child);
        visit(child);
      }
    }
  };
  visit(documentHandle);
  return found;
};

let watchdog;
const poll = setInterval(() => {
  const ready = globalThis.__voa && globalThis.__voa.isLoaded() && islandHost.__voaMounted;
  if (!ready) return;
  clearInterval(poll);
  try {
    // 结构证据：hydration 清空占位并由 wasm 重建组件 DOM。
    emit('placeholder.cleared', islandHost.textContent.includes('prerender-placeholder') ? 'false' : 'true');
    emit('mount.children', String(islandHost.childNodes.length));
    const hero = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('hero'));
    emit('structure.hero', String(hero.length));
    const title = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('hero-title'));
    emit('structure.title.text', title.length ? title[0].textContent : '<missing>');
    const label = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('state-label'));
    emit('structure.label.text', label.length ? label[0].textContent : '<missing>');
    const input = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('label-input'));
    emit('structure.input.value', input.length ? String(input[0].value) : '<missing>');
    const buttons = find((node) => node.tagName === 'BUTTON');
    emit('structure.buttons', String(buttons.length));

    const byText = (text) => buttons.find((button) => button.textContent === text);
    // 事件 → V/Wasm 状态 → DOM：点击 Turn on。
    byText('Turn on').dispatchEvent({ type: 'click' });
    emit('afterOn.label.text', label[0].textContent);
    emit('afterOn.input.value', String(input[0].value));
    // 点击 Turn off 回到初值。
    byText('Turn off').dispatchEvent({ type: 'click' });
    emit('afterOff.label.text', label[0].textContent);
    // 文本输入经 input 事件写回信号。
    input[0].value = 'hello';
    input[0].dispatchEvent({ type: 'input' });
    emit('afterInput.label.text', label[0].textContent);
    // 卸载重建：点击 Rebuild island 后回到初始状态且仍只挂载一份组件 DOM。
    byText('Rebuild island').dispatchEvent({ type: 'click' });
    const heroAfter = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('hero'));
    emit('afterRebuild.hero', String(heroAfter.length));
    const labelAfter = find((node) => (node.getAttribute('class') || '').split(/\s+/).includes('state-label'));
    emit('afterRebuild.label.text', labelAfter.length ? labelAfter[0].textContent : '<missing>');
    emit('loader.requests', requests.join(','));
    emit('loaded', 'true');
    clearTimeout(watchdog);
  } catch (error) {
    console.error('interaction probe failed:', error && error.stack ? error.stack : error);
    process.exitCode = 1;
  }
}, 20);

watchdog = setTimeout(() => {
  console.error('interaction probe timeout; isLoaded=' + (globalThis.__voa && globalThis.__voa.isLoaded()));
  process.exit(1);
}, 15000);
"##;
    let output =
        Command::new("node").arg("-e").arg(SCRIPT).arg(dist.to_str().expect("UTF-8 路径")).output().expect("Node 必须可用（岛交互探针宿主）");
    assert!(
        output.status.success(),
        "岛交互探针失败:\nstdout={}\nstderr={}",
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
    pairs.iter().find_map(|(k, v)| (k == key).then_some(v.as_str())).unwrap_or_else(|| panic!("交互探针缺少 `{key}` 行: {pairs:?}"))
}

#[test]
fn island_event_roundtrip_updates_dom_state() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = build_example(temp.path());
    let dist = project.join("dist");
    let pairs = run_interaction_probe(&dist);

    // 挂载与结构：占位内容被清空，wasm 重建唯一组件 DOM 树。
    assert_eq!(kv(&pairs, "placeholder.cleared"), "true", "hydration 必须清空预渲染占位");
    assert_eq!(kv(&pairs, "mount.children"), "1", "挂载点下应只有一份组件 DOM");
    assert_eq!(kv(&pairs, "structure.hero"), "1", "应重建 `.hero` 根元素");
    assert_eq!(kv(&pairs, "structure.title.text"), "你好，静态世界");
    assert_eq!(kv(&pairs, "structure.label.text"), "Label: off");
    assert_eq!(kv(&pairs, "structure.input.value"), "off");
    assert_eq!(kv(&pairs, "structure.buttons"), "3");

    // 事件 → V/Wasm 状态 → DOM 更新。
    assert_eq!(kv(&pairs, "afterOn.label.text"), "Label: on", "Turn on 应把状态翻转为 on");
    assert_eq!(kv(&pairs, "afterOn.input.value"), "on", "输入框绑定值应同步");
    assert_eq!(kv(&pairs, "afterOff.label.text"), "Label: off", "Turn off 应回到 off");
    assert_eq!(kv(&pairs, "afterInput.label.text"), "Label: hello", "输入事件应写回信号");

    // 卸载重建：重新水合后状态回到初值，DOM 不双份并存。
    assert_eq!(kv(&pairs, "afterRebuild.hero"), "1", "重建后仍应只有一份组件 DOM");
    assert_eq!(kv(&pairs, "afterRebuild.label.text"), "Label: off", "重建后状态应回到初值");
    assert_eq!(kv(&pairs, "loaded"), "true");
}
