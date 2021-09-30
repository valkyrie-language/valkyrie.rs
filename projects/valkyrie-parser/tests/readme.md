# parser tests

这里放 parser 侧回归测试。

## 职责
- 验证语法树形状、节点命名、`NamePath` 和 `span` 保真。
- 覆盖 parser 对真实 Valkyrie 源码的最小解析能力。

## 禁止
- 不把类型检查、优化、目标后端测试长期留在这里。
- 历史遗留测试目录在迁移前可暂存，但新增测试必须以 parser 职责为准。
