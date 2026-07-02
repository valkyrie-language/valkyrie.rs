# CLR (.NET) 后端

CLR (Common Language Runtime) 使用基于栈的 CIL。本仓库的正式生成路径是：**MIR → 类型化 MSIL 模型 → 手写 PE/CLI 二进制**，与 `valkyrie.v` 侧 `std.data.binary.pe` 同构。

## 1. 编译流程

```text
FragmentSubmission (含 MIR)
  -> nyar-emitter/lowering/backends/clr  (MSIL 模块)
  -> std-data/binary/pe::PeWriter        (ECMA-335 PE/COFF + CLI 元数据)
  -> .dll / .exe
```

- **入口**：`projects/nyar-emitter/src/lowering/backends/clr/`（`mod.rs`、`mir.rs`、`types.rs` 等）。
- **二进制**：`projects/std-data/src/binary/pe/`（`PeWriter`；模块注释明确「不依赖 ilasm」）。
- **MSIL 文本模型**：`projects/std-data/src/text/msil/`（编码方法体与元数据行，再由 PE writer 落盘）。
- 栈机后端不做寄存器分配；从 MIR 生成 `ldloc` / `stloc` / `call` 等 CIL，再写入方法体与元数据表。

## 2. 硬性约束（生成路径）

| 允许 | 禁止（不得作为生成器） |
| --- | --- |
| 手写 PE/CLI/IL（`PeWriter` + MSIL 模型） | `ilasm`、`dnlib`、Mono.Cecil、`csc`、`System.Reflection.Emit` 产盘 |
| `dotnet` / 运行时仅用于**执行与验读** | 把外部汇编器当正式流水线一环 |
| `ildasm` 等仅用于 spy/调试检视 | 依赖外部工具写出可分发程序集 |

自举目标 `valkyrie.v` 必须与上述 seed 同构：同样手写 PE/CLI，不得改走 dnlib/ilasm。

## 3. 类型映射要点

- 语言层**没有**名为 `string` 的类型；文本为 `utf8` / `utf16`（及 `std.text.Utf8Text` / `Utf16Text`）。
- ABI 上二者都可能落在 `System.String` 堆对象，但语义必须分叉：
  - **Utf16**：code-unit 索引（可直连 `get_Length` / `Substring` 等）。
  - **Utf8**：Unicode **标量**索引（经 `std.adaptor.clr.text`，禁止误降为 UTF-16 code-unit API）。
- `void`（ADT 代数 0，平台层常为 `NyarType::Bottom`）与 `unit`（ADT 代数 1，`NyarType::Unit`）**禁止混淆**。返回签名在 CIL 上可能都编码为 `void`，但语言/IR 语义仍须区分；`LocalVarSig` 不得使用 `ELEMENT_TYPE_VOID`（ECMA-335）。

## 4. 与 JVM / WASM 的差异

1. **值类型**：`structure` 可映射为 CLR `valuetype`。
2. **泛型**：CLR 保留运行时特化信息（长期能力；当前以已实现路径为准）。
3. **尾调用**：CIL 支持 `tail.` 前缀。
4. **FFI**：`[clr(...)]` / CLR 专用导入与 WASM/WASI **完全独立**，不做跨目标垫片混用。

## 5. 特性处理（摘要）

### Trait

- 具名 `trait` 可映射接口；默认实现与 CLR 接口默认方法方向一致。开放 witness 须在进入后端前闭合或显式失败。

### 代数效应

- 非 `resume` 的 `raise` 可映射 `throw`。
- 可恢复路径需状态机重写（类似 async / iterator），见 `clr/suspend.rs`。

### 内存

- 托管堆与分代 GC；终结逻辑可对齐 `IDisposable` 模式。

## 6. 实施状态（以源码为准）

- [x] MIR → MSIL 降低与类型映射（含 `utf8` / `utf16`、`structure`→valuetype、ADT/unite 基础）
- [x] 手写 `PeWriter` 生成 PE/CLI（**无 ilasm**）
- [x] 入口点、字段/数组投影、尾调用等已在后端落地的能力
- [ ] 完整可恢复代数效应
- [ ] 更完整的 CLR 专用 FFI 面与集成测试矩阵

短期重点：效应路径、FFI 契约、数组/切片边界、与 `valkyrie.v` PE 包字节级同构验证。  
长期重点：CLR 原生泛型、效应与异常边界的进一步对齐——**不再**把「引入 PE 摆脱 ilasm」列为待办（已完成）。
