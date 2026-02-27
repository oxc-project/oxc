use crate::format_element::{FormatElement, LineMode};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum IndentStyle {
    Tab,
    Space,
}

impl Default for IndentStyle {
    fn default() -> Self {
        Self::Space
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum LineEnding {
    Lf,
    Crlf,
    Cr,
}

impl Default for LineEnding {
    fn default() -> Self {
        Self::Lf
    }
}

impl LineEnding {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
            Self::Cr => "\r",
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PrinterOptions {
    pub indent_style: IndentStyle,
    pub indent_width: u8,
    pub line_ending: LineEnding,
    pub line_width: u16,
}

impl Default for PrinterOptions {
    fn default() -> Self {
        Self {
            indent_style: IndentStyle::Space,
            indent_width: 2,
            line_ending: LineEnding::Lf,
            line_width: 80,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Printer {
    options: PrinterOptions,
}

impl Printer {
    pub fn new(options: PrinterOptions) -> Self {
        Self { options }
    }

    pub fn print(&self, elements: &[FormatElement]) -> String {
        let mut output = String::new();
        let mut indent_level: usize = 0;
        let mut at_line_start = true;

        for element in elements {
            match element {
                FormatElement::Text(text) => {
                    if at_line_start {
                        Self::write_indent(&mut output, indent_level, self.options);
                        at_line_start = false;
                    }
                    output.push_str(text);
                }
                FormatElement::Space => {
                    if at_line_start {
                        Self::write_indent(&mut output, indent_level, self.options);
                        at_line_start = false;
                    }
                    output.push(' ');
                }
                FormatElement::Line(LineMode::Hard) => {
                    output.push_str(self.options.line_ending.as_str());
                    at_line_start = true;
                }
                FormatElement::IndentStart => {
                    indent_level += 1;
                }
                FormatElement::IndentEnd => {
                    indent_level = indent_level.saturating_sub(1);
                }
            }
        }

        output
    }

    fn write_indent(output: &mut String, indent_level: usize, options: PrinterOptions) {
        match options.indent_style {
            IndentStyle::Tab => {
                for _ in 0..indent_level {
                    output.push('\t');
                }
            }
            IndentStyle::Space => {
                let count = indent_level * usize::from(options.indent_width.max(1));
                for _ in 0..count {
                    output.push(' ');
                }
            }
        }
    }
}
