use std::marker::PhantomData;
use std::{fmt, rc::Rc, str::FromStr};

use oxc_allocator::Vec;
use oxc_ast::{Comment, ast::Program};

use crate::format::JsFormatter;
use crate::formatter::formatter::Formatter;
use crate::formatter::prelude::{dynamic_text, if_group_breaks, text};
use crate::formatter::printer::PrinterOptions;
use crate::formatter::{
    AttributePosition, BracketSameLine, BracketSpacing, Buffer, CommentKind, CommentPlacement,
    CommentStyle, Comments, DecoratedComment, Expand, Format, FormatContext, FormatOptions,
    FormatResult, FormatRule, IndentStyle, IndentWidth, LineEnding, LineWidth, QuoteStyle,
    SourceComment, SyntaxTriviaPieceComments,
};
use crate::write;

#[derive(Debug, Clone)]
pub struct JsFormatContext<'a> {
    options: JsFormatOptions,
    source_text: &'a str,
    comments: Comments,
    // cached_function_body: Option<(AnyJsFunctionBody, FormatElement)>,
    // source_map: Option<TransformSourceMap>,
}

impl<'a> JsFormatContext<'a> {
    pub fn new(program: &'a Program<'a>, options: JsFormatOptions) -> Self {
        Self {
            options,
            source_text: program.source_text,
            comments: Comments::from_oxc_comments(&program.comments),
        }
    }

    pub fn source_text(&self) -> &'a str {
        self.source_text
    }
}

impl<'a> FormatContext<'a> for JsFormatContext<'a> {
    type Options = JsFormatOptions;
    type Style = JsCommentStyle;
    type CommentRule = FormatJsLeadingComment<'a>;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_text(&self) -> &'a str {
        self.source_text
    }

    fn comments(&self) -> &Comments {
        &self.comments
    }
}

#[derive(Default)]
pub struct FormatJsLeadingComment<'a> {
    _phantom_data: PhantomData<&'a ()>,
}

