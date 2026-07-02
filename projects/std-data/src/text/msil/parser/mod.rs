use super::lexer::{Lexer, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsilTextMethod {
    pub signature: String,
    pub name: String,
    pub start_line: usize,
    pub body: Vec<String>,
}

pub struct MsilParser;

impl MsilParser {
    pub fn parse_methods(source: &str) -> Vec<MsilTextMethod> {
        Parser::parse(source)
    }
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<super::lexer::Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn parse(source: &'a str) -> Vec<MsilTextMethod> {
        let tokens = Lexer::tokenize(source);
        if tokens.is_empty() {
            return Vec::new();
        }
        let mut parser = Self { source, tokens, index: 0 };
        parser.parse_methods()
    }

    fn parse_methods(&mut self) -> Vec<MsilTextMethod> {
        let mut result = Vec::new();
        while !self.is_at_end() {
            let token = self.peek();
            if token.kind != TokenKind::Line {
                self.advance();
                continue;
            }
            let line = self.line_text(&token);
            if !line.trim_start().starts_with(".method") {
                self.advance();
                continue;
            }
            let start_line = token.line_number;
            let signature = extract_signature(line.trim_start());
            let (end_index, balanced) = self.find_block_end();
            let end_index = if balanced { end_index } else { self.tokens.len().saturating_sub(2) };
            let body = (self.index..=end_index)
                .filter_map(|idx| {
                    let token = &self.tokens[idx];
                    if token.kind == TokenKind::Line { Some(self.line_text(token).to_string()) } else { None }
                })
                .collect::<Vec<_>>();
            result.push(MsilTextMethod { name: extract_method_name(&signature), signature, start_line, body });
            self.index = end_index + 1;
        }
        result
    }

    fn find_block_end(&self) -> (usize, bool) {
        let mut depth: i32 = 0;
        let mut seen_open = false;
        for idx in self.index..self.tokens.len() {
            let token = &self.tokens[idx];
            if token.kind != TokenKind::Line {
                continue;
            }
            for ch in self.line_text(token).chars() {
                if ch == '{' {
                    depth += 1;
                    seen_open = true;
                }
                else if ch == '}' {
                    depth -= 1;
                }
                if seen_open && depth == 0 {
                    return (idx, true);
                }
            }
        }
        (self.tokens.len().saturating_sub(2), false)
    }

    fn line_text(&self, token: &super::lexer::Token) -> &str {
        &self.source[token.span.clone()]
    }

    fn peek(&self) -> super::lexer::Token {
        self.tokens.get(self.index).cloned().unwrap_or_else(|| super::lexer::Token::eof(self.source.len(), 0))
    }

    fn advance(&mut self) -> super::lexer::Token {
        let token = self.peek();
        if !self.is_at_end() {
            self.index += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

fn extract_signature(method_line: &str) -> String {
    let sig = method_line[".method".len()..].trim();
    let brace_idx = sig.find('{');
    match brace_idx {
        Some(idx) => sig[..idx].trim(),
        None => sig.trim(),
    }
    .trim_end()
    .to_string()
}

fn extract_method_name(signature: &str) -> String {
    let paren_idx = signature.find('(');
    let head = match paren_idx {
        Some(idx) => signature[..idx].trim(),
        None => signature.trim(),
    };

    let tokens: Vec<&str> = head.split_whitespace().collect();
    match tokens.last() {
        Some(last) => last.to_string(),
        None => signature.to_string(),
    }
}
