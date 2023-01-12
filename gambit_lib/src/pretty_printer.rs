// use std::cmp::max;
use std::io::ErrorKind;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum PrinterError {
    #[error("IO error: {kind}")]
    IOError { kind: ErrorKind },
}

pub struct PrettyPrinter {
    pub row: usize,
    pub column: usize,
    pub tab_width: usize,
    pub page_width: usize,
    pub indent: usize,
    pub newline: String,
}

impl PrettyPrinter {
    pub fn new(tab_width: usize, page_width: usize, newline: &str) -> PrettyPrinter {
        PrettyPrinter {
            row: 1,
            column: 1,
            tab_width,
            page_width,
            indent: 0,
            newline: String::from(newline),
        }
    }

    pub fn increase_indent(&mut self) {
        let max_indent = (self.page_width as f64 / self.indent as f64) - 1.0;
        if self.indent >= max_indent as usize {
            self.indent = max_indent as usize;
        } else {
            self.indent += 1;
        }
    }

    pub fn decrease_indent(&mut self) {
        if self.indent == 0 {
            return;
        }
        self.indent -= 1;
    }

    fn indent_length(&self) -> usize {
        self.indent * self.tab_width
    }

    pub fn indent_string(&self) -> String {
        // There is probably a String library method that can do this in one function call.
        let mut s = String::new();
        let size = self.indent_length();
        let mut i = 0;
        while i < size {
            s += " ";
            i += 1;
        }
        s
    }

    pub fn write_indent<W: Write>(&mut self, stream: &mut W) -> Result<(), PrinterError> {
        let indention = self.indent_string();
        if indention.len() > 0 {
            if let Err(e) = self.write_basic_string(stream, &indention) {
                return Err(e);
            }
        }
        Ok(())
    }

    pub fn write_newline<W: Write>(&mut self, stream: &mut W) -> Result<(), PrinterError> {
        if let Err(e) = write!(stream, "{}", self.newline) {
            return Err(PrinterError::IOError { kind: e.kind() });
        }
        self.row += 1;
        self.column = 1;
        Ok(())
    }

    pub fn write_space<W: Write>(&mut self, stream: &mut W) -> Result<(), PrinterError> {
        if self.column == self.page_width {
            self.write_newline(stream)?;
            self.write_indent(stream)?;
        }
        self.write_basic_string(stream, " ")?;
        Ok(())
    }

    pub fn write_token<W: Write>(
        &mut self,
        stream: &mut W,
        token: &str,
    ) -> Result<(), PrinterError> {
        if self.column > self.page_width {
            // We have overreached on a previous write. Go to a newline.
            self.write_newline(stream)?;
        }

        if token.len() > self.page_width {
            // This token will not fit in the current page width at all.  So, we
            // print the indentation and let the token spill over the page width.
            if self.column > self.indent_length() {
                // We are already past the last supported indention
                self.write_newline(stream)?;
                self.write_indent(stream)?;
                self.write_basic_string(stream, token)?;
            } else {
                self.write_basic_string(stream, token)?;
            }
        } else {
            if token.len() > (self.page_width - self.column) {
                self.write_newline(stream)?;
                self.write_indent(stream)?;
            }
            self.write_basic_string(stream, token)?;
        }
        Ok(())
    }

    pub fn write_string<W: Write>(&mut self, stream: &mut W, s: &str) -> Result<(), PrinterError> {
        let composed_string = String::from("\"") + s + "\"";
        self.write_token(stream, &composed_string)?;
        Ok(())
    }

    pub fn write_flowable_text<W: Write>(
        &mut self,
        stream: &mut W,
        s: &str,
        next_line_text: &str,
    ) -> Result<(), PrinterError> {
        // flowable text is a piece of text that can be separated in the output stream without
        // altering the meaning of the program.
        if s.len() > (self.page_width - self.column) {
            let space_left = self.page_width - self.column;
            if space_left == 0 {
                self.write_newline(stream)?;
                self.write_indent(stream)?;
                self.write_token(stream, next_line_text)?;
                self.write_flowable_text(stream, s, next_line_text)?;
            } else {
                let first_part = &s[..space_left];
                if first_part.len() > 0 {
                    self.write_basic_string(stream, first_part)?;
                }
                self.write_newline(stream)?;
                self.write_indent(stream)?;
                self.write_token(stream, next_line_text)?;

                let rest = &s[space_left..];
                if rest.len() > 0 {
                    self.write_flowable_text(stream, rest, next_line_text)?;
                }
            }
        } else {
            self.write_basic_string(stream, s)?;
        }

        Ok(())
    }

    fn write_basic_string<W: Write>(
        &mut self,
        stream: &mut W,
        s: &str,
    ) -> Result<(), PrinterError> {
        if let Err(e) = write!(stream, "{s}") {
            return Err(PrinterError::IOError { kind: e.kind() });
        }
        self.column += s.len();
        Ok(())
    }
}

pub fn write_indent<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_indent(stream) {
        log::info!("Unable to write indentation: {e}");
    }
}

pub fn write_space<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_space(stream) {
        log::info!("Unable to write space character: {e}");
    }
}

pub fn write_newline<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_newline(stream) {
        log::info!("Unable to write newline: {e}");
    }
}

pub fn write_token<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, token: &str) {
    if let Err(e) = printer.write_token(stream, token) {
        log::info!("Unable to write token: {e}");
    }
}

pub fn write_string<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, s: &str) {
    if let Err(e) = printer.write_string(stream, s) {
        log::info!("Unable to write string: {e}");
    }
}

pub fn write_flowable_text<W: Write>(
    printer: &mut PrettyPrinter,
    stream: &mut W,
    s: &str,
    next_line_text: &str,
) {
    if let Err(e) = printer.write_flowable_text(stream, s, next_line_text) {
        log::info!("Unable to write punctuation: {e}");
    }
}
