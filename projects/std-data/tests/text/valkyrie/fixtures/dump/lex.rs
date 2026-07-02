use std_data::text::valkyrie::lexer::{Keyword, Lexer, Token, TokenKind};

use super::escape::escape_snapshot_text;

/// 将 lossless token 流格式化为稳定的 `*.lex` 文本快照。
pub fn dump_lex_snapshot(source: &str) -> String {
    let tokens = Lexer::tokenize_lossless(source).expect("tokenize_lossless");
    let mut lines = Vec::with_capacity(tokens.len());
    for token in tokens {
        let text = source[token.span.clone()].to_string();
        lines.push(format!(
            "Token {{ kind: {}, span: {}..{}, text: {} }}",
            token_kind_name(&token.kind),
            token.span.start,
            token.span.end,
            escape_snapshot_text(&text)
        ));
    }
    lines.join("\n")
}

fn token_kind_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Identifier => "Identifier",
        TokenKind::StringLiteral => "StringLiteral",
        TokenKind::IntegerLiteral => "IntegerLiteral",
        TokenKind::FloatLiteral => "FloatLiteral",
        TokenKind::LParen => "LParen",
        TokenKind::RParen => "RParen",
        TokenKind::LBrace => "LBrace",
        TokenKind::RBrace => "RBrace",
        TokenKind::LBracket => "LBracket",
        TokenKind::RBracket => "RBracket",
        TokenKind::LOffsetBracket => "LOffsetBracket",
        TokenKind::ROffsetBracket => "ROffsetBracket",
        TokenKind::LAngle => "LAngle",
        TokenKind::RAngle => "RAngle",
        TokenKind::Comma => "Comma",
        TokenKind::Colon => "Colon",
        TokenKind::Apostrophe => "Apostrophe",
        TokenKind::Semicolon => "Semicolon",
        TokenKind::Equal => "Equal",
        TokenKind::FatArrow => "FatArrow",
        TokenKind::Arrow => "Arrow",
        TokenKind::LArrow => "LArrow",
        TokenKind::DoubleColon => "DoubleColon",
        TokenKind::Dot => "Dot",
        TokenKind::DotDot => "DotDot",
        TokenKind::DotDotEq => "DotDotEq",
        TokenKind::DotDotLt => "DotDotLt",
        TokenKind::Ellipsis => "Ellipsis",
        TokenKind::PlusEqual => "PlusEqual",
        TokenKind::MinusEqual => "MinusEqual",
        TokenKind::StarEqual => "StarEqual",
        TokenKind::SlashEqual => "SlashEqual",
        TokenKind::PercentEqual => "PercentEqual",
        TokenKind::Plus => "Plus",
        TokenKind::Minus => "Minus",
        TokenKind::Star => "Star",
        TokenKind::Slash => "Slash",
        TokenKind::Percent => "Percent",
        TokenKind::Bang => "Bang",
        TokenKind::AndAnd => "AndAnd",
        TokenKind::OrOr => "OrOr",
        TokenKind::EqEq => "EqEq",
        TokenKind::NotEq => "NotEq",
        TokenKind::LessEq => "LessEq",
        TokenKind::GreaterEq => "GreaterEq",
        TokenKind::BacktickSymbol => "BacktickSymbol",
        TokenKind::Question => "Question",
        TokenKind::QuestionDot => "QuestionDot",
        TokenKind::Pipe => "Pipe",
        TokenKind::PipeGt => "PipeGt",
        TokenKind::Ampersand => "Ampersand",
        TokenKind::Tilde => "Tilde",
        TokenKind::Caret => "Caret",
        TokenKind::Shl => "Shl",
        TokenKind::Shr => "Shr",
        TokenKind::At => "At",
        TokenKind::HollowDiamond => "HollowDiamond",
        TokenKind::SolidDiamond => "SolidDiamond",
        TokenKind::LineComment => "LineComment",
        TokenKind::Whitespace => "Whitespace",
        TokenKind::TemplateDirective => "TemplateDirective",
        TokenKind::Eof => "Eof",
        TokenKind::Keyword(keyword) => keyword_kind_name(*keyword),
    }
}

fn keyword_kind_name(keyword: Keyword) -> &'static str {
    match keyword {
        Keyword::Namespace => "KeywordNamespace",
        Keyword::Using => "KeywordUsing",
        Keyword::Micro => "KeywordMicro",
        Keyword::Class => "KeywordClass",
        Keyword::Structure => "KeywordStructure",
        Keyword::Trait => "KeywordTrait",
        Keyword::Imply => "KeywordImply",
        Keyword::Unite => "KeywordUnite",
        Keyword::Type => "KeywordType",
        Keyword::Const => "KeywordConst",
        Keyword::Mut => "KeywordMut",
        Keyword::Ref => "KeywordRef",
        Keyword::Own => "KeywordOwn",
        Keyword::Where => "KeywordWhere",
        Keyword::In => "KeywordIn",
        Keyword::As => "KeywordAs",
        Keyword::True => "KeywordTrue",
        Keyword::False => "KeywordFalse",
        Keyword::Return => "KeywordReturn",
        Keyword::Break => "KeywordBreak",
        Keyword::Continue => "KeywordContinue",
        Keyword::Yield => "KeywordYield",
        Keyword::Raise => "KeywordRaise",
        Keyword::Resume => "KeywordResume",
        Keyword::Catch => "KeywordCatch",
        Keyword::If => "KeywordIf",
        Keyword::Else => "KeywordElse",
        Keyword::Loop => "KeywordLoop",
        Keyword::While => "KeywordWhile",
        Keyword::Match => "KeywordMatch",
        Keyword::Case => "KeywordCase",
        Keyword::Fallthrough => "KeywordFallthrough",
        Keyword::Let => "KeywordLet",
        Keyword::Lazy => "KeywordLazy",
        Keyword::Null => "KeywordNull",
        Keyword::Widget => "KeywordWidget",
        Keyword::Singleton => "KeywordSingleton",
        Keyword::Mezzo => "KeywordMezzo",
        Keyword::Macro => "KeywordMacro",
        Keyword::Neural => "KeywordNeural",
        Keyword::Flags => "KeywordFlags",
        Keyword::Union => "KeywordUnion",
        Keyword::Enums => "KeywordEnums",
        Keyword::Until => "KeywordUntil",
        Keyword::Try => "KeywordTry",
        Keyword::When => "KeywordWhen",
        Keyword::Assert => "KeywordAssert",
        Keyword::Scope => "KeywordScope",
        Keyword::Tests => "KeywordTests",
        Keyword::Constructor => "KeywordConstructor",
        Keyword::Not => "KeywordNot",
        Keyword::Is => "KeywordIs",
        Keyword::Nil => "KeywordNil",
        Keyword::KwSelf => "KeywordKwSelf",
        Keyword::KwSelfType => "KeywordKwSelfType",
        Keyword::KwSome => "KeywordKwSome",
        Keyword::KwNone => "KeywordKwNone",
    }
}

#[allow(dead_code)]
fn token_text<'a>(source: &'a str, token: &Token) -> &'a str {
    &source[token.span.clone()]
}
