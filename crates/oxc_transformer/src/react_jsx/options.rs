use std::borrow::Cow;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactJsxOptions {
    // Both Runtimes
    //
    /// Decides which runtime to use.
    pub runtime: ReactJsxRuntime,

    /// This toggles behavior specific to development, such as adding __source and __self.
    ///
    /// Defaults to `true`.
    #[serde(default = "default_as_true")]
    pub development: bool,

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
    //
    // `useBuiltIns` and `useSpread` are deprecated in babel 8.
}

impl Default for ReactJsxOptions {
    fn default() -> Self {
        Self {
            runtime: ReactJsxRuntime::default(),
            development: default_as_true(),
            pure: default_as_true(),
            import_source: default_for_import_source(),
            pragma: default_for_pragma(),
            pragma_frag: default_for_pragma_frag(),
        }
    }
}

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
#[derive(Debug, Default, Clone, Deserialize)]
pub enum ReactJsxRuntime {
    Classic,
    /// The default runtime is switched to automatic in Babel 8.
    #[default]
    Automatic,
}
