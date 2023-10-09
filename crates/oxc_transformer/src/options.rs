#[derive(Debug, Default, Clone)]
pub struct TransformOptions {
    pub target: TransformTarget,
    pub react: Option<TransformReactOptions>,
}

/// See <https://www.typescriptlang.org/tsconfig#target>
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum TransformTarget {
    ES5,
    ES2015,
    ES2016,
    ES2019,
    ES2021,
    ES2022,
    #[default]
    ESNext,
}

#[derive(Debug, Default, Clone)]
pub struct TransformReactOptions {
    _runtime: TransformReactRuntime,
}

#[derive(Debug, Default, Clone)]
pub enum TransformReactRuntime {
    #[default]
    Classic,
    Automatic,
}
