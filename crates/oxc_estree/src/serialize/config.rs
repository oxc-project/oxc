/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;
    /// `true` if `range` field should be emitted
    const RANGES: bool;

    fn new() -> Self;

    /// Whether to include range information in the serialized output (runtime helper)
    #[inline(always)]
    fn ranges(&self) -> bool {
        Self::RANGES
    }
}

/// Config for serializing AST with TypeScript fields,
pub struct ConfigTS;

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;
    const RANGES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        false
    }
}

/// Config for serializing AST with TypeScript fields, with ranges.
pub struct ConfigTSWithRanges;

impl Config for ConfigTSWithRanges {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;
    const RANGES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        true
    }
}

/// Config for serializing AST without TypeScript fields.
pub struct ConfigJS;

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;
    const RANGES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        false
    }
}

/// Config for serializing AST without TypeScript fields, with ranges.
pub struct ConfigJSWithRanges;

impl Config for ConfigJSWithRanges {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;
    const RANGES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        true
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
pub struct ConfigFixesTS;

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;
    const RANGES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        false
    }
}

/// Config for serializing AST with TypeScript fields, with fixes, with ranges.
pub struct ConfigFixesTSWithRanges;

impl Config for ConfigFixesTSWithRanges {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;
    const RANGES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        true
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
pub struct ConfigFixesJS;

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;
    const RANGES: bool = false;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        false
    }
}

/// Config for serializing AST without TypeScript fields, with fixes, with ranges.
pub struct ConfigFixesJSWithRanges;

impl Config for ConfigFixesJSWithRanges {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;
    const RANGES: bool = true;

    #[inline(always)]
    fn new() -> Self {
        Self
    }

    #[inline]
    fn ranges(&self) -> bool {
        true
    }
}
