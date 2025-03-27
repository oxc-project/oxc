use super::{
    CommentStyle, Comments, FormatOptions, FormatRule, SimpleFormatOptions, SourceComment,
};

/// Context object storing data relevant when formatting an object.
pub trait FormatContext<'a> {
    type Options: FormatOptions;

    type Style: CommentStyle;

    /// Rule for formatting comments.
    type CommentRule: FormatRule<SourceComment, Context = Self> + Default;

    /// Returns the formatting options
    fn options(&self) -> &Self::Options;

    /// Returns a reference to the program's comments.
    fn comments(&self) -> &Comments;

    /// Returns the formatting options
    fn source_text(&self) -> &'a str;
}
