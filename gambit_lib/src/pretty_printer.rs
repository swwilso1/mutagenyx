//! The `pretty_printer` module contains a low-level stream 'token' emitter to use when
//! reconstructing source code from an AST.

use crate::error::GambitError;
use std::io::Write;

/// Object that encapsulates the behavior needed to write structured output to any object that
/// implements the [`Write`] trait.
pub struct PrettyPrinter {
    pub row: usize,
    pub column: usize,
    pub tab_width: usize,
    pub page_width: usize,
    pub indent: usize,
    pub newline: String,
}

impl PrettyPrinter {
    /// Initialize a new pretty-printer object
    ///
    /// # Arguments
    ///
    /// * `tab_width` - The number of spaces to use for a tab character.
    /// * `page_width` - The number of spaces to use for the column width of the document.  If the
    /// column-width is small, the pretty-printer may overflow the `page_width` in order to prevent
    /// introducing line breaks inside of tokens that exceed the length of `page_width`.
    /// * `newline` - The string slice to use for newline characters.  For unix-like platforms, use
    /// "\n".  For Windows, use "\r\n".
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

    /// Increase the indentation level.
    ///
    /// When the printer emits the next newline character and starts a new-line, it will first print
    /// indentations based on the number of indentations recorded in the indentation level.
    pub fn increase_indent(&mut self) {
        let max_indent = (self.page_width as f64 / self.indent as f64) - 1.0;
        if self.indent >= max_indent as usize {
            self.indent = max_indent as usize;
        } else {
            self.indent += 1;
        }
    }

    /// Decrease the indentation level.
    pub fn decrease_indent(&mut self) {
        if self.indent == 0 {
            return;
        }
        self.indent -= 1;
    }

    /// Return the length of the current indentation in spaces.
    fn indent_length(&self) -> usize {
        self.indent * self.tab_width
    }

    /// Return the string slice that represents the space characters that make up the indentation
    /// prefix for a new line.
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

