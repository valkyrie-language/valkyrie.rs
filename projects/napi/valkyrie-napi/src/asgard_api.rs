//! Asgard 程序化 N-API 产品面（`asgardBuildStatic` / `asgardServe`），不经 argv CLI。

use napi_derive::napi;

/// `asgardBuildStatic` 入参。
#[napi(object)]
pub struct AsgardStaticBuildOptions {
    /// 项目根目录。
    pub project_dir: String,
    /// 入口根组件文件路径（相对 `projectDir`，`.awsl` 规范拼写或同义缩写 `.w`，须显式给出）。
    pub entry: String,
    /// 输出目录；缺省由 Asgard 决定。
    pub output_dir: Option<String>,
    /// 页面标题；缺省为根组件名（入口文件 stem）。
    pub title: Option<String>,
    /// 产物基址（目录 URL）；缺省 `/`。
    pub base: Option<String>,
    /// 显式资源目录（相对 `projectDir`）；缺省不拷贝资源。
    pub assets_dir: Option<String>,
}

/// `asgardBuildStatic` 的结构化返回。
#[napi(object)]
pub struct AsgardStaticBuildOutcome {
    /// 是否成功。
    pub ok: bool,
    /// 失败时的错误描述。
    pub error: Option<String>,
    /// 实际写出产物的目录。
    pub output_dir: Option<String>,
    /// 产物 `index.html` 的完整路径。
    pub page_path: Option<String>,
    /// 根组件名。
    pub component_name: Option<String>,
}

/// `asgardServe` 入参。
#[napi(object)]
pub struct AsgardServeOptions {
    /// 产物根目录。
    pub root: String,
    /// 部署前缀（目录 URL）；缺省 `/`。
    pub base: Option<String>,
    /// 监听主机；缺省 `127.0.0.1`。
    pub host: Option<String>,
    /// 监听端口；缺省 `3000`，`0` 表示系统分配。
    pub port: Option<u32>,
}

/// `asgardServe` 的结构化返回；成功时进程内已启动服务线程，本调用阻塞直到进程退出。
#[napi(object)]
pub struct AsgardServeOutcome {
    /// 是否成功启动。
    pub ok: bool,
    /// 失败时的错误描述。
    pub error: Option<String>,
    /// 实际监听地址（`host:port`）。
    pub address: Option<String>,
}

/// 构建静态站点（页面 + WASM chunk + 岛包产物）并返回结构化结果。
#[napi]
pub fn asgard_build_static(options: AsgardStaticBuildOptions) -> AsgardStaticBuildOutcome {
    let outcome = language_valkyrie::product::build_static_site(language_valkyrie::product::StaticBuildOptions {
        project_dir: options.project_dir.into(),
        entry: options.entry.into(),
        output_dir: options.output_dir.map(Into::into),
        title: options.title,
        base: options.base,
        assets_dir: options.assets_dir.map(Into::into),
    });
    AsgardStaticBuildOutcome {
        ok: outcome.ok,
        error: outcome.error,
        output_dir: outcome.output_dir,
        page_path: outcome.page_path,
        component_name: outcome.component_name,
    }
}

/// 托管静态产物（`application/wasm` 等 MIME 由 Asgard `serve` 单一拥有）。
#[napi]
pub fn asgard_serve(options: AsgardServeOptions) -> AsgardServeOutcome {
    let outcome = language_valkyrie::product::serve_static(language_valkyrie::product::ServeOptions {
        root: options.root.into(),
        base: options.base,
        host: options.host,
        port: options.port.and_then(|port| u16::try_from(port).ok()),
    });
    AsgardServeOutcome { ok: outcome.ok, error: outcome.error, address: outcome.address }
}
