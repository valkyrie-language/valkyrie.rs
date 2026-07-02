# Valkyrie 错误处理与诊断指南

Valkyrie 采用结构化、国际化优先的诊断思路。下文的 `ValkyrieError` 草图表达**设计意图**；具体类型名与模块以各 crate 现行 `miette`/`Diagnostic` 实现为准（工作区已无独立的 `valkyrie-error` crate）。

## 1. 设计原则

- **结构化 (Structured)**: 错误应包含错误代码、i18n 键和参数化数据。
- **国际化 (i18n)**: 错误消息不应硬编码，而是通过 `key` 在不同语言的本地化文件中查找。
- **轻量级 (Lightweight)**: 使用 `Box<ErrorKind>` 模式，确保错误类型的大小保持最小，优化性能。
- **结构化诊断优先**: 新代码优先使用 `miette` / 显式错误类型与帮助信息，避免把失败吞成无上下文字符串。
- **宏依赖政策**: 语言主链与 emitter 诊断面以显式实现为主。工作区根 `Cargo.toml` 仍声明 `thiserror` 供部分包管理/工具 crate 使用——**不得**把「全仓库严禁 thiserror」写成与 lockfile 矛盾的绝对禁令；新诊断代码优先 `miette`，避免再扩散 `anyhow` 风格上下文丢失。

## 2. 核心数据结构

### `ValkyrieError`

这是所有错误的顶层容器：

```rust
pub structure ValkyrieError {
    pub code: u32,                  // 唯一的错误代码，如 0x001
    pub kind: Box<ValkyrieErrorKind>, // 具体的错误分类（包含 i18n key 和数据）
    pub labels: Vec<ValkyrieLabel>, // 源码标注（支持 i18n）
    pub help: Option<ValkyrieHelp>, // 修复建议（支持 i18n）
}

pub structure ValkyrieLabel {
    pub span: SourceSpan,
    pub key: &'static str, // 标注内容的 i18n key
    pub data: BTreeMap<String, String>, // 标注内容的 i18n 参数
    pub primary: bool,
}

pub structure ValkyrieHelp {
    pub key: &'static str,          // 修复建议的 i18n key
    pub data: BTreeMap<String, String>, // 修复建议的 i18n 参数
}
```

### `ValkyrieErrorKind`

使用枚举对错误进行分类，每个变体直接携带该错误所需的参数化数据。`key` 由变体类型隐式决定：

```rust
pub enum ValkyrieErrorKind {
    UnexpectedToken { expected: Vec<String>, found: String },
    UndefinedVariable { name: String },
    TypeMismatch { expected: String, found: String },
    // ...
}

impl ValkyrieErrorKind {
    pub fn key(&self) -> &'static str {
        match self {
            Self::UnexpectedToken { .. } => "error.syntax.unexpected_token",
            Self::UndefinedVariable { .. } => "error.semantic.undefined_variable",
            // ...
        }
    }
}
```

## 3. 实现示例

### 触发一个语法错误

```rust
impl ValkyrieError {
    pub fn unexpected_token(span: SourceSpan, expected: Vec<String>, found: String) -> Self {
        Self {
            code: 1001,
            kind: Box::new(ValkyrieErrorKind::UnexpectedToken { expected, found }),
            labels: vec![ValkyrieLabel::primary(span, Some("unexpected.token"))],
            help: Some(ValkyrieHelp::new("check.syntax.manual")),
        }
    }
}
```

### 使用 Builder 模式快速构造

```rust
let error = ValkyrieError::new(1001, kind)
    .add_label(ValkyrieLabel::primary(span, Some("label.key")))
    .with_help(ValkyrieHelp::new("help.key").with_data("arg", "value"));
```

### 快速构造 IO 错误

```rust
impl ValkyrieError {
    pub fn io_error(error: std::io::Error, path: Option<Pathbuf>, ) -> Self {
        Self::new(
            0x0001,
            ValkyrieErrorKind::IoError {
                path,
                message: error.to_string(),
            },
        )
    }
}
```

## 4. 国际化 (i18n) 流水线

1. **触发错误**: 编译器在发现问题时构造 `ValkyrieError`。
2. **携带数据**: 错误对象仅携带 `key` 和 `data`（如 `{ "name": "foo" }`）。
3. **渲染诊断**: 
   - 诊断系统根据当前语言环境加载翻译文件（如 `zh-CN.toml`）。
   - 使用 `key` 找到模板：`error.type.mismatch = "类型不匹配：期望 {expected}，但找到了 {found}"`。
   - 将 `data` 中的值填充进模板。

## 5. 最佳实践

- **保持 ErrorKind 纯净**: 仅包含用于进一步分析或恢复的必要数据。
- **延迟渲染**: 不要在构造错误时格式化字符串。格式化应仅在最终输出给用户时进行。
- **使用 Error Code**: 面向用户的错误宜分配稳定错误代码，便于查阅文档。
- **避免 `anyhow` 吞类型**: 编译主链需要可匹配的错误种类以决定是否继续后续阶段；不要用无类型擦除的上下文错误替代诊断枚举。
- **宏与手写 Diagnostic**: 新诊断优先 `miette` + 显式 `Diagnostic`/`Display`。部分工具 crate 已使用 `thiserror`（见根 `Cargo.toml`）——不必为「绝对禁止」而强行回写；主链新代码仍优先显式实现。
