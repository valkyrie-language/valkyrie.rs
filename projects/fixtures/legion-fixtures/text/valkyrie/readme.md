# Valkyrie text snapshot fixtures

Legion 工程使用的 Valkyrie 源文件与文本 sidecar 基线。fixture crate 持有语料，Nyar 测试工具在运行时读取这些文件。

## 文件形态

每个源文件配多个文本 sidecar，同级存放：

```
lexer/angle_less.v
lexer/angle_less.v.lex
lexer/angle_less.v.parse
lexer/angle_less.v.highlight
```

## 输出协议

- `*.lex`：lossless token 流，每行 `Token { kind, span, text }`
- `*.parse`：分层 parser tree dump——CST 壳（Trivia / Statement / Error）+ AST / Xg / Tg 子树展开；每节点含稳定 `kind` 与
  `span`
- `*.highlight`：merged highlight spans（lexical + semantic），每行 `Span { kind, span, text, modifier }`；语义层为空时等价于
  lexical merged

文本输出由对应的 formatter/printer 生成，sidecar 保存稳定的 token、parse 和 highlight 结果。

## 重生成

```powershell
$env:NYAR_TEST_REGENERATE = "1"
cargo test -p std-data text_fixture_lex_and_parse_regression
cargo test -p nyar-language text_fixture_highlight_regression
```

也支持 `VALKYRIE_TEST_REGENERATE` 与 `LEGION_TEST_REGENERATE`（宽语义：`1` / `true` / `yes` / `on` / `regenerate`）。

首次运行缺少 sidecar 时会自动创建，无需显式 regenerate。

## 目录约定

| 子目录     | 覆盖范围                                                   |
|------------|------------------------------------------------------------|
| `lexer/`   | lossless token、trivia、XML token、字符串字面量            |
| `core/`    | 核心语法：参数、类型表达式、construct                      |
| `surface/` | 声明级 surface form：widget、flags、union/enums            |
| `vx/`      | `.vx` widget markup、T-Grammar meta fixup（走 `parse_vx`） |

## 消费方

| 用途 | Sidecar |
|------|---------|
| 词法与解析回归 | `.lex` / `.parse` |
| 高亮回归 | `.highlight` |

语料根目录由 `legion-fixtures::root()` 提供，测试通过该入口定位 `text/valkyrie`。
