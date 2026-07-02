# WASM / WASI 后端

本文同时覆盖 **core WASM** 与 **WASI（含组件模型）**。二者共享 MIR 降低与手写二进制层；差异在宿主导入与组件包装。

## 现状（以源码为准）

```text
FragmentSubmission (含 MIR)
  -> nyar-emitter/lowering/backends/wasm
  -> std-data/binary/wasm   (core module / sections 手写编解码)
  -> 可选：组件模型 / WASI host 路径 (host/wasi_cm 等)
```

- **无语言类型 `string`**：文本为 `utf8` / `utf16`（及 `Utf8Text` / `Utf16Text`）。WASI Canonical ABI 中的「字符串」对应 **utf8 字节序列**（常与 `list<u8>` 对接），不是语言级 `string`。
- **`void` ≠ `unit`**：`unit` 在 WASM 返回位常为无结果；`void`/`Bottom` 表示永不返回。禁止混用。
- **GC**：wasm/wasi 目标下 GC 为语言规范**强制**要求，不可关闭（线性内存仍可用于底层 FFI）。
- **生成路径**：正式产物应通过直接写入 WASM / Component sections 完成，与 `valkyrie.v` 的 `std.data.binary.wasm` **同构**。`wit-component` **不得**作为正式生成器；`wasm-tools` / `wasmtime` 仅宜用于验读或测试，不得依赖其为自举产盘路径。

## 设计摘要

### 类型与运算

- 映射：`i32`/`i64`/`f32`/`f64`/`bool`/`utf8`/`utf16`/`unit` 等。
- 指针、引用、数组、类在线性内存视角下常为 `i32`（wasm32）或 `i64`（wasm64）；GC 路径使用 `struct`/`array`/引用类型。

### WASM vs WASI

| | WASM（core / 宿主如 JS glue） | WASI |
| --- | --- | --- |
| 重点 | GC 对象、线性内存、指令发射 | Preview 组件、`[wasi(...)]` 导入、Canon ABI |
| Host | 如 `env.utf8_*`（标量约定） | `wasi:cli` / `wasi:io` 等 |
| 适配器 | `std.adaptor.wasm` | `std.adaptor.wasi` |

### 组件模型

- 目标：手写 Component Section、别名与 `canon lower`，以支持 `list<u8>`（utf8）传递。
- Binaryen **asyncify** 不再作为首选效应方案；可恢复路径优先 stack-switching 或 CPS（见 emitter 配置）。

### Trait / 效应

- Witness table（胖指针）+ `call_indirect`。
- 非 resume 的 `raise` 优先映射异常处理提案指令；完整 resume 仍在演进。

## 与历史叙述的区别

旧文档中的 `UIR (Chomsky)` 流水线、以及「CFG→UIR 转型中」的表述**不是**当前主叙述。现行为 **MIR → wasm 后端**。部分集成测试与组件封装代码若仍调用外部 `wasm-tools`，视为过渡/验读，**自举与正式文档约束以手写 section 为准**。

## 待办方向（摘要）

- 效应：non-resumable `throw`、handler、`try_table`；可恢复路径。
- Witness 布局与 `call_indirect` 完备化。
- GC 字符串/数组动态分配与线性内存 FFI 桥。
- 与 `valkyrie.v` wasm 包字节级同构验证。
