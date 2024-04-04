use serde::Deserialize;

pub use crate::decorators::DecoratorsOptions;
pub use crate::es2020::Es2020Options;
pub use crate::es2021::Es2021Options;
pub use crate::es2022::Es2022Options;
pub use crate::es2024::Es2024Options;
pub use crate::typescript::TypeScriptOptions;

#[derive(Debug, Default, Clone, Deserialize)]
pub struct TransformerOptions {
    // Specs
    pub es2020: Es2020Options,
    pub es2021: Es2021Options,
    pub es2022: Es2022Options,
    pub es2024: Es2024Options,
    // Ecosystem
    pub decorators: Option<DecoratorsOptions>,
    pub typescript: Option<TypeScriptOptions>,
}

#[inline]
pub fn default_as_true() -> bool {
    true
}
