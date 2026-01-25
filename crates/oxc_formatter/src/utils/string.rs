use std::{borrow::Cow, ops::Deref};

use oxc_span::SourceType;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};
use unicode_width::UnicodeWidthStr;

use crate::{
    QuoteProperties, QuoteStyle,
    formatter::{Format, Formatter, prelude::*},
};

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum StringLiteralParentKind {
    /// Variant to track tokens that are inside an expression
    Expression,
    /// Variant to track tokens that are inside a member
    Member,
    /// Variant to track tokens that are inside an import attribute
    ImportAttribute,
    /// Variant used when the string literal is inside a directive. This will apply
    /// a simplified logic of normalisation
    Directive,
}

/// Data structure of convenience to format string literals
#[derive(Clone, Copy, Debug)]
pub struct FormatLiteralStringToken<'a> {
    /// The current string
    string: &'a str,

    jsx: bool,

    /// The parent that holds the token
    parent_kind: StringLiteralParentKind,
}

impl<'a> FormatLiteralStringToken<'a> {
    pub fn new(string: &'a str, jsx: bool, parent_kind: StringLiteralParentKind) -> Self {
        Self { string, jsx, parent_kind }
    }

    pub fn clean_text(&self, f: &Formatter<'_, 'a>) -> CleanedStringLiteralText<'a> {
        let options = f.options();
        let source_type = f.context().source_type();

        let chosen_quote_style =
            if self.jsx { options.jsx_quote_style } else { options.quote_style };

        let is_quote_needed = match options.quote_properties {
            QuoteProperties::AsNeeded => false,
            QuoteProperties::Preserve => true,
            QuoteProperties::Consistent => f.context().is_quote_needed(),
        };

        let string_cleaner =
            LiteralStringNormalizer::new(*self, chosen_quote_style, is_quote_needed);

        let content = string_cleaner.normalize_text(source_type);

        CleanedStringLiteralText { text: content }
    }
}

pub struct CleanedStringLiteralText<'a> {
    text: Cow<'a, str>,
}

impl Deref for CleanedStringLiteralText<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl CleanedStringLiteralText<'_> {
    pub fn width(&self) -> usize {
        self.text.width()
    }
}

impl<'a> Format<'a> for CleanedStringLiteralText<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        text(f.context().allocator().alloc_str(&self.text)).fmt(f);
    }
}

/// Data structure of convenience to store some information about the
/// string that has been processed
#[derive(Clone, Copy)]
struct StringInformation {
    /// Currently used quote
    current_quote: QuoteStyle,
    /// This is the quote that is calculated and eventually used inside the string.
    /// It could be different from the one inside the formatter options
    preferred_quote: QuoteStyle,
    /// It flags if the raw content has quotes (single or double). The raw content is the
    /// content of a string literal without the quotes
    raw_content_has_quotes: bool,
}

