//! `valkyrie-wasi-host`：`WASI` 宿主二进制入口。
//!
//! 用法：`valkyrie-wasi-host <wasm_file> <source_file> [output_file]`
//!
//! 加载 `WASM` 模块，注入自定义导入函数（`read_source_byte` / `emit_byte` / 算术辅助），
//! 读取源文件并收集输出字节。如果提供 `output_file`，将输出写入文件；否则以 `main` 返回值退出。

use std::path::PathBuf;

use miette::Result;
use valkyrie_interpreter::wasi::run_wasi_module_to_file;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("用法: {} <wasm_file> <source_file> [output_file]", args.first().map(String::as_str).unwrap_or("valkyrie-wasi-host"));
        std::process::exit(1);
    }

    let wasm_path = PathBuf::from(&args[1]);
    let source_path = PathBuf::from(&args[2]);
    let output_path = if args.len() >= 4 { Some(PathBuf::from(&args[3])) } else { None };

    let exit_code = run_wasi_module_to_file(&wasm_path, &source_path, output_path.as_deref())?;

    std::process::exit(exit_code);
}
