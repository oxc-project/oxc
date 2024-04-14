use std::borrow::Cow;

use serde::Deserialize;

use crate::Ctx;

#[inline]
fn default_as_true() -> bool {
    true
}

#[inline]
fn default_for_import_source() -> Cow<'static, str> {
    Cow::Borrowed("react")
}

fn default_for_pragma() -> Cow<'static, str> {
    Cow::Borrowed("React.createElement")
}

fn default_for_pragma_frag() -> Cow<'static, str> {
    Cow::Borrowed("React.Fragment")
}

/// Decides which runtime to use.
///
/// Auto imports the functions that JSX transpiles to.
/// classic does not automatic import anything.
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReactJsxRuntime {
    Classic,
    /// The default runtime is switched to automatic in Babel 8.
    #[default]
    Automatic,
}

impl ReactJsxRuntime {
    pub fn is_classic(self) -> bool {
        self == Self::Classic
    }

    pub fn is_automatic(self) -> bool {
        self == Self::Automatic
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReactOptions {
    #[serde(skip)]
    pub jsx_plugin: bool,

    #[serde(skip)]
    pub display_name_plugin: bool,

    #[serde(skip)]
    pub jsx_self_plugin: bool,

    #[serde(skip)]
    pub jsx_source_plugin: bool,

    // Both Runtimes
    //
    /// Decides which runtime to use.
    pub runtime: ReactJsxRuntime,

    /// This toggles behavior specific to development, such as adding __source and __self.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_as_true")]
    pub development: bool,

    /// Toggles whether or not to throw an error if a XML namespaced tag name is used.
    ///
    /// Though the JSX spec allows this, it is disabled by default since React's JSX does not currently have support for it.
    #[serde(default = "default_as_true")]
    pub throw_if_namespace: bool,

    /// Enables `@babel/plugin-transform-react-pure-annotations`.
    ///
    /// It will mark top-level React method calls as pure for tree shaking.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_as_true")]
    pub pure: bool,
    //
    // React Automatic Runtime
    //
    /// Replaces the import source when importing functions.
    ///
    /// Defaults to `react`.
    #[serde(default = "default_for_import_source")]
    pub import_source: Cow<'static, str>,
    //
    // React Classic Runtime
    //
    /// Replace the function used when compiling JSX expressions.
    ///
    /// It should be a qualified name (e.g. React.createElement) or an identifier (e.g. createElement).
    ///
    /// Note that the @jsx React.DOM pragma has been deprecated as of React v0.12
    ///
    /// Defaults to `React.createElement`.
    #[serde(default = "default_for_pragma")]
    pub pragma: Cow<'static, str>,

    /// Replace the component used when compiling JSX fragments. It should be a valid JSX tag name.
    ///
    /// Defaults to `React.Fragment`.
    #[serde(default = "default_for_pragma_frag")]
    pub pragma_frag: Cow<'static, str>,

    /// `useBuiltIns` is deprecated in Babel 8.
    ///
    /// This value is used to skip Babel tests, and is not used in oxc.
    pub use_built_ins: Option<bool>,

    /// `useSpread` is deprecated in Babel 8.
    ///
    /// This value is used to skip Babel tests, and is not used in oxc.
    pub use_spread: Option<bool>,
}

impl Default for ReactOptions {
    fn default() -> Self {
        Self {
            jsx_plugin: true,
            display_name_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            runtime: ReactJsxRuntime::default(),
            development: default_as_true(),
            throw_if_namespace: default_as_true(),
            pure: default_as_true(),
            import_source: default_for_import_source(),
            pragma: default_for_pragma(),
            pragma_frag: default_for_pragma_frag(),
            use_built_ins: None,
            use_spread: None,
        }
    }
}

impl ReactOptions {
    pub fn is_jsx_self_plugin_enabled(&self) -> bool {
        self.jsx_self_plugin && self.development
    }

    pub fn is_jsx_source_plugin_enabled(&self) -> bool {
        self.jsx_source_plugin && self.development
    }

    /// Scan through all comments and find the following pragmas
    ///
    /// * @jsxRuntime classic / automatic
    ///
    /// The comment does not need to be a jsdoc,
    /// otherwise `JSDoc` could be used instead.
    ///
    /// This behavior is aligned with babel.
    pub(crate) fn update_with_comments(&mut self, ctx: &Ctx) {
        let semantic = &ctx.semantic;
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
    }
}
