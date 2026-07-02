# 降级指南 (Lowering)

## 1. 现状：语义到目标

现行降低不是「全部进 Chomsky UIR 再由 Gaia 统一发射」，而是：

```text
AST -> HIR -> MIR
  -> FragmentSubmission
  -> 目标专用 lowering (clr / jvm / wasm / …)
  -> 手写二进制编解码 (std-data)
```

语言特性（模式匹配、效应、名义类型等）应在 **HIR/MIR** 阶段定界；后端只翻译已闭合的可执行表示。

## 2. 愿景对照（勿混入现行）

| 旧叙述 | 现行理解 |
| --- | --- |
| Oaks AST → UIR → Gaia | `nyar-language` → `nyar-emitter` + `std-data` |
| 后端只需代价模型 | 各目标手写指令与二进制格式 |
| 统一 intent 树覆盖 CLR/JVM/WASM | 分后端 MIR lower + 分区规划 |

## 3. 特性降低摘要

### 模式匹配

- AST→HIR：模式与类型绑定、穷尽性。
- MIR：分支与提取器；unite payload 等由各后端按约定发射（如 CLR tag+payload）。

### 效应

- HIR/MIR 保留 raise/handler/resume 边界。
- CLR/JVM/WASM 各自映射 throw / 状态机 / CPS 或 stack-switch（见各后端文档）。

### 文本与单位类型

- 降低时保持 `utf8`/`utf16` 分叉；禁止引入语言 `string`。
- `void`/`unit` 不得在 lowering 中被 `is_unitish` 类逻辑抹成同一语义（ABI 折叠须显式注释）。

## 4. 自举同构

`valkyrie.v` 中的 `std.data.binary.*` 应与 seed 的 `std-data` 手写路径对齐；禁止改走外部工具生成器。