impl FormatLiteralStringToken<'_> {
    /// This function determines which quotes should be used inside to enclose the string.
    /// The function take as a input the string **without quotes**.
    ///
    /// # How it works
    ///
    /// The function determines the preferred quote and alternate quote.
    /// The preferred quote is the one that comes from the formatter options. The alternate quote is the other one.
    ///
    /// We check how many preferred quotes we have inside the content. If this number is greater than the
    /// number alternate quotes that we have inside the content,
    /// then we swap them, so we can reduce the number of escaped quotes.
    ///
    /// For example, let's suppose that the preferred quote is double, and we have a string like this:
    /// ```js
    /// (" content \"\"\" don't ")
    /// ```
    /// Excluding the quotes at the start and beginning, we have three double quote and one single quote.
    /// If we decided to keep them like this, we would have three escaped quotes.
    ///
    /// But then, we choose the single quote as preferred quote and we would have only one quote that is escaped,
    /// resulting into a string like this:
    /// ```js
    /// (' content """ dont\'t ')
    /// ```
    /// Like this, we reduced the number of escaped quotes.
    fn compute_string_information(&self, chosen_quote: QuoteStyle) -> StringInformation {
        let literal = self.string;
        let alternate_quote = chosen_quote.other();
        let chosen_quote_byte = chosen_quote.as_byte();
        let alternate_quote_byte = alternate_quote.as_byte();

        debug_assert!(
            literal
                .bytes()
                .next()
                .is_some_and(|c| c == chosen_quote_byte || c == alternate_quote_byte),
            "string must start with a quote"
        );
        debug_assert!(
            literal
                .bytes()
                .last()
                .is_some_and(|c| c == chosen_quote_byte || c == alternate_quote_byte),
            "string must end with a quote"
        );

        let quoteless = &literal[1..literal.len() - 1];
        let (chosen_quote_count, alternate_quote_count) = quoteless.bytes().fold(
            (0u32, 0u32),
            |(chosen_quote_count, alternate_quote_count), current_character| {
                if current_character == chosen_quote_byte {
                    (chosen_quote_count + 1, alternate_quote_count)
                } else if current_character == alternate_quote_byte {
                    (chosen_quote_count, alternate_quote_count + 1)
                } else {
                    (chosen_quote_count, alternate_quote_count)
                }
            },
        );

        let current_quote =
            literal.bytes().next().and_then(QuoteStyle::from_byte).unwrap_or_default();

        StringInformation {
            current_quote,
            preferred_quote: if chosen_quote_count > alternate_quote_count {
                alternate_quote
            } else {
                chosen_quote
            },
            raw_content_has_quotes: chosen_quote_count > 0 || alternate_quote_count > 0,
        }
    }
}

/// Struct of convenience used to manipulate the string. It saves some state in order to apply
/// the normalize process.
struct LiteralStringNormalizer<'a> {
    /// The current token
    token: FormatLiteralStringToken<'a>,
    /// The quote that was set inside the configuration
    chosen_quote_style: QuoteStyle,
    /// State whether we need to print the quotes or not.
    is_quote_needed: bool,
}

impl<'a> LiteralStringNormalizer<'a> {
    pub fn new(
        token: FormatLiteralStringToken<'a>,
        chosen_quote_style: QuoteStyle,
        is_quote_needed: bool,
    ) -> Self {
        Self { token, chosen_quote_style, is_quote_needed }
    }

    fn normalize_text(&self, source_type: SourceType) -> Cow<'a, str> {
        // Handle JSX attribute strings specially - they use HTML entity escaping
        if self.token.jsx && self.token.parent_kind == StringLiteralParentKind::Expression {
            return self.normalize_jsx_attribute();
        }

