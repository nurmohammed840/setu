use std::fmt::{Arguments, Display, Write};

#[derive(Clone)]
pub struct CodeWriter {
    pub buffer: String,
    indent: &'static str,
    indent_level: u16,
}

impl CodeWriter {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            indent: "    ",
            indent_level: 0,
        }
    }

    pub fn indent(&mut self) {
        self.indent_level += 1;
    }

    pub fn dedent(&mut self) {
        debug_assert!(self.indent_level > 0);
        self.indent_level -= 1;
    }

    pub fn write_line(&mut self, s: &str) {
        self.write_indent();
        self.buffer.push_str(s);
        self.newline();
    }

    pub fn line(&mut self, args: Arguments<'_>) {
        self.write_indent();
        let _ = self.buffer.write_fmt(args);
        self.newline();
    }

    pub fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.buffer.push_str(self.indent);
        }
    }

    pub fn newline(&mut self) {
        self.buffer.push('\n');
    }

    pub fn block(&mut self, header: impl Display, f: impl FnOnce(&mut Self)) {
        self.line(format_args!("{header} {{"));
        self.scope(f);
        self.write_line("}");
    }

    pub fn scope(&mut self, f: impl FnOnce(&mut Self)) {
        self.indent();
        f(self);
        self.dedent();
    }
}
