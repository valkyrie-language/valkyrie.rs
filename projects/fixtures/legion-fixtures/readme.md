# legion-fixtures

Legion bootstrap/run 语料 fixture（供下游经 git `rev` 消费）。

## 职责

- 提供 `cmds_run/`、`bootstrap_node/`、`runtime_smoke/`、`text/` 等 Legion 黑盒语料
- 经 git 依赖被 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) 的 `rusty-valkyrie` 集成测试消费
- 不链接 Oak 解析链，避免语料维护与 lexer/parser 回归耦合

## 使用

```rust
use legion_fixtures::root;
let corpus = root().join("cmds_run");
```
