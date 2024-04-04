use std::borrow::Cow;

use serde::Deserialize;

pub use crate::compiler_assumptions::CompilerAssumptions;
pub use crate::decorators::DecoratorsOptions;
pub use crate::es2020::Es2020Options;
pub use crate::es2021::Es2021Options;
pub use crate::es2022::Es2022Options;
pub use crate::es2024::Es2024Options;
pub use crate::react::ReactOptions;
pub use crate::typescript::TypeScriptOptions;

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

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TransformOptions {
    // Core
    pub assumptions: CompilerAssumptions,

    // Specs
    pub es2020: Es2020Options,
    pub es2021: Es2021Options,
    pub es2022: Es2022Options,
    pub es2024: Es2024Options,

    // Ecosystem
    pub decorators: Option<DecoratorsOptions>,
    pub jsx: Option<JsxOptions>,
    pub react: Option<ReactOptions>,
    pub typescript: Option<TypeScriptOptions>,
}

#[inline]
pub fn default_as_true() -> bool {
    true
}
