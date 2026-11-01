//! 真实静态交付产物的宿主证据：真实执行 `asgard build --static` 与 `asgard serve`，
//! 在产物字节、loader 实际请求与 HTTP 响应头三个层面断言合同，不用"目录里有 wasm"或
//! HTML 字符串搜索冒充执行证据。
//!
//! 证据边界（勿以绿灯掩盖）：
//! - `chunk-dependency-graph` 仍是 blocked（见 `asgard_delivery_contract.rs` 的条目根因：
//!   `compile_v_bundle` 每次只产出一个模块、`boot.js::start` 只实例化 `manifest.wasm[0]`）。
//!   因此本文件把当前单 chunk 形态冻结为可证事实：manifest 只有 1 个 wasm 条目且没有
//!   `chunks`/`dependencies`/`init_order` 字段，loader 实际请求恰好是 manifest + 该 chunk；
//!   多 chunk 拓扑与按需边界不能由本文件声称成立。
//! - 浏览器内事件→状态→DOM 语义由 `event-state-dom` blocked 条目承载，本文件不代替它。
//!
//! 复现命令（自 `projects/examples/asgard-static` 出发，CLI 入口经
//! `valkyrie_harness::asgard_cli` 解析）：
//!
//! ```text
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   build --static source/index.awsl <examples/asgard-static>
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   serve --root <examples/asgard-static>/dist --base /app/ --port <port>
//! ```

use std::{
    ffi::OsStr,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    thread,
    time::Duration,
};

use tempfile::TempDir;
use valkyrie_harness::asgard_cli;

const ENTRY: &str = "source/index.awsl";
const MODULE_STEM: &str = "asgard-static";

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

fn expect_success(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "asgard 应以 0 退出，实际 {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}", output.status);
    stdout
}

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

