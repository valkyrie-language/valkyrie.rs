//! Hermes lexer for the minimal schema / query subset.

/// Token kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// Identifier or keyword text (resolved later).
    Ident,
    /// Integer literal.
    Integer,
    /// String literal (`"..."`).
    String,
    /// `@@ident` primary-key field marker (lexeme is field name).
    AtAtIdent,
    /// `@ident` unique-key field marker (lexeme is field name).
    AtIdent,
    /// `$ident` / `@ident` param in query context — stored as Param with full lexeme.
    Param,
    /// `:`
    Colon,
    /// `,`
    Comma,
    /// `;`
    Semicolon,
    /// `=`
    Equal,
    /// `*`
    Star,
    /// `{`
    LeftBrace,
    /// `}`
    RightBrace,
    /// `<`
    LeftAngle,
    /// `>`
    RightAngle,
    /// End of input.
    Eof,
}

/// Source token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// Kind.
    pub kind: TokenKind,
    /// Byte offset of lexeme start.
    pub start: usize,
    /// Byte offset after lexeme.
    pub end: usize,
}

impl Token {
    /// Lexeme slice from source.
    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

/// Tokenize Hermes source.
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut lexer = Lexer { source, offset: 0, tokens: Vec::new() };
    lexer.run();
    lexer.tokens
}

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    fn run(&mut self) {
        while !self.is_at_end() {
            self.skip_trivia();
            if self.is_at_end() {
                break;
            }
            let start = self.offset;
            let ch = self.peek_char().unwrap();
            match ch {
                ':' => {
                    self.advance_char();
                    self.push(TokenKind::Colon, start);
                }
                ',' => {
                    self.advance_char();
                    self.push(TokenKind::Comma, start);
                }
                ';' => {
                    self.advance_char();
                    self.push(TokenKind::Semicolon, start);
                }
                '=' => {
                    self.advance_char();
                    self.push(TokenKind::Equal, start);
                }
                '*' => {
                    self.advance_char();
                    self.push(TokenKind::Star, start);
                }
                '{' => {
                    self.advance_char();
                    self.push(TokenKind::LeftBrace, start);
                }
                '}' => {
                    self.advance_char();
                    self.push(TokenKind::RightBrace, start);
                }
                '<' => {
                    self.advance_char();
                    self.push(TokenKind::LeftAngle, start);
                }
                '>' => {
                    self.advance_char();
                    self.push(TokenKind::RightAngle, start);
                }
                '"' => self.read_string(start),
                '$' => self.read_param(start),
                '@' => self.read_at(start),
                c if c.is_ascii_digit() => self.read_integer(start),
                c if is_ident_start(c) => self.read_ident(start),
                _ => {
                    // Skip unknown single char so parse can report a useful error later.
                    self.advance_char();
                }
            }
        }
        self.tokens.push(Token { kind: TokenKind::Eof, start: self.offset, end: self.offset });
    }

    fn skip_trivia(&mut self) {
        loop {
            self.skip_spaces();
            if self.peek_char() == Some('/') && self.peek_char_at(1) == Some('/') {
                while let Some(c) = self.peek_char() {
                    if c == '\n' {
                        break;
                    }
                    self.advance_char();
                }
                continue;
            }
            if self.peek_char() == Some('#') {
                while let Some(c) = self.peek_char() {
                    if c == '\n' {
                        break;
                    }
                    self.advance_char();
                }
                continue;
            }
            break;
        }
    }

    fn skip_spaces(&mut self) {
        while matches!(self.peek_char(), Some(c) if c.is_whitespace()) {
            self.advance_char();
        }
    }

    fn read_string(&mut self, start: usize) {
        self.advance_char(); // "
        while let Some(c) = self.peek_char() {
            if c == '"' {
                self.advance_char();
                break;
            }
            if c == '\\' {
                self.advance_char();
                self.advance_char();
                continue;
            }
            self.advance_char();
        }
        self.push(TokenKind::String, start);
    }

    fn read_param(&mut self, start: usize) {
        self.advance_char(); // $
        while matches!(self.peek_char(), Some(c) if is_ident_continue(c)) {
            self.advance_char();
        }
        self.push(TokenKind::Param, start);
    }

    fn read_at(&mut self, start: usize) {
        self.advance_char(); // @
        if self.peek_char() == Some('@') {
            self.advance_char();
            let name_start = self.offset;
            while matches!(self.peek_char(), Some(c) if is_ident_continue(c)) {
                self.advance_char();
            }
            // Token span covers only the identifier after @@ (CS primary-key field name).
            self.tokens.push(Token { kind: TokenKind::AtAtIdent, start: name_start, end: self.offset });
            let _ = start;
            return;
        }
        // Could be @ident (unique field) or leftover — treat as AtIdent if followed by ident.
        if matches!(self.peek_char(), Some(c) if is_ident_start(c)) {
            let name_start = self.offset;
            while matches!(self.peek_char(), Some(c) if is_ident_continue(c)) {
                self.advance_char();
            }
            self.tokens.push(Token { kind: TokenKind::AtIdent, start: name_start, end: self.offset });
            let _ = start;
            return;
        }
        // Lone `@` — emit as Param-like text for query `@id`
        self.tokens.push(Token { kind: TokenKind::Param, start, end: self.offset });
    }

    fn read_integer(&mut self, start: usize) {
        while matches!(self.peek_char(), Some(c) if c.is_ascii_digit()) {
            self.advance_char();
        }
        self.push(TokenKind::Integer, start);
    }

    fn read_ident(&mut self, start: usize) {
        while matches!(self.peek_char(), Some(c) if is_ident_continue(c)) {
            self.advance_char();
        }
        self.push(TokenKind::Ident, start);
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        self.tokens.push(Token { kind, start, end: self.offset });
    }

    fn is_at_end(&self) -> bool {
        self.offset >= self.source.len()
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn peek_char_at(&self, n: usize) -> Option<char> {
        self.source[self.offset..].chars().nth(n)
    }

    fn advance_char(&mut self) {
        if let Some(c) = self.peek_char() {
            self.offset += c.len_utf8();
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_primary_unique_markers() {
        let source = "@@id @email name";
        let tokens = tokenize(source);
        assert_eq!(tokens[0].kind, TokenKind::AtAtIdent);
        assert_eq!(tokens[0].lexeme(source), "id");
        assert_eq!(tokens[1].kind, TokenKind::AtIdent);
        assert_eq!(tokens[1].lexeme(source), "email");
        assert_eq!(tokens[2].kind, TokenKind::Ident);
        assert_eq!(tokens[2].lexeme(source), "name");
    }
}