        let str_info = self.token.compute_string_information(self.chosen_quote_style);
        match self.token.parent_kind {
            StringLiteralParentKind::Expression => self.normalize_string_literal(str_info),
            StringLiteralParentKind::Directive => self.normalize_directive(str_info),
            StringLiteralParentKind::ImportAttribute => self.normalize_import_attribute(str_info),
            StringLiteralParentKind::Member => self.normalize_type_member(str_info, source_type),
        }
    }

    /// Normalizes a JSX attribute string value using HTML entity escaping.
    fn normalize_jsx_attribute(&self) -> Cow<'a, str> {
        let raw_content = self.raw_content();
        let current_quote =
            self.token.string.bytes().next().and_then(QuoteStyle::from_byte).unwrap_or_default();

        let (normalized, chosen_quote) = normalize_jsx_string(raw_content, self.chosen_quote_style);

        let quote_char = chosen_quote.as_char();

        match normalized {
            Cow::Borrowed(s) if s == raw_content && current_quote == chosen_quote => {
                // No changes needed, return original
                normalize_newlines(self.token.string, ['\r'])
            }
            Cow::Borrowed(s) => {
                // Content unchanged but quotes need swapping
                let normalized_newlines = normalize_newlines(s, ['\r']);
                Cow::Owned(std::format!("{quote_char}{normalized_newlines}{quote_char}"))
            }
            Cow::Owned(s) => {
                // Content changed
                let normalized_newlines = normalize_newlines(&s, ['\r']);
                Cow::Owned(std::format!("{quote_char}{normalized_newlines}{quote_char}"))
            }
        }
    }

    fn normalize_import_attribute(&self, string_information: StringInformation) -> Cow<'a, str> {
        let quoteless = self.raw_content();
        let can_remove_quotes = !self.is_quote_needed && is_identifier_name_patched(quoteless);
        if can_remove_quotes {
            Cow::Borrowed(quoteless)
        } else {
            self.normalize_string_literal(string_information)
        }
    }

    fn normalize_directive(&self, string_information: StringInformation) -> Cow<'a, str> {
        // In directives, unnecessary escapes should be preserved.
        // See https://github.com/prettier/prettier/issues/1555
        // Thus we don't normalize the string.
        //
        // Since the string is not normalized, we should not change the quotes,
        // if the directive contains some quotes.
        //
        // Note that we could change the quotes if the preferred quote is escaped.
        // However, Prettier doesn't go that far.
        let normalized = normalize_newlines(self.raw_content(), ['\r']);
        match normalized {
            Cow::Borrowed(string) => {
                if string_information.raw_content_has_quotes {
                    Cow::Borrowed(self.token.string)
                } else {
                    self.swap_quotes(string, string_information)
                }
            }
            Cow::Owned(string) => {
                let mut s = String::with_capacity(string.len() + 2);
                let quote = if string_information.raw_content_has_quotes {
                    string_information.current_quote.as_char()
                } else {
                    string_information.preferred_quote.as_char()
                };
                s.push(quote);
                s.push_str(&string);
                s.push(quote);
                Cow::Owned(s)
            }
        }
    }

    fn can_remove_number_quotes_by_file_type(&self, source_type: SourceType) -> bool {
        let text_to_check = self.raw_content();

        if text_to_check.bytes().next().is_some_and(|b| b.is_ascii_digit()) {
            if let Ok(parsed) = text_to_check.parse::<f64>() {
                // In TypeScript, numbers like members have different meaning from numbers.
                // Hence, if we see a number, we bail straightaway
                if source_type.is_typescript() {
                    return false;
                }

                // Rule out inexact floats and octal literals
                return parsed.to_string() == text_to_check;
            }

            return false;
        }
        false
    }

    fn normalize_type_member(
        &self,
        string_information: StringInformation,
        source_type: SourceType,
    ) -> Cow<'a, str> {
        let quoteless = self.raw_content();
        let can_remove_quotes = !self.is_quote_needed
            && (self.can_remove_number_quotes_by_file_type(source_type)
                || is_identifier_name_patched(quoteless));
        if can_remove_quotes {
            Cow::Borrowed(quoteless)
        } else {
            self.normalize_string_literal(string_information)
        }
    }

    fn normalize_string_literal(&self, string_information: StringInformation) -> Cow<'a, str> {
        let preferred_quote = string_information.preferred_quote;
        let polished_raw_content = normalize_string(
            self.raw_content(),
            string_information.preferred_quote,
            string_information.current_quote != string_information.preferred_quote,
        );

        match polished_raw_content {
            Cow::Borrowed(raw_content) => self.swap_quotes(raw_content, string_information),
            Cow::Owned(mut s) => {
                // content is owned, meaning we allocated a new string,
                // so we force replacing quotes, regardless
                s.insert(0, preferred_quote.as_char());
                s.push(preferred_quote.as_char());
                Cow::Owned(s)
            }
        }
    }

    /// Returns the string without its quotes.
    fn raw_content(&self) -> &'a str {
        let content = self.token.string;
        &content[1..content.len() - 1]
    }

    fn swap_quotes(&self, content_to_use: &'a str, str_info: StringInformation) -> Cow<'a, str> {
        let preferred_quote = str_info.preferred_quote.as_char();
        let original = self.token.string;

        if original.starts_with(preferred_quote) {
            Cow::Borrowed(original)
        } else {
            Cow::Owned(std::format!("{preferred_quote}{content_to_use}{preferred_quote}",))
        }
    }
}

