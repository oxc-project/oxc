use std::path::PathBuf;

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

    /// Print normal comments?
    ///
    /// At present, only some leading comments are preserved.
    ///
    /// Does not control legal and annotation comments.
    ///
    /// Default is `true`.
    pub comments: bool,

    /// Print annotation comments.
    ///
    /// * jsdoc: `/** jsdoc */`
    /// * pure: `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`
    /// * webpack: `/* webpackChunkName */`
    /// * vite: `/* @vite-ignore */`
    /// * coverage: `v8 ignore`, `c8 ignore`, `node:coverage`, `istanbul ignore`
    ///
    /// Default is `true`.
    pub annotation_comments: bool,

    /// Print legal comments.
    ///
    /// * starts with `//!` or `/*!`.
    /// * contains `/* @license */` or `/* @preserve */`
    ///
    /// Default is [LegalComment::Inline].
    pub legal_comments: LegalComment,

    /// Enable sourcemap.
    ///
    /// The provided path sets the `source` field in the returned sourcemap.
    ///
    /// Default is `None` - no sourcemap is produced.
    pub source_map_path: Option<PathBuf>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            single_quote: false,
            minify: false,
            comments: true,
            annotation_comments: true,
            legal_comments: LegalComment::Inline,
            source_map_path: None,
        }
    }
}

impl CodegenOptions {
    /// Minify whitespace and remove comments.
    pub fn minify() -> Self {
        Self {
            single_quote: false,
            minify: true,
            comments: false,
            annotation_comments: false,
            legal_comments: LegalComment::None,
            source_map_path: None,
        }
    }

    #[inline]
    pub(crate) fn print_normal_comment(&self) -> bool {
        self.comments
    }

    #[inline]
    pub(crate) fn print_legal_comment(&self) -> bool {
        self.legal_comments.is_inline()
    }

    #[inline]
    pub(crate) fn print_annotation_comment(&self) -> bool {
        self.annotation_comments
    }
}

/// Legal comment
///
/// <https://esbuild.github.io/api/#legal-comments>
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LegalComment {
    /// Do not preserve any legal comments.
    None,
    /// Preserve all legal comments (default).
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