    /// Write out the indentation string to the `stream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - The object that implements the [`Write`] trait.
    pub fn write_indent<W: Write>(&mut self, stream: &mut W) -> Result<(), GambitError> {
        let indention = self.indent_string();
        if indention.len() > 0 {
            if let Err(e) = self.write_basic_string(stream, &indention) {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Write newline characters to the `stream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - The object that implements the [`Write`] trait.
    pub fn write_newline<W: Write>(&mut self, stream: &mut W) -> Result<(), GambitError> {
        if let Err(e) = write!(stream, "{}", self.newline) {
            return Err(GambitError::from(e));
        }
        self.row += 1;
        self.column = 1;
        Ok(())
    }

    /// Write a space character to the `stream`.
    ///
    /// # Arguments
    ///
    /// * `stream` - The object that implements the [`Write`] trait.
    pub fn write_space<W: Write>(&mut self, stream: &mut W) -> Result<(), GambitError> {
        if self.column == self.page_width {
            self.write_newline(stream)?;
            self.write_indent(stream)?;
        }
        self.write_basic_string(stream, " ")?;
        Ok(())
    }

    /// Write `token` to `stream`
    ///
    /// # Arguments
    ///
    /// * `stream` - The [`Write`] object that receives the token.
    /// * `token` - The string slice to write to `stream`.
    pub fn write_token<W: Write>(
        &mut self,
        stream: &mut W,
        token: &str,
    ) -> Result<(), GambitError> {
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

    /// Write a string value to the stream.  The function will emit the string surrounded by the
    /// \" delimiters.
    ///
    /// # Arguments
    ///
    /// * `stream` - the [`Write`] object that will receive the string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// f.write_string(stream, "The quick brown dog...");
    /// ```
    ///
    /// will output:
    ///
    /// "The quick brown dog..."
    pub fn write_string<W: Write>(&mut self, stream: &mut W, s: &str) -> Result<(), GambitError> {
        let composed_string = String::from("\"") + s + "\"";
        self.write_token(stream, &composed_string)?;
        Ok(())
    }

    /// Write a string value to the stream.  The function will emit the string surrounded by three
    /// \" delimiters.
    ///
    /// # Arguments
    ///
    /// * `stream` - The [`Write`] object that will receive the string.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// f.write_triple_string(stream, "The quick brown dog...");
    /// ```
    ///
    /// will output:
    ///
    /// """The quick brown dog..."""
    pub fn write_triple_string<W: Write>(
        &mut self,
        stream: &mut W,
        s: &str,
    ) -> Result<(), GambitError> {
        let composed_string = String::from("\"\"\"") + s + "\"\"\"";
        self.write_token(stream, &composed_string)?;
        Ok(())
    }

    /// Write a string of text to `stream`.  The printer may break the string at any point it deems
    /// necessary and does not treat the text as an atomic token.
    ///
    /// # Arguments
    ///
    /// * `stream` - The [`Write`] object that will receive the text.
    /// * `s` - The text to write.
    /// * `next_line_text` - Sometimes, when the printer breaks a line of flowablew text and
    /// writes the remaining text on the next line, the context requires that the next line start
    /// with a particular set of content.  `next_line_text` contains the text that the printer should
    /// write in the event that it breaks the flowable text into mutliple lines.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// foo.write_flowable_text(stream,
    ///                         "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Cras fermentum hendrerit mi, sit amet finibus ante pulvinar eget.",
    ///                         " * ");
    /// ```
    ///
    /// Might produce the output:
    ///
    /// Lorem ipsum dolor sit amet,
    ///  * consectetur adipiscing elit.
    ///  * Cras fermentum hendrerit mi,
    ///  * sit amet finibus ante pulvinar
    ///  * eget.
    pub fn write_flowable_text<W: Write>(
        &mut self,
        stream: &mut W,
        s: &str,
        next_line_text: &str,
    ) -> Result<(), GambitError> {
        // flowable text is a piece of text that can be separated in the output stream without
        // altering the meaning of the program.

        // First remove newlines. Existing newlines break the flow of the text in an arbitrary
        // way and breaks the column accounting algorithms.  This is a naive implementation of
        // character removal.
        let mut text = String::from(s);
        let mut index = text.find("\n");
        while index != None {
            let actual_index = index.unwrap();
            text.remove(actual_index);
            index = text.find("\n");
        }

        // Extra spaces may also break the flow.  So we replace two or more spaces with one space
        // until we have no more double spaces.
        index = text.find("  ");
        while index != None {
            let actual_index = index.unwrap();
            text.remove(actual_index + 1);
            index = text.find("  ");
        }

        let t = text.as_str();

        if t.len() > (self.page_width - self.column) {
            let space_left = self.page_width - self.column;
            if space_left == 0 {
                self.write_newline(stream)?;
                self.write_indent(stream)?;
                self.write_token(stream, next_line_text)?;
                self.write_flowable_text(stream, t, next_line_text)?;
            } else {
                let first_part = &t[..space_left];
                if first_part.len() > 0 {
                    self.write_basic_string(stream, first_part)?;
                }
                self.write_newline(stream)?;
                self.write_indent(stream)?;
                self.write_token(stream, next_line_text)?;

                let rest = &t[space_left..];
                if rest.len() > 0 {
                    self.write_flowable_text(stream, rest, next_line_text)?;
                }
            }
        } else {
            self.write_basic_string(stream, t)?;
        }

        Ok(())
    }

    /// Low-level function to write a string to the stream.
    ///
    /// # Argument
    ///
    /// * `stream` - The [`Write`] object that will receive the text.
    /// * `s` - The string slice referring to the text to write to `stream`.
    fn write_basic_string<W: Write>(&mut self, stream: &mut W, s: &str) -> Result<(), GambitError> {
        if let Err(e) = write!(stream, "{s}") {
            return Err(GambitError::from(e));
        }
        self.column += s.len();
        Ok(())
    }
}

/// Helper function to write an indent to `stream` while suppressing any errors.  The function sends
/// errors to the log.
///
/// # Arguments
///
/// * `printer` - The pretty printer that will write the indent to `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
pub fn write_indent<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_indent(stream) {
        log::info!("Unable to write indentation: {e}");
    }
}

/// Helper function to write a space to `stream` while suppressing any errors.  The function sends
/// errors to the log.
///
/// # Arguments
///
/// * `printer` - The pretty-printer that will write the space to `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
pub fn write_space<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_space(stream) {
        log::info!("Unable to write space character: {e}");
    }
}

/// Helper function to write a newline to `stream` while suppressing any errors.  The function
/// sends errors to the log.
///
/// # Arguments
///
/// * `printer` - The pretty-printer that will write the newline to the `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
pub fn write_newline<W: Write>(printer: &mut PrettyPrinter, stream: &mut W) {
    if let Err(e) = printer.write_newline(stream) {
        log::info!("Unable to write newline: {e}");
    }
}

/// Helper function to write a token to `stream` while suppressing any errors.  The function
/// sends errors to the log.
///
/// # Arguments
///
/// * `printer` - The pretty-printer that will write the token to the `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
pub fn write_token<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, token: &str) {
    if let Err(e) = printer.write_token(stream, token) {
        log::info!("Unable to write token: {e}");
    }
}

/// Helper function to write a string to `stream` while suppressing any errors.  The function
/// sends errors to the log.
///
/// The pretty-printer will output the string as "`s`".
///
/// # Arguments
///
/// * `printer` - The pretty-printer that will write the token to the `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
/// * `s` - The string slice containing the text to send to `stream`.
pub fn write_string<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, s: &str) {
    if let Err(e) = printer.write_string(stream, s) {
        log::info!("Unable to write string: {e}");
    }
}

/// Helper function to write a string to `stream` while suppressing any errors. The function
/// sends errors to the log.
///
/// The pretty-printer will output the string as """`s`""".
///
/// # Arguments
///
/// * `printer` - The pretty-printer that will write the string to `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
/// * `s` - The string slice containing the text to send to `stream`.
pub fn write_triple_string<W: Write>(printer: &mut PrettyPrinter, stream: &mut W, s: &str) {
    if let Err(e) = printer.write_triple_string(stream, s) {
        log::info!("Unable to write string: {e}");
    }
}

/// Helper function to write flowable text to `stream` while suppressing any errors.  The
/// function sends errors to the log.
///
/// The flowable text has the same semantics as [`PrettyPrinter::write_flowable_text`]
///
/// # Arguments
///
/// * `printer` - The pretty-printer that writes the text to `stream`.
/// * `stream` - The [`Write`] object that will receive the text.
/// * `s` - The flowable text.
/// * `next_line_text` - Text to write if the pretty-printer needs to break `s` into multiple lines.
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
