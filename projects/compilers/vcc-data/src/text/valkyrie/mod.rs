//! Valkyrie 源码文本 facade — 重导出 [`oak_valkyrie`]（权威 parser 不在 `vcc-data`）。

pub use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage, ValkyrieLexer, ValkyrieParser, ValkyrieRoot, ast, lexer, parser, printer,
};
pub use oak_valkyrie::printer::parse_source;
