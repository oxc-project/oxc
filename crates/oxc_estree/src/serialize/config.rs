/// Trait for configs for AST serialization.
pub trait Config: Default {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Config for serializing AST with TypeScript fields.
#[derive(Default)]
#[repr(transparent)]
pub struct ConfigTS {
    ranges: bool,
}

impl ConfigTS {
    #[inline(always)]
    pub fn new(ranges: bool) -> Self {
        Self { ranges }
    }
}

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST without TypeScript fields.
#[derive(Default)]
#[repr(transparent)]
pub struct ConfigJS {
    ranges: bool,
}

impl ConfigJS {
    #[inline(always)]
    pub fn new(ranges: bool) -> Self {
        Self { ranges }
    }
}

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
#[derive(Default)]
#[repr(transparent)]
pub struct ConfigFixesTS {
    ranges: bool,
}

impl ConfigFixesTS {
    #[inline(always)]
    pub fn new(ranges: bool) -> Self {
        Self { ranges }
    }
}

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
#[derive(Default)]
#[repr(transparent)]
pub struct ConfigFixesJS {
    ranges: bool,
}

impl ConfigFixesJS {
    #[inline(always)]
    pub fn new(ranges: bool) -> Self {
        Self { ranges }
    }
}

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }
}
