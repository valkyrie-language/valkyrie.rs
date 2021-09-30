# compiler tests

这里放编译器侧测试。

## 职责
- 验证 `HIR / MIR / LIR`、类型检查、优化与模块解析。
- 保护 `CLR` 自举主线需要的编译契约。
- 防止实现只对 `CLR` 过拟合，兼顾 `JVM / WASM / native` 的最小护栏。
- 用 `spec/` 把 `row / trait / class / sealed class / unite / effect` 的语义规范先钉死，即便当前仍有已知缺口。

## 分层
- `type_checker/`: 类型检查、约束求解、名义子类型与 trait witness 事实。
- `pipeline/`: `AST -> HIR -> MIR -> target input` 主链边界。
- `optimizer/`: 静态化、去虚化与封闭类优化。
- `spec/`: 语义规范测试，允许先用 `#[ignore = "..."]` 保留尚未补齐的场景。
