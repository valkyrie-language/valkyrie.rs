# nyar-types

`nyar-types` 是当前 Rust 自举主链使用的最小共享类型集。

## 职责
- 提供当前主链实际需要的通用名类型，例如 `QualifiedName`。
- 提供最小兼容错误类型，例如 `NyarError`。
- 只服务于当前 workspace 打通，不承载语言级 `HIR / MIR / LIR`。

## 禁止
- 不在这里扩张成新的 `god object` 类型仓库。
- 不把后端容器、语言语义和运行时状态全塞进来。
- 不为了兼容旧遗产而引入大而全历史模型。
