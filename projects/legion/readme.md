# legion

`legion` 是 Valkyrie 工作区的聚合入口与编排前台。

## 职责
- 提供 `legion` CLI 入口。
- 负责 workspace、manifest、build plan、target 选择与任务编排。
- 聚合 `valkyrie-parser`、`valkyrie-compiler`、`valkyrie-interpreter` 的能力。

## 禁止
- 不在这里实现语言主链。
- 不在这里定义 `AST / HIR / MIR / LIR`。
- 不在这里堆积目标后端细节或运行时实现。
- 不引入新的统一 `god ir`。
