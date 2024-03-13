use std::borrow::Cow;

use oxc_semantic::Semantic;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxOptions {
    /// Decides which runtime to use.
    pub runtime: Option<ReactJsxRuntimeOption>,
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

    /// When spreading props, use Object.assign directly instead of Babel's extend helper.
    /// Use `Some<T>` instead of `bool` because we want to know if user set this field explicitly,
    /// which used for creating warning, <https://github.com/oxc-project/oxc/blob/c3e2098c04d8916cb812bdd16d2026bb430ac25f/crates/oxc_transformer/src/react_jsx/mod.rs#L111-L114>
    pub use_built_ins: Option<bool>,
    /// When spreading props, use inline object with spread elements directly instead of Babel's extend helper or Object.assign.
    /// Use `Some<T>` instead of `bool` because we want to know if user set this field explicitly,
    /// which used for creating warning, <https://github.com/oxc-project/oxc/blob/c3e2098c04d8916cb812bdd16d2026bb430ac25f/crates/oxc_transformer/src/react_jsx/mod.rs#L111-L114>
    pub use_spread: Option<bool>,
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
            runtime: None,
            throw_if_namespace: default_throw_if_namespace(),
            import_source: default_import_source(),
            pragma: default_pragma(),
            pragma_frag: default_pragma_frag(),
            use_built_ins: None,
            use_spread: None,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReactJsxRuntimeOption {
    Valid(ReactJsxRuntime),
    // The order matters. The most permissive variant (i.e. the catch-all)
    // should be tried last during deserialization.
    Unknown(String),
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReactJsxRuntime {
    /// Does not automatically import anything.
    Classic,
    /// Auto imports the functions that JSX transpiles to (default).
    Automatic,
    /// Mixed, everything is like automatic but with `pragma` and `pragma_frag` support without imports.
    Mixed,
}

impl ReactJsxRuntime {
    pub fn is_classic(&self) -> bool {
        matches!(self, Self::Classic)
    }

    pub fn is_automatic(&self) -> bool {
        matches!(self, Self::Automatic)
    }

    pub fn is_mixed(&self) -> bool {
        matches!(self, Self::Mixed)
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
        for (_, span) in semantic.trivias().comments() {
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
                    self.runtime = Some(ReactJsxRuntimeOption::Valid(ReactJsxRuntime::Classic));
                    continue;
                }
                Some("automatic") => {
                    self.runtime = Some(ReactJsxRuntimeOption::Valid(ReactJsxRuntime::Automatic));
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
