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

    /// Print comments?
    ///
    /// Default is `true`.
    pub comments: bool,

    /// Print annotation comments, e.g. `/* #__PURE__ */` and `/* #__NO_SIDE_EFFECTS__ */`.
    ///
    /// Only takes into effect when `comments` is false.
    ///
    /// Default is `false`.
    pub annotation_comments: bool,

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
            source_map_path: None,
        }
    }
}

impl CodegenOptions {
    pub(crate) fn print_annotation_comments(&self) -> bool {
        !self.minify && (self.comments || self.annotation_comments)
    }
}
