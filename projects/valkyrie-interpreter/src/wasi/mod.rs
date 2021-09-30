//! `WASI` 运行时宿主，基于 `wasmtime`。
//!
//! 为 `legion bootstrap --target wasi` 提供宿主支持：
//! 加载 `WASM` 模块，注入自定义导入函数，读取源文件并收集输出字节。
//!
//! 约定的导入函数（`env` 模块）：
//! - `read_source_byte() -> i32`：读取源文件下一字节，`EOF` 返回 `-1`
//! - `emit_byte(byte: i32)`：收集输出字节
//! - `add(a, b) -> i32`：整数加法
//! - `sub(a, b) -> i32`：整数减法
//! - `lt(a, b) -> i32`：小于比较，返回 `0` / `1`
//! - `eq(a, b) -> i32`：等于比较，返回 `0` / `1`

use std::path::Path;

use miette::{miette, IntoDiagnostic, Result};
use wasmtime::{Caller, Engine, Func, Instance, Module, Store};

/// `WASI` 运行时描述符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasiRuntime;

impl WasiRuntime {
    /// 运行时家族名称。
    pub const FAMILY: &'static str = "wasi";

    /// `ABI` 标识。
    pub const ABI: &'static str = "wasip1";

    /// 启动器名称。
    pub const LAUNCHER: &'static str = "wasmtime";
}

/// 宿主状态，在导入函数调用间共享。
struct HostState {
    /// 源文件字节流。
    source_bytes: Vec<u8>,
    /// 当前读取位置。
    source_pos: usize,
    /// 收集的输出字节。
    output_bytes: Vec<u8>,
}

impl HostState {
    /// 创建宿主状态，预加载源文件内容。
    fn new(source_bytes: Vec<u8>) -> Self {
        Self { source_bytes, source_pos: 0, output_bytes: Vec::new() }
    }
}

/// `WASI` 宿主运行结果。
#[derive(Debug)]
pub struct WasiHostResult {
    /// `main` 函数返回值。
    pub exit_code: i32,
    /// 收集的输出字节。
    pub output_bytes: Vec<u8>,
}

/// 运行 `WASM` 模块，注入自定义导入函数。
///
/// 参数：
/// - `wasm_path`：`WASM` 模块文件路径
/// - `source_path`：源文件路径（通过 `read_source_byte` 导入提供给模块）
///
/// 返回 `WasiHostResult`，包含 `main` 返回值和收集的输出字节。
pub fn run_wasi_module(wasm_path: &Path, source_path: &Path) -> Result<WasiHostResult> {
    let source_bytes = std::fs::read(source_path)
        .into_diagnostic()
        .map_err(|e| e.wrap_err(format!("读取源文件失败: {}", source_path.display())))?;

    let engine = Engine::default();
    let module = Module::from_file(&engine, wasm_path)
        .map_err(|e| miette!("加载 WASM 模块失败: {}: {}", wasm_path.display(), e))?;

    let mut store = Store::new(&engine, HostState::new(source_bytes));

    // 构建导入函数集合。
    let imports = build_imports(&mut store, &module);

    let instance = Instance::new(&mut store, &module, &imports)
        .map_err(|e| miette!("实例化 WASM 模块失败，可能是导入函数不匹配: {}", e))?;

    // 查找并调用 main 入口。
    let main_func = instance
        .get_func(&mut store, "main")
        .or_else(|| instance.get_func(&mut store, "_start"))
        .ok_or_else(|| miette!("WASM 模块未导出 main 或 _start 入口"))?;

    let mut results = [wasmtime::Val::I32(0)];
    main_func
        .call(&mut store, &[], &mut results)
        .map_err(|e| miette!("调用 main 函数失败: {}", e))?;

    let exit_code = match results[0] {
        wasmtime::Val::I32(v) => v,
        _ => 0,
    };

    // 取回收集的输出字节。
    let output_bytes = std::mem::take(&mut store.data_mut().output_bytes);

    Ok(WasiHostResult { exit_code, output_bytes })
}

/// 运行 `WASM` 模块并将输出写入指定文件。
///
/// 这是 `run_wasi_module` 的封装，将收集的输出字节写入 `output_path`。
/// 如果没有输出路径，则将 `main` 返回值作为退出码。
pub fn run_wasi_module_to_file(wasm_path: &Path, source_path: &Path, output_path: Option<&Path>) -> Result<i32> {
    let result = run_wasi_module(wasm_path, source_path)?;

    if let Some(output) = output_path {
        if !result.output_bytes.is_empty() {
            std::fs::write(output, &result.output_bytes)
                .into_diagnostic()
                .map_err(|e| e.wrap_err(format!("写入输出文件失败: {}", output.display())))?;
        }
        Ok(0)
    }
    else {
        Ok(result.exit_code)
    }
}

/// 构建 `WASM` 模块所需的导入函数集合。
///
/// 根据 `Module` 的 `import` 列表，按顺序提供对应的 `Func`。
/// 支持的导入模块为 `env`，函数名包括：
/// `read_source_byte` / `emit_byte` / `add` / `sub` / `lt` / `eq`。
fn build_imports(store: &mut Store<HostState>, module: &Module) -> Vec<wasmtime::Extern> {
    let mut imports = Vec::new();

    for import in module.imports() {
        let func = match (import.module(), import.name()) {
            ("env", "read_source_byte") => {
                Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>| -> i32 {
                    let state = caller.data_mut();
                    if state.source_pos < state.source_bytes.len() {
                        let byte = state.source_bytes[state.source_pos] as i32;
                        state.source_pos += 1;
                        byte
                    }
                    else {
                        -1
                    }
                })
            }
            ("env", "emit_byte") => {
                Func::wrap(&mut *store, |mut caller: Caller<'_, HostState>, byte: i32| {
                    let state = caller.data_mut();
                    state.output_bytes.push((byte & 0xFF) as u8);
                })
            }
            ("env", "add") => Func::wrap(&mut *store, |a: i32, b: i32| -> i32 { a.wrapping_add(b) }),
            ("env", "sub") => Func::wrap(&mut *store, |a: i32, b: i32| -> i32 { a.wrapping_sub(b) }),
            ("env", "lt") => Func::wrap(&mut *store, |a: i32, b: i32| -> i32 { if a < b { 1 } else { 0 } }),
            ("env", "eq") => Func::wrap(&mut *store, |a: i32, b: i32| -> i32 { if a == b { 1 } else { 0 } }),
            _ => {
                continue;
            }
        };
        imports.push(wasmtime::Extern::Func(func));
    }

    imports
}
