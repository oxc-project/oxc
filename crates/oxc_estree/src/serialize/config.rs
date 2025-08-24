/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;

    fn new(ranges: bool) -> Self;
    fn new_with_loc(ranges: bool, loc: bool) -> Self;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;

    /// Get whether output should contain `loc` fields.
    fn loc(&self) -> bool;
}

/// Config for serializing AST with TypeScript fields.
pub struct ConfigTS {
    ranges: bool,
    loc: bool,
}

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges, loc: false }
    }

    #[inline(always)]
    fn new_with_loc(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }
}

/// Config for serializing AST without TypeScript fields.
pub struct ConfigJS {
    ranges: bool,
    loc: bool,
}

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges, loc: false }
    }

    #[inline(always)]
    fn new_with_loc(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
pub struct ConfigFixesTS {
    ranges: bool,
    loc: bool,
}

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges, loc: false }
    }

    #[inline(always)]
    fn new_with_loc(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
pub struct ConfigFixesJS {
    ranges: bool,
    loc: bool,
}

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;

    #[inline(always)]
    fn new(ranges: bool) -> Self {
        Self { ranges, loc: false }
    }

    #[inline(always)]
    fn new_with_loc(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }
}
