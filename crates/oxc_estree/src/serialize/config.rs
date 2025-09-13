use super::structs::{LocProvider, NoLocProvider};

/// Trait for configs for AST serialization.
pub trait Config {
    /// `true` if output should contain TS fields
    const INCLUDE_TS_FIELDS: bool;
    /// `true` if should record paths to `Literal` nodes that need fixing on JS side
    const FIXES: bool;

    /// Type of location provider this config uses
    type LocProvider: LocProvider;

    fn new(ranges: bool, loc: bool) -> Self;

    /// Create a new config with location provider for accurate loc fields
    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self;

    /// Get whether output should contain `range` fields.
    fn ranges(&self) -> bool;

    /// Get whether output should contain `loc` fields.
    fn loc(&self) -> bool;

    /// Get the location provider for translating offsets to line/column
    fn loc_provider(&self) -> &Self::LocProvider;
}

/// Config for serializing AST with TypeScript fields.
pub struct ConfigTS {
    ranges: bool,
    loc: bool,
    loc_provider: NoLocProvider,
}

impl Config for ConfigTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    type LocProvider = NoLocProvider;

    #[inline(always)]
    fn new(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc, loc_provider: NoLocProvider }
    }

    #[inline(always)]
    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }

    #[inline(always)]
    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}

/// Config for serializing AST without TypeScript fields.
pub struct ConfigJS {
    ranges: bool,
    loc: bool,
    loc_provider: NoLocProvider,
}

impl Config for ConfigJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    type LocProvider = NoLocProvider;

    #[inline(always)]
    fn new(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc, loc_provider: NoLocProvider }
    }

    #[inline(always)]
    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }

    #[inline(always)]
    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}

/// Config for serializing AST with TypeScript fields, with fixes.
pub struct ConfigFixesTS {
    ranges: bool,
    loc: bool,
    loc_provider: NoLocProvider,
}

impl Config for ConfigFixesTS {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = true;

    type LocProvider = NoLocProvider;

    #[inline(always)]
    fn new(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc, loc_provider: NoLocProvider }
    }

    #[inline(always)]
    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }

    #[inline(always)]
    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}

/// Config for serializing AST without TypeScript fields, with fixes.
pub struct ConfigFixesJS {
    ranges: bool,
    loc: bool,
    loc_provider: NoLocProvider,
}

impl Config for ConfigFixesJS {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = true;

    type LocProvider = NoLocProvider;

    #[inline(always)]
    fn new(ranges: bool, loc: bool) -> Self {
        Self { ranges, loc, loc_provider: NoLocProvider }
    }

    #[inline(always)]
    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    #[inline(always)]
    fn ranges(&self) -> bool {
        self.ranges
    }

    #[inline(always)]
    fn loc(&self) -> bool {
        self.loc
    }

    #[inline(always)]
    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}
