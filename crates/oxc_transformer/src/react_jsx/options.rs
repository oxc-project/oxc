use std::borrow::Cow;

use oxc_semantic::Semantic;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxOptions {
    /// Decides which runtime to use.
    #[serde(default)]
    pub runtime: ReactJsxRuntime,
    /// Toggles whether or not to throw an error if an XML namespaced tag name is used. e.g. `<f:image />`
    /// Though the JSX spec allows this, it is disabled by default since React's JSX does not currently have support for it.
    #[serde(default = "default_throw_if_namespace")]
    pub throw_if_namespace: bool,
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
    /// Replace the component used when compiling JSX fragments. It should be a valid JSX tag name. default to `React.Fragment`
    #[serde(default = "default_pragma_frag")]
    pub pragma_frag: Cow<'static, str>,

    /// When spreading props, use inline object with spread elements directly instead of Babel's extend helper or Object.assign.
    pub use_built_ins: Option<bool>,
}

fn default_throw_if_namespace() -> bool {
    true
}

fn default_import_source() -> Cow<'static, str> {
    Cow::Borrowed("react")
}

fn default_pragma() -> Cow<'static, str> {
    Cow::Borrowed("React.createElement")
}

fn default_pragma_frag() -> Cow<'static, str> {
    Cow::Borrowed("React.Fragment")
}

impl Default for ReactJsxOptions {
    fn default() -> Self {
        Self {
            runtime: ReactJsxRuntime::Automatic,
            throw_if_namespace: default_throw_if_namespace(),
            import_source: default_import_source(),
            pragma: default_pragma(),
            pragma_frag: default_pragma_frag(),
            use_built_ins: None,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReactJsxRuntime {
    /// Does not automatically import anything.
    Classic,
    /// Auto imports the functions that JSX transpiles to (default).
    #[default]
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
            let mut comment = span.source_text(semantic.source_text()).trim_start();
            // strip leading jsdoc comment `*` and then whitespaces
            while let Some(cur_comment) = comment.strip_prefix('*') {
                comment = cur_comment.trim_start();
            }
            // strip leading `@`
            let Some(comment) = comment.strip_prefix('@') else { continue };

            // read jsxRuntime
            match comment.strip_prefix("jsxRuntime").map(str::trim) {
                Some("classic") => {
                    self.runtime = ReactJsxRuntime::Classic;
                    continue;
                }
                Some("automatic") => {
                    self.runtime = ReactJsxRuntime::Automatic;
                    continue;
                }
                _ => {}
            }

            // read jsxImportSource
            if let Some(import_source) = comment.strip_prefix("jsxImportSource").map(str::trim) {
                self.import_source = Cow::from(import_source.to_string());
                continue;
            }

            // read jsxFrag
            if let Some(pragma_frag) = comment.strip_prefix("jsxFrag").map(str::trim) {
                self.pragma_frag = Cow::from(pragma_frag.to_string());
                continue;
            }

            // Put this condition at the end to avoid breaking @jsxXX
            // read jsx
            if let Some(pragma) = comment.strip_prefix("jsx").map(str::trim) {
                self.pragma = Cow::from(pragma.to_string());
            }
        }
        self
    }
}
