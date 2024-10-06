use napi_derive::napi;

/// Babel Parser Options
///
/// <https://github.com/babel/babel/blob/main/packages/babel-parser/typings/babel-parser.d.ts>
#[napi(object)]
#[derive(Default)]
pub struct ParserOptions {
    #[napi(ts_type = "'script' | 'module' | 'unambiguous' | undefined")]
    pub source_type: Option<String>,
    pub source_filename: Option<String>,
    /// Emit `ParenthesizedExpression` in AST.
    ///
    /// If this option is true, parenthesized expressions are represented by
    /// (non-standard) `ParenthesizedExpression` nodes that have a single `expression` property
    /// containing the expression inside parentheses.
    ///
    /// Default: true
    pub preserve_parens: Option<bool>,
}

#[napi(object)]
pub struct ParseResult {
    pub program: String,
    pub comments: Vec<Comment>,
    pub errors: Vec<String>,
}

#[napi(object)]
pub struct Comment {
    #[napi(ts_type = "'Line' | 'Block'")]
    pub r#type: &'static str,
    pub value: String,
    pub start: u32,
    pub end: u32,
}
