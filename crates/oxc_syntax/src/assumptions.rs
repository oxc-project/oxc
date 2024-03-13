use oxc_macros::SerAttrs;

#[cfg(feature = "serde")]
use serde::Deserialize;

/// Compiler assumptions
///
/// See <https://babeljs.io/docs/assumptions>
#[derive(Debug, Default, Clone, Copy, SerAttrs)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct CompilerAssumptions {
    /// When using operators that check for null or undefined, assume that they are never used with the special value document.all.
    /// See <https://babeljs.io/docs/assumptions#nodocumentall>.
    #[serde(default)]
    pub no_document_all: bool,
}
