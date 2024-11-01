use std::path::PathBuf;

/// Legal comment
///
/// <https://esbuild.github.io/api/#legal-comments>
#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub enum LegalComment {
    /// Do not preserve any legal comments (default).
    #[default]
    None,
    /// Preserve all legal comments.
    Inline,
    /// Move all legal comments to the end of the file.
    Eof,
    /// Move all legal comments to a .LEGAL.txt file and link to them with a comment.
    Linked,
    /// Move all legal comments to a .LEGAL.txt file but to not link to them.
    External,
}

impl LegalComment {
    /// Is inline mode.
    pub fn is_inline(self) -> bool {
        self == Self::Inline
    }
}

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

    /// Print all comments?
    ///
    /// Default is `true`.
    pub comments: bool,

    /// Print annotation comments, e.g. `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`.
    ///
    /// Only takes into effect when `comments` is false.
    ///
    /// Default is `false`.
    pub annotation_comments: bool,

    /// Print legal comments.
    ///
    /// Only takes into effect when `comments` is false.
    ///
    /// <https://esbuild.github.io/api/#legal-comments>
    ///
    /// Default is [LegalComment::None].
    pub legal_comments: LegalComment,

    /// Override the source map path. This affects the `sourceMappingURL`
    /// comment at the end of the generated code.
    ///
    /// By default, the source map path is the same as the input source code
    /// (with a `.map` extension).
    pub source_map_path: Option<PathBuf>,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        Self {
            single_quote: false,
            minify: false,
            comments: true,
            annotation_comments: false,
            legal_comments: LegalComment::default(),
            source_map_path: None,
        }
    }
}

impl CodegenOptions {
    pub(crate) fn print_comments(&self) -> bool {
        !self.minify
            && (self.comments || self.annotation_comments || self.legal_comments.is_inline())
    }

    pub(crate) fn print_annotation_comments(&self) -> bool {
        !self.minify && (self.comments || self.annotation_comments)
    }
}
