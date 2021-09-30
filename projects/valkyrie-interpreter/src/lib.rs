//! `valkyrie-interpreter`：基于 `wasmtime` 的 `WASI` 运行时宿主。
//!
//! 为 `legion bootstrap --target wasi` 提供宿主支持：
//! 加载 `WASM` 模块，提供自定义导入函数（`read_source_byte` / `emit_byte` / 算术辅助），
//! 读取源文件并收集输出字节。

pub mod wasi;

pub use wasi::WasiRuntime;
