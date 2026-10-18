# valkyrie-fixtures

Valkyrie 语言 Oak 解析器回归 fixture（仅测试用）。

## 职责

- 为 `oak-valkyrie` 词法、格式化、打印、解析与 builder 提供快照测试
- 不链接 `legion` 生产链，避免 parser 回归与交付 lane 耦合

## 运行

```bash
cargo test -p valkyrie-fixtures
```
