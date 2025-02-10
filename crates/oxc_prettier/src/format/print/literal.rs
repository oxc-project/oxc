use std::borrow::Cow;

use cow_utils::CowUtils;
use oxc_allocator::String;
use oxc_span::Span;

use crate::{
    dynamic_text,
    ir::{Doc, JoinSeparator},
    join, Prettier,
};

/// Print quoted string.
/// Quotes are automatically chosen based on the content of the string and option.
pub fn print_string<'a>(
    p: &Prettier<'a>,
    quoted_raw_text: &'a str,
    prefer_single_quote: bool,
) -> Doc<'a> {
    debug_assert!(
        quoted_raw_text.starts_with('\'') && quoted_raw_text.ends_with('\'')
            || quoted_raw_text.starts_with('"') && quoted_raw_text.ends_with('"')
    );

    let original_quote = quoted_raw_text.chars().next().unwrap();
    let not_quoted_raw_text = &quoted_raw_text[1..quoted_raw_text.len() - 1];

    let enclosing_quote = get_preferred_quote(not_quoted_raw_text, prefer_single_quote);

    // This keeps useless escape as-is
    if original_quote == enclosing_quote {
        return dynamic_text!(p, quoted_raw_text);
    }

    dynamic_text!(p, make_string(p, not_quoted_raw_text, enclosing_quote).into_bump_str())
}

// TODO: Can this be removed? It does not exist in Prettier
/// Print quoted string from not quoted text.
/// Mainly this is used to add quotes for object property keys.
pub fn print_string_from_not_quoted_raw_text<'a>(
    p: &Prettier<'a>,
    not_quoted_raw_text: &str,
    prefer_single_quote: bool,
) -> Doc<'a> {
    let enclosing_quote = get_preferred_quote(not_quoted_raw_text, prefer_single_quote);
    dynamic_text!(p, make_string(p, not_quoted_raw_text, enclosing_quote).into_bump_str())
}

// See https://github.com/prettier/prettier/blob/3.3.3/src/utils/print-number.js
// Perf: the regexes from prettier code above are ported to manual search for performance reasons.
pub fn print_number<'a>(p: &Prettier<'a>, raw_text: &str) -> Doc<'a> {
    let mut string = raw_text.cow_to_ascii_lowercase();

    // Remove unnecessary plus and zeroes from scientific notation.
    if let Some((head, tail)) = string.split_once('e') {
        let negative = if tail.starts_with('-') { "-" } else { "" };
        let trimmed = tail.trim_start_matches(['+', '-']).trim_start_matches('0');
        if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
            string = Cow::Owned(std::format!("{head}e{negative}{trimmed}"));
        }
    }

    // Remove unnecessary scientific notation (1e0).
    if let Some((head, tail)) = string.split_once('e') {
        if tail.trim_start_matches(['+', '-']).trim_start_matches('0').is_empty() {
            string = Cow::Owned(head.to_string());
        }
    }

    // Make sure numbers always start with a digit.
    if string.starts_with('.') {
        string = Cow::Owned(std::format!("0{string}"));
    }

    // Remove extraneous trailing decimal zeroes.
    if let Some((head, tail)) = string.split_once('.') {
        if let Some((head_e, tail_e)) = tail.split_once('e') {
            if !head_e.is_empty() {
                let trimmed = head_e.trim_end_matches('0');
                if trimmed.is_empty() {
                    string = Cow::Owned(std::format!("{head}.0e{tail_e}"));
                } else {
                    string = Cow::Owned(std::format!("{head}.{trimmed}e{tail_e}"));
                }
            }
        } else if !tail.is_empty() {
            let trimmed = tail.trim_end_matches('0');
            if trimmed.is_empty() {
                string = Cow::Owned(std::format!("{head}.0"));
            } else {
                string = Cow::Owned(std::format!("{head}.{trimmed}"));
            }
        }
    }

    // Remove trailing dot.
    if let Some((head, tail)) = string.split_once('.') {
        if tail.is_empty() {
            string = Cow::Owned(head.to_string());
        } else if tail.starts_with('e') {
            string = Cow::Owned(std::format!("{head}{tail}"));
        }
    }

    dynamic_text!(p, &string)
}

pub fn get_preferred_quote(not_quoted_raw_text: &str, prefer_single_quote: bool) -> char {
    let (preferred_quote_char, alternate_quote_char) =
        if prefer_single_quote { ('\'', '"') } else { ('"', '\'') };

    let mut preferred_quote_count = 0;
    let mut alternate_quote_count = 0;

    for character in not_quoted_raw_text.chars() {
        if character == preferred_quote_char {
            preferred_quote_count += 1;
        } else if character == alternate_quote_char {
            alternate_quote_count += 1;
        }
    }

    if preferred_quote_count > alternate_quote_count {
        alternate_quote_char
    } else {
        preferred_quote_char
    }
}

fn make_string<'a>(
    p: &Prettier<'a>,
    not_quoted_raw_text: &str,
    enclosing_quote: char,
) -> String<'a> {
    let other_quote = if enclosing_quote == '"' { '\'' } else { '"' };

    let mut result = String::new_in(p.allocator);
    result.push(enclosing_quote);

    let mut chars = not_quoted_raw_text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&nc) = chars.peek() {
                if nc == other_quote {
                    // Skip(remove) useless escape
                    chars.next();
                    result.push(nc);
                } else {
                    result.push('\\');
                    if let Some(nc) = chars.next() {
                        result.push(nc);
                    }
                }
            } else {
                result.push('\\');
            }
        } else if c == enclosing_quote {
            result.push('\\');
            result.push(c);
        } else {
            result.push(c);
        }
    }

    result.push(enclosing_quote);
    result
}