impl<'a> Format<'a> for FormatLiteralStringToken<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) {
        self.clean_text(f).fmt(f);
    }
}

/// This function is responsible of:
///
/// - escaping `preferred_quote`
/// - unescape alternate quotes of `preferred_quote` if `quotes_will_change`
/// - normalize the new lines by replacing `\r\n` and `\r` with `\n`.
///
/// The function allocates a new string only if at least one change is performed.
///
/// In the following example `"` is escaped and the newline is normalized.
///
/// ```
/// use biome_formatter::token::string::{normalize_string, Quote};
/// assert_eq!(
///     normalize_string(" \"He\\llo\\tworld\" \\' \\' \r\n ", Quote::Double, true),
///     " \\\"He\\llo\\tworld\\\" ' ' \n ",
/// );
/// ```
pub fn normalize_string(
    raw_content: &str,
    preferred_quote: QuoteStyle,
    quotes_will_change: bool,
) -> Cow<'_, str> {
    let alternate_quote = preferred_quote.other().as_byte();
    let preferred_quote = preferred_quote.as_byte();
    let mut reduced_string = String::new();
    let mut copy_start = 0;
    let mut bytes = raw_content.bytes().enumerate().peekable();
    while let Some((byte_index, byte)) = bytes.next() {
        match byte {
            // If the next character is escaped
            b'\\' => {
                if let Some(&(escaped_index, escaped)) = bytes.peek() {
                    if escaped == b'\r' {
                        bytes.next(); // consume the \r
                        // Copy up to (not including) the \r
                        reduced_string.push_str(&raw_content[copy_start..escaped_index]);
                        if bytes.next_if(|(_, b)| *b == b'\n').is_some() {
                            // \\\r\n -> keep \\ and \n, skip \r
                            // The \n will be included when we copy from copy_start
                        } else {
                            // \\\r -> convert \r to \n
                            reduced_string.push('\n');
                        }
                        copy_start = escaped_index + 1;
                    } else if quotes_will_change && escaped == alternate_quote {
                        bytes.next(); // consume the escaped character
                        // Unescape alternate quotes if quotes are changing
                        reduced_string.push_str(&raw_content[copy_start..byte_index]);
                        copy_start = escaped_index;
                    } else {
                        bytes.next(); // consume the escaped character
                    }
                }
            }
            // Normalize \r\n and \r to \n
            b'\r' => {
                reduced_string.push_str(&raw_content[copy_start..byte_index]);
                if bytes.next_if(|(_, b)| *b == b'\n').is_some() {
                    // \r\n -> skip \r, the \n will be included when we copy from copy_start
                } else {
                    // Single \r -> convert to \n
                    reduced_string.push('\n');
                }
                copy_start = byte_index + 1;
            }
            _ => {
                // If we encounter a preferred quote and it's not escaped, we have to replace it with
                // an escaped version.
                // This is done because of how the enclosed strings can change.
                // Check `computed_preferred_quote` for more details.
                if byte == preferred_quote {
                    reduced_string.push_str(&raw_content[copy_start..byte_index]);
                    reduced_string.push('\\');
                    copy_start = byte_index;
                }
            }
        }
    }
    if copy_start == 0 && reduced_string.is_empty() {
        Cow::Borrowed(raw_content)
    } else {
        // Copy the remaining characters
        reduced_string.push_str(&raw_content[copy_start..]);
        Cow::Owned(reduced_string)
    }
}

