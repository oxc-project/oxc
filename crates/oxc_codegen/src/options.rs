use std::path::PathBuf;

use oxc_data_structures::code_buffer::{DEFAULT_INDENT_WIDTH, IndentChar};

/// Codegen Options.
#[derive(Debug, Clone)]
pub struct CodegenOptions {
    /// Use single quotes instead of double quotes.
    ///
    /// Default is `false`.
    pub single_quote: bool,

    /// Remove whitespace.
    ///
    /// Default is `false`.
    pub minify: bool,

    /// Print comments?
    ///
    /// At present, only some leading comments are preserved.
    ///
    /// Default is [CommentOptions::default].
    pub comments: CommentOptions,

    /// Enable sourcemap.
    ///
    /// The provided path sets the `source` field in the returned sourcemap.
    ///
    /// Default is `None` - no sourcemap is produced.
    pub source_map_path: Option<PathBuf>,

    /// Indentation character.
    ///
    /// Default is [`IndentChar::Tab`].
    pub indent_char: IndentChar,

    /// Number of characters per indentation level.
    ///
    /// Default is `1`.
    pub indent_width: usize,

    /// Initial indentation level for generated code.
    ///
    /// Default is `0`.
    pub initial_indent: u32,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            single_quote: false,
            minify: false,
            comments: CommentOptions::default(),
            source_map_path: None,
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
            initial_indent: 0,
        }
    }
}

impl CodegenOptions {
    /// Minify whitespace and remove comments.
    pub fn minify() -> Self {
        Self {
            single_quote: false,
            minify: true,
            comments: CommentOptions::disabled(),
            source_map_path: None,
            indent_char: IndentChar::default(),
            indent_width: DEFAULT_INDENT_WIDTH,
            initial_indent: 0,
        }
    }

    #[inline]
    pub(crate) fn print_normal_comment(&self) -> bool {
        self.comments.normal
    }

    #[inline]
    pub(crate) fn print_legal_comment(&self) -> bool {
        self.comments.legal.is_inline()
    }

    #[inline]
    pub(crate) fn print_jsdoc_comment(&self) -> bool {
        self.comments.jsdoc
    }

    #[inline]
    pub(crate) fn print_annotation_comment(&self) -> bool {
        self.comments.annotation
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Comment Options
pub struct CommentOptions {
    /// Print normal comments that do not have special meanings.
    ///
    /// At present only statement level comments are printed.
    ///
    /// Default is `true`.
    pub normal: bool,

    /// Print jsdoc comments.
    ///
    /// * jsdoc: `/** jsdoc */`
    ///
    /// Default is `true`.
    pub jsdoc: bool,

    /// Print annotation comments.
    ///
    /// * pure: `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`
    /// * webpack: `/* webpackChunkName */`
    /// * vite: `/* @vite-ignore */`
    /// * coverage: `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    ///
    /// Default is `true`.
    pub annotation: bool,

    /// Print legal comments.
    ///
    /// * starts with `//!` or `/*!`.
    /// * contains `/* @license */` or `/* @preserve */`
    ///
    /// Default is [`LegalComment::Inline`].
    pub legal: LegalComment,
}

impl Default for CommentOptions {
    fn default() -> Self {
        Self { normal: true, jsdoc: true, annotation: true, legal: LegalComment::default() }
    }
}

impl CommentOptions {
    /// Disable Comments.
    pub fn disabled() -> Self {
        Self { normal: false, jsdoc: false, annotation: false, legal: LegalComment::None }
    }
}

/// Legal comment
///
/// <https://esbuild.github.io/api/#legal-comments>
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum LegalComment {
    /// Do not preserve any legal comments.
    None,
    /// Preserve all legal comments (default).
    #[default]
    Inline,
    /// Move all legal comments to the end of the file.
    Eof,
    /// Return all legal comments and link then to them with a comment to the provided string.
    Linked(String),
    /// Move all legal comments to a .LEGAL.txt file but to not link to them.
    External,
}

impl LegalComment {
    /// Is None.
    pub fn is_none(&self) -> bool {
        *self == Self::None
    }

    /// Is inline mode.
    pub fn is_inline(&self) -> bool {
        *self == Self::Inline
    }

    /// Is EOF mode.
    pub fn is_eof(&self) -> bool {
        *self == Self::Eof
    }
}
