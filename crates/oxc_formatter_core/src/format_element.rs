#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FormatElement {
    Text(String),
    Space,
    Line(LineMode),
    IndentStart,
    IndentEnd,
}

impl FormatElement {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LineMode {
    Hard,
}