/// Counts actual single and double quotes in JSX attribute content,
/// accounting for HTML entities `&apos;` and `&quot;`.
fn count_jsx_quotes(raw_content: &str) -> (u32, u32) {
    let mut single_count = 0u32;
    let mut double_count = 0u32;
    let mut i = 0;
    let bytes = raw_content.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'\'' {
            single_count += 1;
            i += 1;
        } else if bytes[i] == b'"' {
            double_count += 1;
            i += 1;
        } else if bytes[i] == b'&' {
            // Check for HTML entities
            if raw_content[i..].starts_with("&apos;") {
                single_count += 1;
                i += 6; // len of "&apos;"
            } else if raw_content[i..].starts_with("&quot;") {
                double_count += 1;
                i += 6; // len of "&quot;"
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    (single_count, double_count)
}

/// Normalizes a JSX attribute string value using HTML entity escaping.
///
/// Algorithm (matching Prettier):
/// 1. Unescape `&apos;` → `'` and `&quot;` → `"`
/// 2. Count quotes to pick the style that minimizes escaping
/// 3. Escape only the chosen quote type using HTML entities
///
/// Returns the normalized content (without quotes) and the chosen quote style.
fn normalize_jsx_string(
    raw_content: &str,
    preferred_quote: QuoteStyle,
) -> (Cow<'_, str>, QuoteStyle) {
    // Count quotes (accounting for HTML entities)
    let (single_count, double_count) = count_jsx_quotes(raw_content);

    // Choose quote that minimizes escaping, preferring the configured quote when counts are equal
    let chosen_quote = if preferred_quote == QuoteStyle::Double {
        if double_count > single_count { QuoteStyle::Single } else { QuoteStyle::Double }
    } else if single_count > double_count {
        QuoteStyle::Double
    } else {
        QuoteStyle::Single
    };

    // Fast path: check if any HTML entities or quotes that need escaping exist
    let has_apos_entity = raw_content.contains("&apos;");
    let has_quot_entity = raw_content.contains("&quot;");
    let has_raw_single = raw_content.as_bytes().contains(&b'\'');
    let has_raw_double = raw_content.as_bytes().contains(&b'"');

    let needs_unescape = has_apos_entity || has_quot_entity;
    let needs_escape = match chosen_quote {
        QuoteStyle::Double => has_raw_double,
        QuoteStyle::Single => has_raw_single,
    };

    // Fast path: no changes needed
    if !needs_unescape && !needs_escape {
        return (Cow::Borrowed(raw_content), chosen_quote);
    }

    // Slow path: allocate and transform
    // First unescape HTML entities, then escape the chosen quote type
    let mut result = String::with_capacity(raw_content.len());
    let mut chars = raw_content.char_indices();

    while let Some((i, ch)) = chars.next() {
        if ch == '&' {
            if raw_content[i..].starts_with("&apos;") {
                // Unescape &apos; to '
                if chosen_quote == QuoteStyle::Single {
                    // Need to keep it escaped
                    result.push_str("&apos;");
                } else {
                    result.push('\'');
                }
                // Skip the remaining 5 chars of "&apos;" (we already consumed '&')
                for _ in 0..5 {
                    chars.next();
                }
            } else if raw_content[i..].starts_with("&quot;") {
                // Unescape &quot; to "
                if chosen_quote == QuoteStyle::Double {
                    // Need to keep it escaped
                    result.push_str("&quot;");
                } else {
                    result.push('"');
                }
                // Skip the remaining 5 chars of "&quot;" (we already consumed '&')
                for _ in 0..5 {
                    chars.next();
                }
            } else {
                result.push('&');
            }
        } else if ch == '\'' && chosen_quote == QuoteStyle::Single {
            // Escape raw single quote
            result.push_str("&apos;");
        } else if ch == '"' && chosen_quote == QuoteStyle::Double {
            // Escape raw double quote
            result.push_str("&quot;");
        } else {
            result.push(ch);
        }
    }

    (Cow::Owned(result), chosen_quote)
}

/// `is_identifier_name` patched with KATAKANA MIDDLE DOT and HALFWIDTH KATAKANA MIDDLE DOT
/// Otherwise `({ 'x・': 0 })` gets converted to `({ x・: 0 })`, which breaks in Unicode 4.1 to
/// 15.
/// <https://github.com/oxc-project/unicode-id-start/pull/3>
pub fn is_identifier_name_patched(content: &str) -> bool {
    let mut chars = content.chars();
    chars.next().is_some_and(is_identifier_start)
        && chars.all(|c| is_identifier_part(c) && c != '・' && c != '･')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_newline() {
        // \n unchanged
        assert_eq!(normalize_string("a\nb", QuoteStyle::Double, true), "a\nb");
        // \r\n -> \n
        assert_eq!(normalize_string("a\r\nb", QuoteStyle::Double, true), "a\nb");
        // \r -> \n (single CR)
        assert_eq!(normalize_string("a\rb", QuoteStyle::Double, true), "a\nb");
        assert_eq!(normalize_string("a\r", QuoteStyle::Double, true), "a\n");
        assert_eq!(normalize_string("\rb", QuoteStyle::Double, true), "\nb");
        // escaped \r\n -> escaped \n
        assert_eq!(normalize_string("a\\\r\nb", QuoteStyle::Double, true), "a\\\nb");
        // escaped \r -> escaped \n (single CR)
        assert_eq!(normalize_string("a\\\rb", QuoteStyle::Double, true), "a\\\nb");
    }

    #[test]
    fn normalize_escapes() {
        assert_eq!(normalize_string("\\", QuoteStyle::Double, true), "\\");
        assert_eq!(normalize_string("\\t", QuoteStyle::Double, true), "\\t");
        assert_eq!(normalize_string("\\\u{2028}", QuoteStyle::Double, true), "\\\u{2028}");
        assert_eq!(normalize_string("\\\u{2029}", QuoteStyle::Double, true), "\\\u{2029}");

        assert_eq!(normalize_string(r"a\a", QuoteStyle::Double, true), r"a\a");
        assert_eq!(normalize_string(r"👍\👍", QuoteStyle::Single, true), r"👍\👍");
        assert_eq!(normalize_string("\\\u{2027}", QuoteStyle::Double, true), "\\\u{2027}");
        assert_eq!(normalize_string("\\\u{2030}", QuoteStyle::Double, true), "\\\u{2030}");
    }

    #[test]
    fn normalize_quotes() {
        assert_eq!(normalize_string("\"", QuoteStyle::Double, true), "\\\"");
        assert_eq!(normalize_string(r"\'", QuoteStyle::Double, true), r"'");

        assert_eq!(normalize_string(r"\'", QuoteStyle::Double, false), r"\'");
        assert_eq!(normalize_string("\"", QuoteStyle::Single, false), "\"");
        assert_eq!(normalize_string("\\'", QuoteStyle::Single, false), "\\'");
        assert_eq!(normalize_string("\\\"", QuoteStyle::Single, false), "\\\"");
    }

    #[test]
    fn jsx_count_quotes() {
        // Raw quotes
        assert_eq!(count_jsx_quotes("'"), (1, 0));
        assert_eq!(count_jsx_quotes("\""), (0, 1));
        assert_eq!(count_jsx_quotes("' \""), (1, 1));

        // HTML entities
        assert_eq!(count_jsx_quotes("&apos;"), (1, 0));
        assert_eq!(count_jsx_quotes("&quot;"), (0, 1));
        assert_eq!(count_jsx_quotes("&apos; &quot;"), (1, 1));

        // Mixed
        assert_eq!(count_jsx_quotes("' &apos;"), (2, 0));
        assert_eq!(count_jsx_quotes("\" &quot;"), (0, 2));

        // No quotes
        assert_eq!(count_jsx_quotes("foo"), (0, 0));
        assert_eq!(count_jsx_quotes("&amp;"), (0, 0));
    }

    #[test]
    fn jsx_normalize_no_changes() {
        // No quotes, no entities - should return borrowed
        let (result, quote) = normalize_jsx_string("foo", QuoteStyle::Double);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result, "foo");
        assert_eq!(quote, QuoteStyle::Double);
    }

    #[test]
    fn jsx_normalize_unescape_entities() {
        // &apos; should be unescaped when using double quotes
        let (result, quote) = normalize_jsx_string("&apos;", QuoteStyle::Double);
        assert_eq!(result, "'");
        assert_eq!(quote, QuoteStyle::Double);

        // &quot; should be unescaped when using single quotes
        let (result, quote) = normalize_jsx_string("&quot;", QuoteStyle::Single);
        assert_eq!(result, "\"");
        assert_eq!(quote, QuoteStyle::Single);
    }

    #[test]
    fn jsx_normalize_escape_raw_quotes() {
        // Raw ' with single quote preference -> switches to double quotes to avoid escaping
        let (result, quote) = normalize_jsx_string("'", QuoteStyle::Single);
        assert_eq!(result, "'");
        assert_eq!(quote, QuoteStyle::Double);

        // Raw " with double quote preference -> switches to single quotes to avoid escaping
        let (result, quote) = normalize_jsx_string("\"", QuoteStyle::Double);
        assert_eq!(result, "\"");
        assert_eq!(quote, QuoteStyle::Single);

        // When both quotes are present and counts are equal, use preferred and escape
        // ' " with single preferred -> use single, escape the '
        let (result, quote) = normalize_jsx_string("' \"", QuoteStyle::Single);
        assert_eq!(result, "&apos; \"");
        assert_eq!(quote, QuoteStyle::Single);

        // ' " with double preferred -> use double, escape the "
        let (result, quote) = normalize_jsx_string("' \"", QuoteStyle::Double);
        assert_eq!(result, "' &quot;");
        assert_eq!(quote, QuoteStyle::Double);
    }

    #[test]
    fn jsx_normalize_prefer_less_escaping() {
        // When preferred is double but double quotes are more common, switch to single
        // Input: ' " " -> 1 single, 2 double
        let (result, quote) = normalize_jsx_string("' \" \"", QuoteStyle::Double);
        assert_eq!(quote, QuoteStyle::Single);
        assert_eq!(result, "&apos; \" \"");

        // When preferred is single but single quotes are more common, switch to double
        // Input: ' ' " -> 2 single, 1 double
        let (result, quote) = normalize_jsx_string("' ' \"", QuoteStyle::Single);
        assert_eq!(quote, QuoteStyle::Double);
        assert_eq!(result, "' ' &quot;");
    }

    #[test]
    fn jsx_normalize_with_entities_and_raw() {
        // &apos; " -> 1 single, 1 double - prefer double quotes
        let (result, quote) = normalize_jsx_string("&apos; \"", QuoteStyle::Double);
        assert_eq!(quote, QuoteStyle::Double);
        assert_eq!(result, "' &quot;");

        // ' &quot; -> 1 single, 1 double - prefer single quotes
        let (result, quote) = normalize_jsx_string("' &quot;", QuoteStyle::Single);
        assert_eq!(quote, QuoteStyle::Single);
        assert_eq!(result, "&apos; \"");
    }

    #[test]
    fn jsx_normalize_other_entities_preserved() {
        // &amp; and other entities should be preserved
        let (result, quote) = normalize_jsx_string("&amp;", QuoteStyle::Double);
        assert_eq!(result, "&amp;");
        assert_eq!(quote, QuoteStyle::Double);

        // Mixed entities
        let (result, quote) = normalize_jsx_string("&apos;&amp;&quot;", QuoteStyle::Double);
        assert_eq!(quote, QuoteStyle::Double);
        assert_eq!(result, "'&amp;&quot;");
    }
}
