/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;

    fn new() -> Self;
}

/// Config for serializing AST with TypeScript fields,
pub struct ConfigTS;

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}

/// Config for serializing AST without TypeScript fields.
pub struct ConfigJS;

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
pub struct ConfigFixesTS;

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
pub struct ConfigFixesJS;

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }
}
