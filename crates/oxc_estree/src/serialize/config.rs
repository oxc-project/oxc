/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;

    fn new(ranges: bool) -> Self;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Config for serializing AST with TypeScript fields.
#[repr(transparent)]
pub struct ConfigTS {
    ranges: bool,
}

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST without TypeScript fields.
#[repr(transparent)]
pub struct ConfigJS {
    ranges: bool,
}

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
#[repr(transparent)]
pub struct ConfigFixesTS {
    ranges: bool,
}

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
#[repr(transparent)]
pub struct ConfigFixesJS {
    ranges: bool,
}

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}