impl<'a> FormatRule<SourceComment> for FormatJsLeadingComment<'a> {
    type Context = JsFormatContext<'a>;
    fn fmt(&self, comment: &SourceComment, f: &mut Formatter<Self::Context>) -> FormatResult<()> {
        let text = comment.span.source_text(f.context().source_text());
        write!(f, [dynamic_text(text, comment.span.start)])
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct JsCommentStyle;

impl CommentStyle for JsCommentStyle {
    fn is_suppression(text: &str) -> bool {
        todo!()
    }

    fn get_comment_kind(comment: &SyntaxTriviaPieceComments) -> CommentKind {
        todo!()
    }

    fn place_comment(&self, comment: DecoratedComment) -> CommentPlacement {
        todo!()
    }
}

// ---

#[derive(Debug, Default, Clone)]
pub struct JsFormatOptions {
    /// The indent style.
    indent_style: IndentStyle,

    /// The indent width.
    indent_width: IndentWidth,

    /// The type of line ending.
    line_ending: LineEnding,

    /// What's the max width of a line. Defaults to 80.
    line_width: LineWidth,

    /// The style for quotes. Defaults to double.
    quote_style: QuoteStyle,

    /// The style for JSX quotes. Defaults to double.
    jsx_quote_style: QuoteStyle,

    /// When properties in objects are quoted. Defaults to as-needed.
    quote_properties: QuoteProperties,

    /// Print trailing commas wherever possible in multi-line comma-separated syntactic structures. Defaults to "all".
    trailing_commas: TrailingCommas,

    /// Whether the formatter prints semicolons for all statements, class members, and type members or only when necessary because of [ASI](https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-automatic-semicolon-insertion).
    semicolons: Semicolons,

    /// Whether to add non-necessary parentheses to arrow functions. Defaults to "always".
    arrow_parentheses: ArrowParentheses,

    /// Whether to insert spaces around brackets in object literals. Defaults to true.
    bracket_spacing: BracketSpacing,

    /// Whether to hug the closing bracket of multiline HTML/JSX tags to the end of the last line, rather than being alone on the following line. Defaults to false.
    bracket_same_line: BracketSameLine,

    /// Attribute position style. By default auto.
    attribute_position: AttributePosition,

    /// Whether to expand object and array literals to multiple lines. Defaults to "auto".
    expand: Expand,
}

impl JsFormatOptions {
    pub fn new() -> Self {
        Self {
            indent_style: IndentStyle::default(),
            indent_width: IndentWidth::default(),
            line_ending: LineEnding::default(),
            line_width: LineWidth::default(),
            quote_style: QuoteStyle::default(),
            jsx_quote_style: QuoteStyle::default(),
            quote_properties: QuoteProperties::default(),
            trailing_commas: TrailingCommas::default(),
            semicolons: Semicolons::default(),
            arrow_parentheses: ArrowParentheses::default(),
            bracket_spacing: BracketSpacing::default(),
            bracket_same_line: BracketSameLine::default(),
            attribute_position: AttributePosition::default(),
            expand: Expand::default(),
        }
    }

    pub fn with_arrow_parentheses(mut self, arrow_parentheses: ArrowParentheses) -> Self {
        self.arrow_parentheses = arrow_parentheses;
        self
    }

    pub fn with_bracket_spacing(mut self, bracket_spacing: BracketSpacing) -> Self {
        self.bracket_spacing = bracket_spacing;
        self
    }

    pub fn with_bracket_same_line(mut self, bracket_same_line: BracketSameLine) -> Self {
        self.bracket_same_line = bracket_same_line;
        self
    }

    pub fn with_indent_style(mut self, indent_style: IndentStyle) -> Self {
        self.indent_style = indent_style;
        self
    }

    pub fn with_indent_width(mut self, indent_width: IndentWidth) -> Self {
        self.indent_width = indent_width;
        self
    }

    pub fn with_line_ending(mut self, line_ending: LineEnding) -> Self {
        self.line_ending = line_ending;
        self
    }

    pub fn with_line_width(mut self, line_width: LineWidth) -> Self {
        self.line_width = line_width;
        self
    }

    pub fn with_quote_style(mut self, quote_style: QuoteStyle) -> Self {
        self.quote_style = quote_style;
        self
    }

    pub fn with_jsx_quote_style(mut self, jsx_quote_style: QuoteStyle) -> Self {
        self.jsx_quote_style = jsx_quote_style;
        self
    }

    pub fn with_quote_properties(mut self, quote_properties: QuoteProperties) -> Self {
        self.quote_properties = quote_properties;
        self
    }

    pub fn with_trailing_commas(mut self, trailing_commas: TrailingCommas) -> Self {
        self.trailing_commas = trailing_commas;
        self
    }

    pub fn with_semicolons(mut self, semicolons: Semicolons) -> Self {
        self.semicolons = semicolons;
        self
    }

    pub fn with_attribute_position(mut self, attribute_position: AttributePosition) -> Self {
        self.attribute_position = attribute_position;
        self
    }

    pub fn with_expand(mut self, expand: Expand) -> Self {
        self.expand = expand;
        self
    }

    pub fn set_arrow_parentheses(&mut self, arrow_parentheses: ArrowParentheses) {
        self.arrow_parentheses = arrow_parentheses;
    }

    pub fn set_bracket_spacing(&mut self, bracket_spacing: BracketSpacing) {
        self.bracket_spacing = bracket_spacing;
    }

    pub fn set_bracket_same_line(&mut self, bracket_same_line: BracketSameLine) {
        self.bracket_same_line = bracket_same_line;
    }

    pub fn set_indent_style(&mut self, indent_style: IndentStyle) {
        self.indent_style = indent_style;
    }

    pub fn set_indent_width(&mut self, indent_width: IndentWidth) {
        self.indent_width = indent_width;
    }

    pub fn set_line_ending(&mut self, line_ending: LineEnding) {
        self.line_ending = line_ending;
    }

    pub fn set_line_width(&mut self, line_width: LineWidth) {
        self.line_width = line_width;
    }

    pub fn set_quote_style(&mut self, quote_style: QuoteStyle) {
        self.quote_style = quote_style;
    }

    pub fn set_jsx_quote_style(&mut self, jsx_quote_style: QuoteStyle) {
        self.jsx_quote_style = jsx_quote_style;
    }

    pub fn set_quote_properties(&mut self, quote_properties: QuoteProperties) {
        self.quote_properties = quote_properties;
    }

    pub fn set_trailing_commas(&mut self, trailing_commas: TrailingCommas) {
        self.trailing_commas = trailing_commas;
    }

    pub fn set_attribute_position(&mut self, attribute_position: AttributePosition) {
        self.attribute_position = attribute_position;
    }

    pub fn set_expand(&mut self, expand: Expand) {
        self.expand = expand;
    }

    pub fn set_semicolons(&mut self, semicolons: Semicolons) {
        self.semicolons = semicolons;
    }

    pub fn arrow_parentheses(&self) -> ArrowParentheses {
        self.arrow_parentheses
    }

    pub fn bracket_spacing(&self) -> BracketSpacing {
        self.bracket_spacing
    }

    pub fn bracket_same_line(&self) -> BracketSameLine {
        self.bracket_same_line
    }

    pub fn quote_style(&self) -> QuoteStyle {
        self.quote_style
    }

    pub fn jsx_quote_style(&self) -> QuoteStyle {
        self.jsx_quote_style
    }

    pub fn quote_properties(&self) -> QuoteProperties {
        self.quote_properties
    }

    pub fn trailing_commas(&self) -> TrailingCommas {
        self.trailing_commas
    }

    pub fn semicolons(&self) -> Semicolons {
        self.semicolons
    }

    pub fn tab_width(&self) -> TabWidth {
        self.indent_width.value().into()
    }

    pub fn attribute_position(&self) -> AttributePosition {
        self.attribute_position
    }

    pub fn expand(&self) -> Expand {
        self.expand
    }
}

impl FormatOptions for JsFormatOptions {
    fn indent_style(&self) -> IndentStyle {
        self.indent_style
    }

    fn indent_width(&self) -> IndentWidth {
        self.indent_width
    }

    fn line_width(&self) -> LineWidth {
        self.line_width
    }

    fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    fn as_print_options(&self) -> PrinterOptions {
        PrinterOptions::from(self)
    }
}

impl fmt::Display for JsFormatOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Indent style: {}", self.indent_style)?;
        writeln!(f, "Indent width: {}", self.indent_width.value())?;
        writeln!(f, "Line ending: {}", self.line_ending)?;
        writeln!(f, "Line width: {}", self.line_width.value())?;
        writeln!(f, "Quote style: {}", self.quote_style)?;
        writeln!(f, "JSX quote style: {}", self.jsx_quote_style)?;
        writeln!(f, "Quote properties: {}", self.quote_properties)?;
        writeln!(f, "Trailing commas: {}", self.trailing_commas)?;
        writeln!(f, "Semicolons: {}", self.semicolons)?;
        writeln!(f, "Arrow parentheses: {}", self.arrow_parentheses)?;
        writeln!(f, "Bracket spacing: {}", self.bracket_spacing.value())?;
        writeln!(f, "Bracket same line: {}", self.bracket_same_line.value())?;
        writeln!(f, "Attribute Position: {}", self.attribute_position)?;
        writeln!(f, "Expand lists: {}", self.expand)
    }
}

