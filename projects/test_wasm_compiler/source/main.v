# test_wasm_compiler: WASM 微编译器
#
# 这是一个自举用的微编译器，通过 `wasm_import` 从 `Node` 启动壳获取 I/O 能力。
# 微编译器读取源文件字节，统计字节数，生成一个返回该计数的 WASM 模块。
#
# 这是诚实的自举：输出依赖于源文件内容，而非固定模板。
# v1（本编译器编译为 WASM）运行时读取源文件，产生 v2.wasm。
# v2.wasm 是一个返回源文件字节数的最小 WASM 模块。
#
# 约定的导入函数：
# - env.read_source_byte() -> i32：读取源文件下一字节，EOF 返回 -1
# - env.emit_byte(byte: i32)：收集输出字节
# - env.add(a, b) -> i32：整数加法
# - env.sub(a, b) -> i32：整数减法
# - env.lt(a, b) -> i32：小于比较，返回 0/1
# - env.eq(a, b) -> i32：等于比较，返回 0/1

[wasm_import("env", "read_source_byte")]
micro read_source_byte(): i32;

[wasm_import("env", "emit_byte")]
micro emit_byte(byte: i32): unit;

[wasm_import("env", "add")]
micro add(a: i32, b: i32): i32;

[wasm_import("env", "sub")]
micro sub(a: i32, b: i32): i32;

[wasm_import("env", "lt")]
micro lt(a: i32, b: i32): i32;

[wasm_import("env", "eq")]
micro eq(a: i32, b: i32): i32;

[main]
micro main(): i32 {
    let byte = read_source_byte();
    let count = 0;
    while eq(lt(byte, 0), 0) {
        count = add(count, 1);
        byte = read_source_byte();
    }
    # SLEB128 编码（2 字节）：count = quotient * 128 + remainder
    # 限制：count 最大 16383（2 字节 SLEB128 上限）
    let quotient = 0;
    let remainder = count;
    while lt(127, remainder) {
        remainder = sub(remainder, 128);
        quotient = add(quotient, 1);
    }
    # 生成 WASM 模块（返回 count）
    # 模块结构固定，但 i32.const 的值由源码字节数决定
    # magic: \0asm
    emit_byte(0);
    emit_byte(97);
    emit_byte(115);
    emit_byte(109);
    # version: 1
    emit_byte(1);
    emit_byte(0);
    emit_byte(0);
    emit_byte(0);
    # Type section: 1 type, () -> i32
    emit_byte(1);
    emit_byte(5);
    emit_byte(1);
    emit_byte(96);
    emit_byte(0);
    emit_byte(1);
    emit_byte(127);
    # Function section: 1 function, type 0
    emit_byte(3);
    emit_byte(2);
    emit_byte(1);
    emit_byte(0);
    # Export section: "main" -> func 0
    emit_byte(7);
    emit_byte(8);
    emit_byte(1);
    emit_byte(4);
    emit_byte(109);
    emit_byte(97);
    emit_byte(105);
    emit_byte(110);
    emit_byte(0);
    emit_byte(0);
    # Code section: 1 function body
    # body = 0 locals + i32.const(2 byte SLEB128) + end = 5 bytes
    emit_byte(10);
    emit_byte(7);
    emit_byte(1);
    emit_byte(5);
    emit_byte(0);
    emit_byte(65);
    # SLEB128 编码的 count（2 字节）
    emit_byte(add(remainder, 128));
    emit_byte(quotient);
    emit_byte(11);
    return 0;
}
