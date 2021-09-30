use std::collections::BTreeMap;

use crate::{error::VonParseError, value::VonValue};

pub struct VonParser<'a> {
    chars: Vec<char>,
    position: usize,
    _source: &'a str,
}

impl<'a> VonParser<'a> {
    pub fn parse(source: &'a str) -> Result<VonValue, VonParseError> {
        let mut parser = Self { chars: source.chars().collect(), position: 0, _source: source };
        let value = parser.parse_value()?;
        parser.skip_trivia();
        if parser.peek().is_some() {
            return Err(parser.error("unexpected trailing input"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<VonValue, VonParseError> {
        self.skip_trivia();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(VonValue::String),
            Some(ch) if ch == '-' || ch.is_ascii_digit() => self.parse_number().map(VonValue::Number),
            Some(_) => self.parse_identifier_value(),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_object(&mut self) -> Result<VonValue, VonParseError> {
        self.expect('{')?;
        let mut members = BTreeMap::new();
        loop {
            self.skip_trivia();
            if self.consume_if('}') {
                break;
            }

            let key = self.parse_key()?;
            self.skip_trivia();
            self.expect(':')?;
            let value = self.parse_value()?;
            members.insert(key, value);

            self.skip_trivia();
            if self.consume_if(',') {
                continue;
            }
            if self.consume_if('}') {
                break;
            }
            return Err(self.error("expected ',' or '}'"));
        }
        Ok(VonValue::Object(members))
    }

    fn parse_array(&mut self) -> Result<VonValue, VonParseError> {
        self.expect('[')?;
        let mut items = Vec::new();
        loop {
            self.skip_trivia();
            if self.consume_if(']') {
                break;
            }
            items.push(self.parse_value()?);
            self.skip_trivia();
            if self.consume_if(',') {
                continue;
            }
            if self.consume_if(']') {
                break;
            }
            return Err(self.error("expected ',' or ']'"));
        }
        Ok(VonValue::Array(items))
    }

    fn parse_key(&mut self) -> Result<String, VonParseError> {
        self.skip_trivia();
        match self.peek() {
            Some('"') => self.parse_string(),
            Some(ch) if is_identifier_start(ch) => self.parse_identifier(),
            Some(_) => Err(self.error("expected object key")),
            None => Err(self.error("unexpected end of input")),
        }
    }

    fn parse_identifier_value(&mut self) -> Result<VonValue, VonParseError> {
        let identifier = self.parse_identifier()?;
        match identifier.as_str() {
            "null" => Ok(VonValue::Null),
            "true" => Ok(VonValue::Bool(true)),
            "false" => Ok(VonValue::Bool(false)),
            _ => Ok(VonValue::String(identifier)),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, VonParseError> {
        self.skip_trivia();
        let mut buffer = String::new();
        let Some(first) = self.peek()
        else {
            return Err(self.error("unexpected end of input"));
        };
        if !is_identifier_start(first) {
            return Err(self.error("expected identifier"));
        }
        buffer.push(first);
        self.position += 1;
        while let Some(ch) = self.peek() {
            if is_identifier_continue(ch) {
                buffer.push(ch);
                self.position += 1;
            }
            else {
                break;
            }
        }
        Ok(buffer)
    }

    fn parse_string(&mut self) -> Result<String, VonParseError> {
        self.expect('"')?;
        let mut buffer = String::new();
        loop {
            match self.next() {
                Some('"') => break,
                Some('\\') => {
                    let escaped = self.next().ok_or_else(|| self.error("unterminated string escape"))?;
                    match escaped {
                        '"' => buffer.push('"'),
                        '\\' => buffer.push('\\'),
                        'n' => buffer.push('\n'),
                        'r' => buffer.push('\r'),
                        't' => buffer.push('\t'),
                        other => buffer.push(other),
                    }
                }
                Some(ch) => buffer.push(ch),
                None => return Err(self.error("unterminated string")),
            }
        }
        Ok(buffer)
    }

    fn parse_number(&mut self) -> Result<i64, VonParseError> {
        self.skip_trivia();
        let start = self.position;
        if self.consume_if('-') && self.peek().is_none_or(|ch| !ch.is_ascii_digit()) {
            return Err(self.error("expected digit after '-'"));
        }
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.position += 1;
        }
        self.chars[start..self.position].iter().collect::<String>().parse::<i64>().map_err(|_| self.error("invalid integer"))
    }

    fn skip_trivia(&mut self) {
        loop {
            while self.peek().is_some_and(|ch| ch.is_whitespace()) {
                self.position += 1;
            }

            if self.peek() == Some('#') {
                while self.peek().is_some_and(|ch| ch != '\n') {
                    self.position += 1;
                }
                continue;
            }

            break;
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), VonParseError> {
        match self.next() {
            Some(ch) if ch == expected => Ok(()),
            Some(_) => Err(self.error(format!("expected '{}'", expected))),
            None => Err(self.error(format!("expected '{}'", expected))),
        }
    }

    fn consume_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.position += 1;
            true
        }
        else {
            false
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn next(&mut self) -> Option<char> {
        let value = self.peek()?;
        self.position += 1;
        Some(value)
    }

    fn error(&self, message: impl Into<String>) -> VonParseError {
        VonParseError::new(self.position, message)
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '@'
}
