/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;

    fn new() -> Self;
}

/// Config for serializing AST with TypeScript fields,
pub struct ConfigTS;

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}

/// Config for serializing AST without TypeScript fields.
pub struct ConfigJS;

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}