/// 在 Node 宿主内执行真实 `boot.js`（最小 DOM/fetch shim），返回 loader 实际请求与装载结果。
///
/// 脚本自带 manifest 形状冻结与 sha256 复算：hash 由落盘 wasm 字节重算后与 manifest
/// 比较，不是对 manifest 自述的转述。任何未声明文件被请求都会因 404 直接失败。
fn run_loader_probe(dist: &Path) -> Vec<(String, String)> {
    const SCRIPT: &str = r##"
const fs = require('fs');
const path = require('path');
const vm = require('vm');
const crypto = require('crypto');

const root = process.argv[1];
const emit = (key, value) => console.log(key + '=' + value);

const manifest = JSON.parse(fs.readFileSync(path.join(root, 'manifest.json'), 'utf8'));
const wasmEntries = manifest.wasm || [];
const components = manifest.components || [];
emit('manifest.wasm.count', String(wasmEntries.length));
emit('manifest.components.count', String(components.length));

// 当前交付是单 chunk：chunk 依赖图字段尚不存在（blocked 条目 `chunk-dependency-graph`）。
const chunkGraphKeys = ['chunks', 'dependencies', 'init_order', 'imports', 'exports'];
const present = chunkGraphKeys.filter((key) => Object.prototype.hasOwnProperty.call(manifest, key));
emit('manifest.chunk_graph', present.length === 0 ? 'absent' : present.join(','));

const entry = wasmEntries[0] || {};
const wasmUrl = String(entry.url || '');
emit('manifest.wasm.url', wasmUrl);
emit('manifest.wasm.glue', String(entry.glue || ''));
const actualHash = 'sha256:' + crypto.createHash('sha256').update(fs.readFileSync(path.join(root, wasmUrl))).digest('hex');
emit('manifest.hash', String(entry.hash || ''));
emit('hash.matches', entry.hash === actualHash ? 'true' : 'false');
components.forEach((component, index) => emit('manifest.component.' + index + '.js', String(component.js)));

const requests = [];
const scripts = [];

function fileFor(url) {
  const text = String(url);
  // 相对产物根引用是当前交付合同；绝对 URL 会绕开部署基址，按失败证据记录。
  if (text.startsWith('/')) throw new Error('absolute request: ' + text);
  return path.join(root, text);
}

globalThis.fetch = function (url) {
  requests.push(String(url));
  const file = fileFor(url);
  if (!fs.existsSync(file)) throw new Error('fetch 404: ' + url);
  const bytes = fs.readFileSync(file);
  return Promise.resolve({
    ok: true,
    status: 200,
    arrayBuffer: async () => bytes,
    json: async () => JSON.parse(bytes.toString('utf8')),
    text: async () => bytes.toString('utf8'),
  });
};

globalThis.document = {
  createElement() { return { src: '', async: false, onload: null, onerror: null }; },
  createTextNode() { return { nodeType: 3, textContent: '' }; },
  createComment() { return { nodeType: 8, textContent: '' }; },
  head: {
    appendChild(script) {
      scripts.push(String(script.src));
      const file = fileFor(script.src);
      if (!fs.existsSync(file)) throw new Error('script 404: ' + script.src);
      vm.runInThisContext(fs.readFileSync(file, 'utf8'), { filename: file });
      if (typeof script.onload === 'function') script.onload();
    },
  },
  // 宿主没有真实岛节点：挂载扫描返回空集合，避免触达 wasm DOM 原语。
  querySelectorAll() { return { forEach() {} }; },
};

const bootFile = path.join(root, 'boot.js');
vm.runInThisContext(fs.readFileSync(bootFile, 'utf8'), { filename: bootFile });

let waited = 0;
const timer = setInterval(() => {
  waited += 20;
  const loaded = globalThis.__voa && globalThis.__voa.isLoaded();
  if (loaded && scripts.length >= components.length) {
    clearInterval(timer);
    emit('request.count', String(requests.length));
    requests.forEach((url, index) => emit('request.' + index, url));
    emit('script.count', String(scripts.length));
    scripts.forEach((url, index) => emit('script.' + index, url));
    emit('wasmModule', String(globalThis.__voa.wasmModule));
    emit('loaded', 'true');
    process.exitCode = 0;
    return;
  }
  if (waited > 5000) {
    clearInterval(timer);
    console.error('loader did not finish within 5s; requests=' + JSON.stringify(requests));
    process.exitCode = 1;
  }
}, 20);
"##;
    let output = Command::new("node")
        .arg("-e")
        .arg(SCRIPT)
        .arg(dist.to_str().expect("UTF-8 路径"))
        .output()
        .expect("Node 必须可用（主动交付线的宿主）");
    assert!(
        output.status.success(),
        "loader 探针失败:\nstdout={}\nstderr={}",
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
    pairs
        .iter()
        .find_map(|(k, v)| (k == key).then_some(v.as_str()))
        .unwrap_or_else(|| panic!("loader 探针缺少 `{key}` 行: {pairs:?}"))
}

/// 真实构建 + 真实 loader 执行：页面引用 boot、manifest 摘要与落盘字节一致、
/// loader 请求集合恰为 manifest + 唯一 chunk。
#[test]
fn loader_requests_only_manifest_declared_artifacts() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let stdout = expect_success(&asgard(&["build", "--static", ENTRY, project.to_str().expect("UTF-8 路径")]));

    assert!(stdout.contains("asgard build (static) complete"), "stdout 应有完成行:\n{stdout}");
    assert!(stdout.contains("component: index"), "stdout 应声明根组件:\n{stdout}");

    let dist = project.join("dist");
    let tree = artifact_tree(&dist);
    let expected = [
        "asgard-static.css",
        "asgard-static.mjs",
        "asgard-static.wasm",
        "asgard-static.wat",
        "boot.js",
        "c/index.js",
        "index.html",
        "manifest.json",
    ]
    .map(str::to_string);
    assert_eq!(tree, expected, "产物树必须与交付合同集合相等");

    // 页面必须真实引用 boot 启动脚本与岛挂载点（不是存在性断言）。
    let html = fs::read_to_string(dist.join("index.html")).expect("读取页面失败");
    assert!(html.contains(r#"<script src="boot.js"></script>"#), "页面必须引用 boot.js:\n{html}");
    assert!(html.contains(r#"data-component="index""#), "页面必须声明岛挂载点:\n{html}");

    let pairs = run_loader_probe(&dist);
    assert_eq!(kv(&pairs, "manifest.wasm.count"), "1", "当前是单 chunk 交付");

    assert_eq!(kv(&pairs, "manifest.components.count"), "1");
    assert_eq!(kv(&pairs, "manifest.chunk_graph"), "absent", "chunk 依赖图仍是 blocked，不得伪造字段");
    assert_eq!(kv(&pairs, "manifest.wasm.url"), "asgard-static.wasm");

    assert_eq!(kv(&pairs, "manifest.wasm.glue"), "asgard-static.mjs");
    assert!(kv(&pairs, "manifest.hash").starts_with("sha256:"), "manifest 必须声明 sha256 摘要");
    assert_eq!(kv(&pairs, "hash.matches"), "true", "manifest 摘要必须等于落盘 wasm 字节的 sha256");
    assert_eq!(kv(&pairs, "loaded"), "true");
    assert_eq!(kv(&pairs, "wasmModule"), MODULE_STEM);

    // loader 实际请求集合：manifest + manifest 声明的 chunk，别无其他。
    assert_eq!(kv(&pairs, "request.count"), "2", "loader 只应请求 manifest 与声明的 chunk");
    let requested: Vec<&str> =
        pairs.iter().filter(|(key, _)| key.starts_with("request.") && key.as_str() != "request.count").map(|(_, value)| value.as_str()).collect();
    let expected_requests = ["manifest.json", "asgard-static.wasm"];
    assert_eq!(requested, expected_requests, "未声明产物不得被加载");
    assert!(!requested.iter().any(|url| url.ends_with(".wat")), "debug 产物 .wat 不得被 loader 请求");

    // 组件胶水按 manifest 顺序加载；manifest 之外脚本不得出现。
    assert_eq!(kv(&pairs, "script.count"), "1");
    assert_eq!(kv(&pairs, "script.0"), "c/index.js");

}

/// 子目录部署与资源目录：`<base href>` 进入页面壳，serve 按 base 前缀交付真实产物与正确 MIME。
#[test]
fn base_prefix_and_assets_are_served_with_real_mime() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    fs::create_dir_all(project.join("assets/nested")).expect("创建资源目录失败");
    fs::write(project.join("assets/logo.txt"), "logo").expect("写入资源失败");
    fs::write(project.join("assets/nested/icon.svg"), "<svg/>").expect("写入嵌套资源失败");

    let stdout = expect_success(&asgard(&[
        "build",
        "--static",
        ENTRY,
        "--base",
        "/app/",
        "--assets",
        "assets",
        project.to_str().expect("UTF-8 路径"),
    ]));

    let dist = project.join("dist");
    let tree = artifact_tree(&dist);
    let expected: Vec<String> = [
        "asgard-static.css",
        "asgard-static.mjs",
        "asgard-static.wasm",
        "asgard-static.wat",
        "boot.js",
        "c/index.js",
        "index.html",
        "logo.txt",
        "manifest.json",
        "nested/icon.svg",
    ]
    .map(str::to_string)
    .to_vec();
    assert_eq!(tree, expected, "资源与产物必须共同构成完整产物树:\n{stdout}");

    // 页面壳写入部署基址，引用保持相对（由 `<base href>` 解析）。
    let html = fs::read_to_string(dist.join("index.html")).expect("读取页面失败");
    assert!(html.contains(r#"<base href="/app/">"#), "非根基址必须写入 `<base href>`:\n{html}");
    assert!(html.contains(r#"<script src="boot.js"></script>"#), "boot 引用必须相对基址:\n{html}");
    assert!(!html.contains(r#"src="/boot.js""#), "boot 引用不得写成根绝对路径:\n{html}");

    // 子进程守卫存活到测试结束；panic 时由 `Drop` 终止监听。
    let server = start_serve(&dist, "/app/");
    let address = server.address;
    let page = http_get(address, "/app/");
    assert_eq!(page.status, 200);
    assert_eq!(page.header("content-type"), Some("text/html; charset=utf-8"));
    assert!(String::from_utf8_lossy(&page.body).contains(r#"<base href="/app/">"#), "serve 应交付带基址的真实页面");

    let wasm = http_get(address, "/app/asgard-static.wasm");
    assert_eq!(wasm.status, 200);
    assert_eq!(wasm.header("content-type"), Some("application/wasm"), "Wasm 必须用 application/wasm");
    let wasm_len = fs::metadata(dist.join("asgard-static.wasm")).expect("读取 wasm 元数据失败").len();
    assert_eq!(wasm.header("content-length").map(str::to_string), Some(wasm_len.to_string()), "响应体长度必须等于落盘 chunk");
    assert_eq!(wasm.body.len() as u64, wasm_len, "serve 必须交付真实 chunk 字节");


    let glue = http_get(address, "/app/asgard-static.mjs");
    assert_eq!(glue.header("content-type"), Some("text/javascript; charset=utf-8"));

    let boot = http_get(address, "/app/boot.js");
    assert_eq!(boot.header("content-type"), Some("text/javascript; charset=utf-8"));

    let manifest = http_get(address, "/app/manifest.json");
    assert_eq!(manifest.header("content-type"), Some("application/json; charset=utf-8"));

    let css = http_get(address, "/app/asgard-static.css");
    assert_eq!(css.header("content-type"), Some("text/css; charset=utf-8"));

    let asset = http_get(address, "/app/logo.txt");
    assert_eq!(asset.header("content-type"), Some("text/plain; charset=utf-8"));

    let svg = http_get(address, "/app/nested/icon.svg");
    assert_eq!(svg.header("content-type"), Some("image/svg+xml"));


    // 部署前缀与越界路径必须拒绝。
    assert_eq!(http_get(address, "/asgard-static.wasm").status, 404, "未带 base 前缀必须拒绝");
    assert_eq!(http_get(address, "/app/../manifest.json").status, 404, "越界路径必须拒绝");

}

/// 真实 `asgard serve` 子进程；测试结束（含 panic）时强制终止，避免残留监听。
struct ServeProcess {
    child: Option<Child>,
    address: SocketAddr,
}

impl Drop for ServeProcess {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("分配空闲端口失败");
    let port = listener.local_addr().expect("读取监听地址失败").port();
    drop(listener);
    port
}

fn start_serve(dist: &Path, base: &str) -> ServeProcess {
    let port = free_port();
    let invocation = asgard_cli::resolve_asgard();
    let mut command = Command::new(&invocation.program);
    for prefix in &invocation.prefix {
        command.arg(prefix);
    }
    command.args([
        "serve".to_string(),
        "--root".to_string(),
        dist.to_str().expect("UTF-8 路径").to_string(),
        "--base".to_string(),
        base.to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        port.to_string(),
    ]);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("无法启动 asgard serve");
    let address = SocketAddr::from(([127, 0, 0, 1], port));

    for _ in 0..120 {
        if TcpStream::connect_timeout(&address, Duration::from_millis(200)).is_ok() {
            return ServeProcess { child: Some(child), address };
        }
        thread::sleep(Duration::from_millis(25));
    }

    let _ = child.kill();
    let output = child.wait_with_output().expect("读取 serve 诊断失败");
    panic!(
        "asgard serve 未在 5 秒内监听 {address}:\nstdout={}\nstderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 一次原始 HTTP/1.1 GET 的结果。
struct HttpReply {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl HttpReply {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(key, _)| key.eq_ignore_ascii_case(name)).map(|(_, value)| value.as_str())
    }
}

fn http_get(address: SocketAddr, path: &str) -> HttpReply {
    let mut stream = TcpStream::connect(address).expect("连接托管服务失败");
    write!(stream, "GET {path} HTTP/1.1\r\nHost: asgard.test\r\nConnection: close\r\n\r\n").expect("写入请求失败");
    stream.flush().expect("刷新请求失败");
    let mut reader = BufReader::new(stream);
    let mut status_line = String::new();
    reader.read_line(&mut status_line).expect("读取状态行失败");
    let status = status_line.split_whitespace().nth(1).and_then(|code| code.parse().ok()).unwrap_or(0);
    let mut headers = Vec::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).expect("读取头部失败") == 0 || line.trim().is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_string(), value.trim().to_string()));
        }
    }
    let mut body = Vec::new();
    reader.read_to_end(&mut body).expect("读取响应体失败");
    HttpReply { status, headers, body }
}
