use std::borrow::Cow;

use oxc_semantic::Semantic;
use serde::Deserialize;

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxOptions {
    /// Decides which runtime to use.
    pub runtime: ReactJsxRuntime,
    /// Toggles whether or not to throw an error if an XML namespaced tag name is used. e.g. `<f:image />`
    /// Though the JSX spec allows this, it is disabled by default since React's JSX does not currently have support for it.
    pub throw_if_namespace: Option<bool>,
    /// Replaces the import source when importing functions. default to `react`
    #[serde(default = "default_import_source")]
    pub import_source: Cow<'static, str>,
    /// Replace the function used when compiling JSX expressions.
    /// It should be a qualified name (e.g. React.createElement) or an identifier (e.g. createElement).
    /// default to `React.createElement`
    ///
    /// Note that the @jsx React.DOM pragma has been deprecated as of React v0.12
    #[serde(default = "default_pragma")]
    pub pragma: Cow<'static, str>,
}

fn default_import_source() -> Cow<'static, str> {
    Cow::Borrowed("react")
}

fn default_pragma() -> Cow<'static, str> {
    Cow::Borrowed("React.createElement")
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReactJsxRuntime {
    /// Does not automatically import anything (default).
    #[default]
    Classic,
    /// Auto imports the functions that JSX transpiles to.
    Automatic,
}

impl ReactJsxRuntime {
    pub fn is_classic(&self) -> bool {
        matches!(self, Self::Classic)
    }

    pub fn is_automatic(&self) -> bool {
        matches!(self, Self::Automatic)
    }
}

impl ReactJsxOptions {
    /// Scan through all comments and find the following pragmas
    ///
    /// * @jsxRuntime classic / automatic
    ///
    /// The comment does not need to be a jsdoc,
    /// otherwise `JSDoc` could be used instead.
    ///
    /// This behavior is aligned with babel.
    pub(crate) fn with_comments(mut self, semantic: &Semantic) -> Self {
        for (_, span) in semantic.trivias().comments_spans() {
            let comment = span.source_text(semantic.source_text());
            // strip leading jsdoc comment `*` and then whitespaces
            let comment = comment.strip_prefix('*').unwrap_or(comment).trim_start();
            // strip leading `@`
            let Some(comment) = comment.strip_prefix('@') else { continue };

            // read jsxRuntime
            match comment.strip_prefix("jsxRuntime").map(str::trim) {
                Some("classic") => self.runtime = ReactJsxRuntime::Classic,
                Some("automatic") => self.runtime = ReactJsxRuntime::Automatic,
                _ => {}
            }

            // read jsxImportSource
            if let Some(import_source) = comment.strip_prefix("jsxImportSource").map(str::trim) {
                self.import_source = Cow::from(import_source.to_string());
            }
        }
        self
    }
}
