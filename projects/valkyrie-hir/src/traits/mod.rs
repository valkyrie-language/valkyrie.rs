
use std::fmt::Write;
use valkyrie_types::Variable;

pub trait IndentFormat {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>)
                               -> std::fmt::Result;
    fn indent_display(&self) -> std::fmt::Result {
        let buffer = String::new();
        let mut context = IndentContext::new("    ".to_string(), buffer);
        self.indent_format(&mut context)?;
        println!("{}", context.writer);
        Ok(())
    }
}

pub struct IndentContext<W> {
    writer: W,
    indent: String,
    level: usize,
}

impl<W: Write> IndentContext<W> {
    pub fn new(indent: String, buffer: W) -> Self {
        Self {
            writer: buffer,
            level: 0,
            indent: indent.to_string(),
        }
    }

    pub fn add_text(&mut self, text: &str) -> std::fmt::Result {
        self.writer.write_str(text)
    }
    pub fn add_char(&mut self, char: char) -> std::fmt::Result {
        self.writer.write_char(char)
    }

    pub fn new_line(&mut self) -> std::fmt::Result {
        self.writer.write_char('\n')?;
        for _ in 0..self.level {
            self.writer.write_str(&self.indent)?;
        }
        Ok(())
    }

    pub fn indent(&mut self, lead: &str) -> std::fmt::Result {
        self.add_text(lead)?;
        self.level += 1;
        self.new_line()
    }

    pub fn dedent(&mut self, tail: &str) -> std::fmt::Result {
        self.level -= 1;
        self.new_line()?;
        self.add_text(tail)?;
        self.new_line()
    }

    pub fn finish(self) -> W {
        self.writer
    }
}
impl<'a> IndentFormat for &'a str {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.add_text(self)
    }
}
impl<'a> IndentFormat for i64 {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.add_text(&self.to_string())
    }
}

impl IndentFormat for Variable {
    fn indent_format<W: Write>(&self, context: &mut IndentContext<W>) -> std::fmt::Result {
        context.add_text(&self.to_string())
    }
}