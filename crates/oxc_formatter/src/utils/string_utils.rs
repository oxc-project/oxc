use std::borrow::Cow;

use unicode_width::UnicodeWidthStr;

use oxc_span::{SourceType, Span};
use oxc_syntax::identifier::is_identifier_name;

use crate::{
    FormatOptions, QuoteProperties, QuoteStyle,
    formatter::{Format, FormatResult, Formatter, prelude::*, token::string::normalize_string},
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
#[derive(Clone, Copy)]
pub struct FormatLiteralStringToken<'a> {
    /// The current string
    string: &'a str,

    span: Span,

    jsx: bool,

    /// The parent that holds the token
    parent_kind: StringLiteralParentKind,
}

impl<'a> FormatLiteralStringToken<'a> {
    pub fn new(
        string: &'a str,
        span: Span,
        jsx: bool,
        parent_kind: StringLiteralParentKind,
    ) -> Self {
        Self { string, span, jsx, parent_kind }
    }

    pub fn clean_text(
        &self,
        source_type: SourceType,
        options: &FormatOptions,
    ) -> CleanedStringLiteralText<'a> {
        let chosen_quote_style =
            if self.jsx { options.jsx_quote_style } else { options.quote_style };
        let chosen_quote_properties = options.quote_properties;

        let mut string_cleaner =
            LiteralStringNormaliser::new(*self, chosen_quote_style, chosen_quote_properties);

        let content = string_cleaner.normalise_text(source_type);
        let normalized_text_width = content.width();

        CleanedStringLiteralText {
            string: self.string,
            text: content,
            span: self.span,
            width: normalized_text_width,
        }
    }
}

pub struct CleanedStringLiteralText<'a> {
    string: &'a str,
    text: Cow<'a, str>,
    span: Span,
    width: usize,
}

impl CleanedStringLiteralText<'_> {
    pub fn width(&self) -> usize {
        self.width
    }
}

impl<'a> Format<'a> for CleanedStringLiteralText<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        syntax_token_cow_slice(self.text.clone(), self.span).fmt(f)
    }
}

impl<'a> Format<'a> for FormatLiteralStringToken<'a> {
    fn fmt(&self, f: &mut Formatter<'_, 'a>) -> FormatResult<()> {
        self.clean_text(f.context().source_type(), f.options()).fmt(f)
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
/// the normalise process.
struct LiteralStringNormaliser<'a> {
    /// The current token
    token: FormatLiteralStringToken<'a>,
    /// The quote that was set inside the configuration
    chosen_quote_style: QuoteStyle,
    /// When properties in objects are quoted that was set inside the configuration
    chosen_quote_properties: QuoteProperties,
}

impl<'a> LiteralStringNormaliser<'a> {
    pub fn new(
        token: FormatLiteralStringToken<'a>,
        chosen_quote_style: QuoteStyle,
        chosen_quote_properties: QuoteProperties,
    ) -> Self {
        Self { token, chosen_quote_style, chosen_quote_properties }
    }

    fn normalise_text(&mut self, source_type: SourceType) -> Cow<'a, str> {
        let str_info = self.token.compute_string_information(self.chosen_quote_style);
        match self.token.parent_kind {
            StringLiteralParentKind::Expression => self.normalise_string_literal(str_info),
            StringLiteralParentKind::Directive => self.normalise_directive(str_info),
            StringLiteralParentKind::ImportAttribute => self.normalise_import_attribute(str_info),
            StringLiteralParentKind::Member => self.normalise_type_member(str_info, source_type),
        }
    }

    fn normalise_import_attribute(
        &mut self,
        string_information: StringInformation,
    ) -> Cow<'a, str> {
        let quoteless = self.raw_content();
        let can_remove_quotes =
            !self.is_preserve_quote_properties() && is_identifier_name(quoteless);
        if can_remove_quotes {
            Cow::Owned(quoteless.to_string())
        } else {
            self.normalise_string_literal(string_information)
        }
    }

    fn normalise_directive(&mut self, string_information: StringInformation) -> Cow<'a, str> {
        // In diretcives, unnecessary escapes should be preserved.
        // See https://github.com/prettier/prettier/issues/1555
        // Thus we don't normalise the string.
        //
        // Since the string is not normalised, we should not change the quotes,
        // if the directive contains some quotes.
        //
        // Note that we could change the quotes if the preferred quote is escaped.
        // However, Prettier doesn't go that far.
        if string_information.raw_content_has_quotes {
            Cow::Borrowed(self.token.string)
        } else {
            self.swap_quotes(self.raw_content(), string_information)
        }
    }

    fn is_preserve_quote_properties(&self) -> bool {
        self.chosen_quote_properties == QuoteProperties::Preserve
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

    fn normalise_type_member(
        &mut self,
        string_information: StringInformation,
        source_type: SourceType,
    ) -> Cow<'a, str> {
        let quoteless = self.raw_content();
        let can_remove_quotes = !self.is_preserve_quote_properties()
            && (self.can_remove_number_quotes_by_file_type(source_type)
                || is_identifier_name(quoteless));
        if can_remove_quotes {
            Cow::Owned(quoteless.to_string())
        } else {
            self.normalise_string_literal(string_information)
        }
    }

    fn normalise_string_literal(&self, string_information: StringInformation) -> Cow<'a, str> {
        let preferred_quote = string_information.preferred_quote;
        let polished_raw_content = normalize_string(
            self.raw_content(),
            string_information.preferred_quote.into(),
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
