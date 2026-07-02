# 体系化自举规范

本文是维护者规范，不是用户指南。自举的目标是证明：

`Rust seed -> v1 -> v1 编译同一源码 -> v2 -> 四后端 runtime`

证明不以进度、artifact 数量或通过率为指标。无法建立 `source -> semantic contract -> artifact contract -> runtime` 来源链的结果一律拒绝。

## 语义基准

neutral semantics 是唯一独立语义基准。它不模拟 CLR/JVM 栈、WASM GC index、WASI ABI、JS handle 或 runtime state。所有 compiler 和 runtime probe 必须与其可观察结果一致。

## 契约边界

每个 package 独立完成 Parse、HIR、Semantic MIR、semantic validation 和 Semantic Package Interface。依赖源码不得拼接进 consumer parser。`nyar`、`nyar-types`、`nyar-analyzer`、`nyar-optimizer`、`nyar-emitter`、package-manager、package-registry 保持语言无关；只有 `nyar.language` 适配具体语言。

执行器只接受已验证的 Backend Artifact Contract：CLR、JVM、编译器生成的 WASM+Node.js glue、WASI Component+Wasmtime。执行器不得读取源码、猜测类型或回调 compiler。

## 类型和 definition

`T?` 是 `Nullable<T>`；`Option<T>` 是显式 nominal sum；`Utf8` 与 `Utf16` 不同；不存在无编码限定的 language-level string。`f64` 只能由 definition/primitive registry 建立 canonical `Float64` identity；`imply f64` 不能建立 primitive identity。

## 反作弊证据

静态扫描只是结构 guard。完整门禁必须包含语义差分、黑盒 adversarial probes 和字段级 provenance。改名、同名异义、package 重排、类型/编码/metadata/ABI 扰动都必须保持正确语义或明确失败。运行成功但来源不可追踪，不产生任何正面状态。

## 状态

状态严格单向：`diagnostic-only`、`contract-proven`、`lane-proven`、`bootstrap-proven`。只有四条 lane 全部完成 link、artifact verify、runtime probe，且 Rust seed/v1/v2 完整证据链存在时，才允许 `bootstrap-proven`。
