/// Trait for configs for AST serialization.
pub trait Config {
    fn new(ranges: bool) -> Self;

    /// `true` if output should contain TS fields.
    fn include_ts_fields(&self) -> bool;

    /// `true` if should record paths to `Literal` nodes that need fixing on JS side.
    fn fixes(&self) -> bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Config for serializing AST with TypeScript fields.
#[repr(transparent)]
pub struct ConfigTS {
    ranges: bool,
}

impl Config for ConfigTS {
    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        true
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        false
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
    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        false
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        false
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
    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        true
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        true
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
    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        false
    }

    #[inline(always)]
    fn fixes(&self) -> bool {
        true
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}
