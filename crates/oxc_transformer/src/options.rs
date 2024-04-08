use std::borrow::Cow;

use serde::Deserialize;

pub use crate::{
    compiler_assumptions::CompilerAssumptions, decorators::DecoratorsOptions, react::ReactOptions,
    typescript::TypeScriptOptions,
};

#[inline]
pub fn default_as_true() -> bool {
    true
}

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TransformOptions {
    // Core
    pub assumptions: CompilerAssumptions,
    pub target: TransformTarget,

    // Ecosystem
    pub decorators: Option<DecoratorsOptions>,
    pub jsx: Option<JsxOptions>,
    pub react: Option<ReactOptions>,
    pub typescript: Option<TypeScriptOptions>,
}

impl TransformOptions {
    pub fn validate(&mut self) {
        if self.jsx.is_none() && (self.react.is_some() || self.typescript.is_some()) {
            self.jsx = Some(JsxOptions::default());
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransformTarget {
    // ES3,
    // ES5,
    // ES2015,
    // ES2016,
    // ES2017,
    // ES2018,
    // ES2019,
    // ES2020,
    // ES2021,
    // ES2022,
    // ES2024,
    #[default]
    ESNext,
}

/// This is used by React, TypeScript, Solid, and anything else.
/// Instead of duplicating these fields in each transform/preset,
/// it standardizes it here.
#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct JsxOptions {
    #[serde(default = "default_for_import_source")]
    pub import_source: Cow<'static, str>,

    #[serde(default = "default_for_pragma")]
    pub pragma: Cow<'static, str>,

    #[serde(default = "default_for_pragma_frag")]
    pub pragma_fragment: Cow<'static, str>,
}

impl Default for JsxOptions {
    fn default() -> Self {
        Self {
            import_source: default_for_import_source(),
            pragma: default_for_pragma(),
            pragma_fragment: default_for_pragma_frag(),
        }
    }
}

fn default_for_import_source() -> Cow<'static, str> {
    Cow::Borrowed("react")
}

fn default_for_pragma() -> Cow<'static, str> {
    Cow::Borrowed("React.createElement")
}

fn default_for_pragma_frag() -> Cow<'static, str> {
    Cow::Borrowed("React.Fragment")
}
