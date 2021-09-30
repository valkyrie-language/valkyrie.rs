# valkyrie-parser

`valkyrie-parser` 负责 Valkyrie 的语法层输入与语法树输出。

## 职责
- 读取源文件并构建 `ValkyrieRoot`。
- 维护强类型语法节点，例如 `NamespaceDeclaration`、`UsingStatement`、`FunctionDeclaration`。
- 为所有需要映射回源码的节点保留 `span`。
- 提供供编译器消费的稳定 parser facade。

## 禁止
- 不在这里做类型检查。
- 不在这里闭合语言语义。
- 不把语法树扩成万能 `AstItem` 或其他 `god object`。
- 不在这里引入 `HIR / MIR / LIR` 语义细节。
