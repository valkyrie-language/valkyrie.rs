#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

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
        let source = source.trim();
        if !source.starts_with('(') || !source.ends_with(')') {
            return Err(WatError::InvalidModule("顶层必须是 `(module ...)`".to_string()));
        }

        let mut cursor = Cursor::new(source);
        cursor.expect_char('(')?;
        cursor.skip_ws();
        let keyword = cursor.read_token()?;
        if keyword != "module" {
            return Err(WatError::InvalidModule("顶层节点必须是 `module`".to_string()));
        }

        cursor.skip_ws();
        let module_name = if cursor.peek_char() == Some('$') { Some(cursor.read_token()?) } else { None };

        let fields = cursor.read_top_level_fields()?;
        cursor.skip_ws();
        cursor.expect_char(')')?;
        cursor.skip_ws();
        if !cursor.is_eof() {
            return Err(WatError::InvalidModule("模块结束后仍有多余内容".to_string()));
        }

        Ok(Self { module_name, fields })
    }

    /// 追加一个顶层字段。
    pub fn push_field(&mut self, field: impl Into<String>) {
        self.fields.push(field.into());
    }

    /// 将模型格式化为 `WAT` 文本。
    pub fn to_text(&self) -> String {
        let mut result = String::from("(module");
        if let Some(module_name) = &self.module_name {
            result.push(' ');
            result.push_str(module_name);
        }

        if self.fields.is_empty() {
            result.push(')');
            return result;
        }

        for field in &self.fields {
            result.push('\n');
            result.push_str("  ");
            result.push_str(field.trim());
        }
        result.push('\n');
        result.push(')');
        result
    }
}

struct Cursor<'a> {
    source: &'a str,
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, offset: 0 }
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn expect_char(&mut self, expected: char) -> Result<(), WatError> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(WatError::InvalidModule(format!("缺少字符 `{expected}`"))),
        }
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.next_char();
            }
            else {
                break;
            }
        }
    }

    fn read_token(&mut self) -> Result<String, WatError> {
        let mut token = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            token.push(ch);
            self.next_char();
        }
        if token.is_empty() {
            return Err(WatError::InvalidModule("期望读取标记，但遇到空内容".to_string()));
        }
        Ok(token)
    }

    fn read_top_level_fields(&mut self) -> Result<Vec<String>, WatError> {
        let mut fields = Vec::new();
        let mut depth = 0usize;
        let mut field_start = None;
        let mut in_string = false;
        let mut escape = false;

        while let Some(ch) = self.peek_char() {
            if depth == 0 && ch == ')' {
                break;
            }
            self.next_char();
            match ch {
                '"' if !escape => {
                    in_string = !in_string;
                }
                '\\' if in_string => {
                    escape = !escape;
                    continue;
                }
                '(' if !in_string => {
                    if depth == 0 {
                        field_start = Some(self.offset - 1);
                    }
                    depth += 1;
                }
                ')' if !in_string => {
                    if depth == 0 {
                        return Err(WatError::UnbalancedParentheses);
                    }
                    depth -= 1;
                    if depth == 0 {
                        let start = field_start.ok_or(WatError::UnbalancedParentheses)?;
                        fields.push(self.source[start..self.offset].trim().to_string());
                        field_start = None;
                    }
                }
                _ => {
                    escape = false;
                }
            }
        }

        if in_string {
            return Err(WatError::UnterminatedString);
        }
        if depth != 0 {
            return Err(WatError::UnbalancedParentheses);
        }
        Ok(fields)
    }
}
