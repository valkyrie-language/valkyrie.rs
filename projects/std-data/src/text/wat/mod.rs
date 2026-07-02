#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

mod lexer;
mod parser;

pub use lexer::{Lexer, Token, TokenKind};
pub use parser::Parser;

/// `WAT` 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatError {
    /// 文本不是合法的 `module` 形式。
    InvalidModule(String),
    /// 括号没有正确闭合。
    UnbalancedParentheses,
    /// 字符串字面量没有结束。
    UnterminatedString,
}

impl Display for WatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidModule(message) => write!(f, "无效的 `WAT` 模块：{message}"),
            Self::UnbalancedParentheses => write!(f, "`WAT` 模块括号不平衡"),
            Self::UnterminatedString => write!(f, "`WAT` 字符串字面量未闭合"),
        }
    }
}

impl std::error::Error for WatError {}

/// `WAT` 文本文档模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WatDocument {
    /// 可选模块名。
    pub module_name: Option<String>,
    /// 顶层模块字段文本。
    pub fields: Vec<String>,
}

impl WatDocument {
    /// 创建一个新的空 `WAT` 模块。
    pub fn new() -> Self {
        Self { module_name: None, fields: Vec::new() }
    }

    /// 解析 `WAT` 文本。
    pub fn parse(source: &str) -> Result<Self, WatError> {
        Parser::parse(source)
    }

    /// 追加一个顶层字段。
    pub fn push_field(&mut self, field: impl Into<String>) {
        self.fields.push(field.into());
    }
}
