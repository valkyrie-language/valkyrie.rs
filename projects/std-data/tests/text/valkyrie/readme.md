# parser tests

这里放 parser 侧回归测试。

## 职责
- 验证语法树形状、节点命名、`NamePath` 和 `span` 保真。
- 覆盖 parser 对真实 Valkyrie 源码的最小解析能力。

## 文本快照 fixture

lexer / parse 回归消费 `legion` 持有的 fixture（本 crate 不持有源文件）：

- 源文件：`legion/tests/fixtures/text/valkyrie/**/*.v`
- sidecar：`.lex` / `.parse`
- runner：`tests/text/valkyrie/fixtures/runner.rs`（`.v` + `.vx`）
- parse dump：分层 parser tree（[`parse_dump`](../../../src/text/valkyrie/parse_dump/mod.rs)）
- 重生成：`NYAR_TEST_REGENERATE=1 cargo test -p std-data text_fixture_lex_and_parse_regression`

详见 [`legion/tests/fixtures/text/valkyrie/readme.md`](../../../legion/tests/fixtures/text/valkyrie/readme.md)。

## 禁止
- 不把类型检查、优化、目标后端测试长期留在这里。
- 历史遗留测试目录在迁移前可暂存，但新增测试必须以 parser 职责为准。
- 新增 lexer / parse 覆盖应优先在 `legion/tests/fixtures/text/valkyrie/` 添加 fixture，而非手写 `TokenKind` / AST 字段断言。
- 不在 `std-data` 或仓库根目录持有语言源 fixture。
