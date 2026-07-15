use std::path::PathBuf;

use napi::Either;
use napi_derive::napi;

use oxc_codegen::IndentChar;

#[napi(string_enum = "lowercase")]
#[derive(Debug)]
pub enum LegalCommentsMode {
    None,
    Inline,
    Eof,
}

#[napi(object)]
#[derive(Debug, Default)]
pub struct CommentOptions {
    /// Print normal comments that do not have special meanings.
    ///
    /// At present only statement level comments are printed.
    ///
    /// @default true
    pub normal: Option<bool>,

    /// Print jsdoc comments (`/** jsdoc */`).
    ///
    /// @default true
    pub jsdoc: Option<bool>,

    /// Print annotation comments, e.g. `/* #__PURE__ */`, `/* webpackChunkName */`,
    /// `/* @vite-ignore */` and coverage ignore comments.
    ///
    /// @default true
    pub annotation: Option<bool>,

    /// How to handle legal comments (comments containing `@license`, `@preserve`,
    /// or starting with `//!` / `/*!`).
    ///
    /// * `"none"` - Do not preserve any legal comments.
    /// * `"inline"` - Preserve all legal comments inline.
    /// * `"eof"` - Move all legal comments to the end of the file.
    ///
    /// @default "inline"
    #[napi(ts_type = "'none' | 'inline' | 'eof'")]
    pub legal: Option<LegalCommentsMode>,
}

impl From<&CommentOptions> for oxc_codegen::CommentOptions {
    fn from(o: &CommentOptions) -> Self {
        let default = oxc_codegen::CommentOptions::default();
        oxc_codegen::CommentOptions {
            normal: o.normal.unwrap_or(default.normal),
            jsdoc: o.jsdoc.unwrap_or(default.jsdoc),
            annotation: o.annotation.unwrap_or(default.annotation),
            legal: match o.legal {
                None => default.legal,
                Some(LegalCommentsMode::None) => oxc_codegen::LegalComment::None,
                Some(LegalCommentsMode::Inline) => oxc_codegen::LegalComment::Inline,
                Some(LegalCommentsMode::Eof) => oxc_codegen::LegalComment::Eof,
            },
        }
    }
}

#[napi(string_enum = "lowercase")]
#[derive(Debug)]
pub enum IndentCharKind {
    Space,
    Tab,
}

#[napi(object)]
#[derive(Debug, Default)]
pub struct PrintOptions {
    /// Original source text the AST was parsed from.
    ///
    /// Required for printing comments (oxc comments reference source spans) and for
    /// accurate source maps. When omitted, comments are not printed and span-derived
    /// output is best-effort.
    pub source_text: Option<String>,

    /// Source filename, used as the `source` field of the source map.
    ///
    /// @default "unknown"
    pub filename: Option<String>,

    /// Produce a source map, returned as `map` on the result.
    ///
    /// @default false
    pub sourcemap: Option<bool>,

    /// Use single quotes instead of double quotes.
    ///
    /// @default false
    pub single_quote: Option<bool>,

    /// Remove whitespace (minified output). Comments are removed unless `comments`
    /// is set explicitly.
    ///
    /// @default false
    pub minify: Option<bool>,

    /// Print comments. Requires `sourceText`.
    ///
    /// `false` disables all comments, `true` or an object enables them selectively.
    ///
    /// @default true (when `sourceText` is provided and not minifying)
    pub comments: Option<Either<bool, CommentOptions>>,

    /// Indentation character.
    ///
    /// @default "tab"
    #[napi(ts_type = "'space' | 'tab'")]
    pub indent_char: Option<IndentCharKind>,

    /// Number of characters per indentation level.
    ///
    /// @default 1
    pub indent_width: Option<u32>,

    /// Initial indentation level for the generated code.
    ///
    /// @default 0
    pub initial_indent: Option<u32>,
}

impl PrintOptions {
    /// Convert N-API print options into codegen options.
    pub fn to_codegen_options(&self) -> oxc_codegen::CodegenOptions {
        let minify = self.minify == Some(true);
        let mut opts = if minify {
            oxc_codegen::CodegenOptions::minify()
        } else {
            oxc_codegen::CodegenOptions::default()
        };

        opts.single_quote = self.single_quote.unwrap_or(opts.single_quote);

        opts.comments = match &self.comments {
            Some(Either::A(false)) => oxc_codegen::CommentOptions::disabled(),
            Some(Either::A(true)) => oxc_codegen::CommentOptions::default(),
            Some(Either::B(o)) => oxc_codegen::CommentOptions::from(o),
            // Comments require source text to resolve their spans.
            None if self.source_text.is_none() => oxc_codegen::CommentOptions::disabled(),
            None => opts.comments,
        };

        if self.sourcemap == Some(true) {
            let filename = self.filename.as_deref().unwrap_or("unknown");
            opts.source_map_path = Some(PathBuf::from(filename));
        }

        opts.indent_char = match self.indent_char {
            Some(IndentCharKind::Space) => IndentChar::Space,
            Some(IndentCharKind::Tab) => IndentChar::Tab,
            None => opts.indent_char,
        };
        if let Some(width) = self.indent_width {
            opts.indent_width = width as usize;
        }
        if let Some(initial) = self.initial_indent {
            opts.initial_indent = initial;
        }

        opts
    }
}
