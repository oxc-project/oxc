#[derive(Debug, Default, Clone, Copy)]
pub struct TransformOptions {
    pub target: TransformTarget,
    pub react: Option<TransformReactOptions>,
    pub assumptions: Assumptions,
}

/// See <https://www.typescriptlang.org/tsconfig#target>
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransformTarget {
    ES5,
    ES2015,
    ES2016,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ES2024,
    #[default]
    ESNext,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct TransformReactOptions {
    _runtime: TransformReactRuntime,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum TransformReactRuntime {
    #[default]
    Classic,
    Automatic,
}

/// Compiler assumptions
///
/// See <https://babeljs.io/docs/assumptions>
#[derive(Debug, Default, Clone, Copy)]
pub struct Assumptions {
    /// When using operators that check for null or undefined, assume that they are never used with the special value document.all.
    /// See <https://babeljs.io/docs/assumptions#nodocumentall>.
    pub no_document_all: bool,
}
