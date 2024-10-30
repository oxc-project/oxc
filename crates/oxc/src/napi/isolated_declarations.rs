use napi_derive::napi;

use super::source_map::SourceMap;

#[napi(object)]
pub struct IsolatedDeclarationsResult {
    pub code: String,
    pub map: Option<SourceMap>,
    pub errors: Vec<String>,
}

#[napi(object)]
#[derive(Debug, Default, Clone, Copy)]
pub struct IsolatedDeclarationsOptions {
    /// Do not emit declarations for code that has an @internal annotation in its JSDoc comment.
    /// This is an internal compiler option; use at your own risk, because the compiler does not check that the result is valid.
    ///
    /// Default: `false`
    ///
    /// See <https://www.typescriptlang.org/tsconfig/#stripInternal>
    pub strip_internal: Option<bool>,
}

impl From<IsolatedDeclarationsOptions> for oxc_isolated_declarations::IsolatedDeclarationsOptions {
    fn from(options: IsolatedDeclarationsOptions) -> Self {
        Self { strip_internal: options.strip_internal.unwrap_or_default() }
    }
}