// ---

#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub struct TabWidth(u8);

impl From<u8> for TabWidth {
    fn from(value: u8) -> Self {
        TabWidth(value)
    }
}

impl From<TabWidth> for u8 {
    fn from(width: TabWidth) -> Self {
        width.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum QuoteProperties {
    #[default]
    AsNeeded,
    Preserve,
}

impl FromStr for QuoteProperties {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "preserve" => Ok(Self::Preserve),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for QuoteProperties"),
        }
    }
}

impl fmt::Display for QuoteProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuoteProperties::AsNeeded => std::write!(f, "As needed"),
            QuoteProperties::Preserve => std::write!(f, "Preserve"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Semicolons {
    #[default]
    Always,
    AsNeeded,
}

impl Semicolons {
    pub const fn is_as_needed(&self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

impl FromStr for Semicolons {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Semicolons. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for Semicolons {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Semicolons::AsNeeded => std::write!(f, "As needed"),
            Semicolons::Always => std::write!(f, "Always"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ArrowParentheses {
    #[default]
    Always,
    AsNeeded,
}

impl ArrowParentheses {
    pub const fn is_as_needed(&self) -> bool {
        matches!(self, Self::AsNeeded)
    }

    pub const fn is_always(&self) -> bool {
        matches!(self, Self::Always)
    }
}

// Required by [Bpaf]
impl FromStr for ArrowParentheses {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "as-needed" => Ok(Self::AsNeeded),
            "always" => Ok(Self::Always),
            _ => Err(
                "Value not supported for Arrow parentheses. Supported values are 'as-needed' and 'always'.",
            ),
        }
    }
}

impl fmt::Display for ArrowParentheses {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArrowParentheses::AsNeeded => std::write!(f, "As needed"),
            ArrowParentheses::Always => std::write!(f, "Always"),
        }
    }
}

/// This enum is used within formatting functions to print or omit trailing commas.
#[derive(Debug, Copy, Clone)]
pub enum FormatTrailingCommas {
    /// Print trailing commas if the option is [TrailingCommas::All].
    All,
    /// Print trailing commas if the option is [TrailingCommas::All] or [TrailingCommas::Es5].
    ES5,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub enum TrailingSeparator {
    /// A trailing separator is allowed and preferred
    #[default]
    Allowed,

    /// A trailing separator is not allowed
    Disallowed,

    /// A trailing separator is mandatory for the syntax to be correct
    Mandatory,

    /// A trailing separator might be present, but the consumer
    /// decides to remove it
    Omit,
}

impl FormatTrailingCommas {
    /// This function returns corresponding [TrailingSeparator] for [format_separated] function.
    pub fn trailing_separator(&self, options: &JsFormatOptions) -> TrailingSeparator {
        if options.trailing_commas.is_none() {
            return TrailingSeparator::Omit;
        }

        match self {
            FormatTrailingCommas::All => {
                if options.trailing_commas.is_all() {
                    TrailingSeparator::Allowed
                } else {
                    TrailingSeparator::Omit
                }
            }
            FormatTrailingCommas::ES5 => TrailingSeparator::Allowed,
        }
    }
}

impl<'ast> Format<JsFormatContext<'_>> for FormatTrailingCommas {
    fn fmt(&self, f: &mut Formatter<JsFormatContext>) -> FormatResult<()> {
        if f.options().trailing_commas.is_none() {
            return Ok(());
        }

        if matches!(self, FormatTrailingCommas::ES5) || f.options().trailing_commas().is_all() {
            write!(f, [if_group_breaks(&text(","))])?;
        }

        Ok(())
    }
}

/// Print trailing commas wherever possible in multi-line comma-separated syntactic structures.
#[derive(Clone, Copy, Default, Debug, Eq, Hash, PartialEq)]
pub enum TrailingCommas {
    /// Trailing commas wherever possible (including function parameters and calls).
    #[default]
    All,
    /// Trailing commas where valid in ES5 (objects, arrays, etc.). No trailing commas in type parameters in TypeScript.
    Es5,
    /// No trailing commas.
    None,
}

impl TrailingCommas {
    pub const fn is_es5(&self) -> bool {
        matches!(self, TrailingCommas::Es5)
    }
    pub const fn is_all(&self) -> bool {
        matches!(self, TrailingCommas::All)
    }
    pub const fn is_none(&self) -> bool {
        matches!(self, TrailingCommas::None)
    }
}

impl FromStr for TrailingCommas {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "es5" => Ok(Self::Es5),
            "all" => Ok(Self::All),
            "none" => Ok(Self::None),
            // TODO: replace this error with a diagnostic
            _ => Err("Value not supported for TrailingCommas"),
        }
    }
}

impl fmt::Display for TrailingCommas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrailingCommas::Es5 => std::write!(f, "ES5"),
            TrailingCommas::All => std::write!(f, "All"),
            TrailingCommas::None => std::write!(f, "None"),
        }
    }
}
