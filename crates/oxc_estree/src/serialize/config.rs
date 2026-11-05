/// Trait for configs for AST serialization.
pub trait Config {
    fn new(include_ts_fields: bool, ranges: bool) -> Self;

    /// `true` if output should contain TS fields.
    fn include_ts_fields(&self) -> bool;

    /// `true` if should record paths to `Literal` nodes that need fixing on JS side.
    fn fixes(&self) -> bool;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;
}

/// Config for serializing AST without fixes.
pub struct ConfigNoFixes {
    include_ts_fields: bool,
    ranges: bool,
}

impl Config for ConfigNoFixes {
    #[inline(always)]
    fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self { include_ts_fields, ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.include_ts_fields
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

/// Config for serializing AST with fixes.
pub struct ConfigFixes {
    include_ts_fields: bool,
    ranges: bool,
}

impl Config for ConfigFixes {
    #[inline(always)]
    fn new(include_ts_fields: bool, ranges: bool) -> Self {
        Self { include_ts_fields, ranges }
    }

    #[inline(always)]
    fn include_ts_fields(&self) -> bool {
        self.include_ts_fields
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
