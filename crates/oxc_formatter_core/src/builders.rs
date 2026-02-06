use crate::format_element::{FormatElement, LineMode};

pub fn text(text: impl Into<String>) -> FormatElement {
    FormatElement::text(text)
}

pub fn space() -> FormatElement {
    FormatElement::Space
}

pub fn hard_line_break() -> FormatElement {
    FormatElement::Line(LineMode::Hard)
}

pub fn indent(elements: Vec<FormatElement>) -> Vec<FormatElement> {
    let mut result = Vec::with_capacity(elements.len() + 2);
    result.push(FormatElement::IndentStart);
    result.extend(elements);
    result.push(FormatElement::IndentEnd);
    result
}

pub fn join<I>(entries: I, separator: Vec<FormatElement>) -> Vec<FormatElement>
where
    I: IntoIterator<Item = Vec<FormatElement>>,
{
    let mut result = Vec::new();
    let mut is_first = true;

    for entry in entries {
        if !is_first {
            result.extend(separator.iter().cloned());
        }
        result.extend(entry);
        is_first = false;
    }

    result
}
