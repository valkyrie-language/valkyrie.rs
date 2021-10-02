use std::fs;

use miette::{IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::ArtifactFormat,
    packaging::{ArtifactDescriptor, TargetLane},
};

use super::{BindingGenerationContext, HostBindingBuilder};

/// `WASM + JS glue` 宿主绑定生成器。
pub(crate) struct JsGlueBindingBuilder;

impl HostBindingBuilder for JsGlueBindingBuilder {
    fn build(&self, context: &BindingGenerationContext<'_>) -> Result<Vec<ArtifactDescriptor>> {
        let launcher_path = context.output_dir.join(format!("{}.mjs", context.artifact_name));
        let launcher = build_node_launcher(context.artifact_name, context.imports);
        fs::write(&launcher_path, launcher).into_diagnostic().wrap_err_with(|| format!("写入 Node 启动壳失败：{}", launcher_path.display()))?;

        Ok(vec![ArtifactDescriptor {
            name: format!("{}.launcher", context.artifact_name),
            kind: nyar::ArtifactKind::AssemblyListing,
            format: ArtifactFormat::RawBinary,
            target: context.target.clone(),
            lane: TargetLane::Wasm,
        }])
    }
}

/// 构建 `Node` 启动壳（`.mjs`）。
///
/// 当 `WASM` 模块有导入函数时，启动壳提供以下约定的实现：
/// - `env.get_input() -> i32`：从命令行参数读取输入值
/// - `env.read_source_byte() -> i32`：从源文件读取下一个字节，`EOF` 返回 `-1`
/// - `env.emit_byte(byte: i32)`：收集输出字节
/// - `env.emit_i32(value: i32)`：以小端序收集 4 字节
/// - `env.add(a: i32, b: i32) -> i32`：整数加法
/// - `env.sub(a: i32, b: i32) -> i32`：整数减法
/// - `env.mul(a: i32, b: i32) -> i32`：整数乘法
/// - `env.lt(a: i32, b: i32) -> i32`：小于比较，返回 `0`/`1`
/// - `env.gt(a: i32, b: i32) -> i32`：大于比较，返回 `0`/`1`
/// - `env.le(a: i32, b: i32) -> i32`：小于等于比较，返回 `0`/`1`
/// - `env.ge(a: i32, b: i32) -> i32`：大于等于比较，返回 `0`/`1`
/// - `env.eq(a: i32, b: i32) -> i32`：等于比较，返回 `0`/`1`
/// - `env.ne(a: i32, b: i32) -> i32`：不等于比较，返回 `0`/`1`
///
/// 启动壳执行流程：
/// 1. 读取命令行参数：`node launcher.mjs [source_path|input_value] [output_path]`
/// 2. 若声明了 `read_source_byte`，则 `argv[2]` 为源文件路径
/// 3. 如果有 `output_path`，将收集的字节写入该文件
/// 4. 否则将 `main()` 返回值作为退出码
fn build_node_launcher(artifact_name: &str, imports: &[(String, String)]) -> String {
    let has_imports = !imports.is_empty();
    let has_read_source = imports.iter().any(|(_, field)| field == "read_source_byte");

    let import_object = if has_imports {
        let mut env_entries = Vec::new();
        for (_module, field) in imports {
            let impl_code = match field.as_str() {
                "get_input" => "() => input_value",
                "read_source_byte" => "() => { if (source_pos < source_bytes.length) { return source_bytes[source_pos++]; } return -1; }",
                "emit_byte" => "(b) => { output_bytes.push(b & 0xFF); }",
                "emit_i32" => "(v) => { output_bytes.push(v & 0xFF, (v >> 8) & 0xFF, (v >> 16) & 0xFF, (v >> 24) & 0xFF); }",
                "add" => "(a, b) => (a + b) | 0",
                "sub" => "(a, b) => (a - b) | 0",
                "mul" => "(a, b) => (a * b) | 0",
                "lt" => "(a, b) => a < b ? 1 : 0",
                "gt" => "(a, b) => a > b ? 1 : 0",
                "le" => "(a, b) => a <= b ? 1 : 0",
                "ge" => "(a, b) => a >= b ? 1 : 0",
                "eq" => "(a, b) => a === b ? 1 : 0",
                "ne" => "(a, b) => a !== b ? 1 : 0",
                _ => "() => { throw new Error(`未实现的导入: ${field}`); }",
            };
            env_entries.push(format!("    {field}: {impl_code},"));
        }
        format!(
            r#"const importObject = {{
  {module}: {{
{entries}
  }}
}};
const {{ instance }} = await WebAssembly.instantiate(wasmBytes, importObject);"#,
            module = imports[0].0,
            entries = env_entries.join("\n")
        )
    }
    else {
        r#"const { instance } = await WebAssembly.instantiate(wasmBytes, {});"#.to_string()
    };

    let output_logic = if has_imports {
        r#"
// 如果有输出路径参数，将收集的字节写入文件
if (output_path && output_bytes.length > 0) {
    const { writeFileSync } = await import("node:fs");
    writeFileSync(output_path, Buffer.from(output_bytes));
    process.exit(0);
} else if (typeof result === "number") {
    process.exit(result);
}"#
    }
    else {
        r#"
if (typeof result === "number") {
    process.exit(result);
}"#
    };

    let arg_parsing = if has_imports {
        if has_read_source {
            r#"const source_path = process.argv[2] || null;
const output_path = process.argv[3] || null;
let source_bytes = Buffer.alloc(0);
let source_pos = 0;
if (source_path) {
    try {
        source_bytes = await readFile(source_path);
    } catch (e) {}
}
const input_value = 0;
const output_bytes = [];
"#
        }
        else {
            r#"const input_value = process.argv[2] ? parseInt(process.argv[2], 10) : 0;
const output_path = process.argv[3] || null;
const output_bytes = [];
"#
        }
    }
    else {
        ""
    };

    format!(
        r#"import {{ readFile }} from "node:fs/promises";
{arg_parsing}const wasmBytes = await readFile(new URL("./{name}.wasm", import.meta.url));
{import_object}
const exports = instance.exports;
const entry = exports.main ?? exports._start;
let result;
if (typeof entry === "function") {{
    result = entry();
}}
{output_logic}
"#,
        arg_parsing = arg_parsing,
        name = artifact_name,
        import_object = import_object,
        output_logic = output_logic
    )
}
