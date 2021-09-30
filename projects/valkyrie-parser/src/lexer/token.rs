//! 词法记号定义。

use std::ops::Range;

/// 词法记号类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// 标识符或关键字。
    Identifier,
    /// 字符串字面量。
    StringLiteral,
    /// 整数字面量。
    IntegerLiteral,
    /// 浮点数字面量。
    FloatLiteral,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `{`
    LBrace,
    /// `}`
    RBrace,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// `<`
    LAngle,
    /// `>`
    RAngle,
    /// `,`
    Comma,
    /// `:`
    Colon,
    /// `;`
    Semicolon,
    /// `=`
    Equal,
    /// `->`
    Arrow,
    /// `::`
    DoubleColon,
    /// `.`
    Dot,
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `%`
    Percent,
    /// `!`
    Bang,
    /// `&&`
    AndAnd,
    /// `||`
    OrOr,
    /// `==`
    EqEq,
    /// `!=`
    NotEq,
    /// `<=`
    LessEq,
    /// `>=`
    GreaterEq,
    /// 反引号包裹的符号字面量（如 `` `+=` ``、`` `!` ``）。
    ///
    /// 用于运算符重载定义，文本通过 `span` 从源码中提取。
    BacktickSymbol,
    /// `?`
    Question,
    /// `?.`
    QuestionDot,
    /// `|`
    Pipe,
    /// `|>`，管道操作符，将左侧表达式的值作为右侧函数的参数。
    PipeGt,
    /// `&`
    Ampersand,
    /// `~`，按位取反运算符。
    Tilde,
    /// `^`，按位异或运算符。
    Caret,
    /// `<<`，左移位运算符。
    Shl,
    /// `>>`，右移位运算符。
    Shr,
    /// `@`，编译器指令前缀。
    At,
    /// 文件结束。
    Eof,
}

/// 词法记号。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// 记号类型。
    pub kind: TokenKind,
    /// 在源文本中的字节范围。
    pub span: Range<usize>,
}
