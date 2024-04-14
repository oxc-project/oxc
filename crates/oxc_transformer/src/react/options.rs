use std::borrow::Cow;

use serde::Deserialize;

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
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

impl Default for ReactOptions {
    fn default() -> Self {
        Self {
            jsx_plugin: true,
            display_name_plugin: true,
            jsx_self_plugin: true,
            jsx_source_plugin: true,
            runtime: ReactJsxRuntime::default(),
            development: default_as_true(),
            pure: default_as_true(),
            import_source: default_for_import_source(),
            pragma: default_for_pragma(),
            pragma_frag: default_for_pragma_frag(),
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
