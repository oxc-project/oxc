use super::{config::Config, dynamic_loc_provider::DynamicLocProvider, structs::LocProvider};

/// Config for TypeScript serialization with dynamic location provider
pub struct ConfigTSWithLoc<P: LocProvider> {
    ranges: bool,
    loc: bool,
    loc_provider: P,
}

impl<P: LocProvider> Config for ConfigTSWithLoc<P> {
    const INCLUDE_TS_FIELDS: bool = true;
    const FIXES: bool = false;

    type LocProvider = P;

    fn new(_ranges: bool, _loc: bool) -> Self {
        // This shouldn't be called for specialized configs, but we need to implement it
        panic!("Use new_with_loc_provider for ConfigTSWithLoc")
    }

    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    fn ranges(&self) -> bool {
        self.ranges
    }

    fn loc(&self) -> bool {
        self.loc
    }

    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}

/// Config for JavaScript serialization with dynamic location provider
pub struct ConfigJSWithLoc<P: LocProvider> {
    ranges: bool,
    loc: bool,
    loc_provider: P,
}

impl<P: LocProvider> Config for ConfigJSWithLoc<P> {
    const INCLUDE_TS_FIELDS: bool = false;
    const FIXES: bool = false;

    type LocProvider = P;

    fn new(_ranges: bool, _loc: bool) -> Self {
        panic!("Use new_with_loc_provider for ConfigJSWithLoc")
    }

    fn new_with_loc_provider(ranges: bool, loc: bool, provider: Self::LocProvider) -> Self {
        Self { ranges, loc, loc_provider: provider }
    }

    fn ranges(&self) -> bool {
        self.ranges
    }

    fn loc(&self) -> bool {
        self.loc
    }

    fn loc_provider(&self) -> &Self::LocProvider {
        &self.loc_provider
    }
}

/// Type aliases for common configurations with dynamic location providers
pub type TSSerializerWithLoc<F, P> = super::ESTreeSerializer<ConfigTSWithLoc<P>, F>;
pub type JSSerializerWithLoc<F, P> = super::ESTreeSerializer<ConfigJSWithLoc<P>, F>;

/// Convenient type aliases for function-based location providers  
pub type TSSerializerWithFn<F, Fn> = TSSerializerWithLoc<F, DynamicLocProvider<Fn>>;
pub type JSSerializerWithFn<F, Fn> = JSSerializerWithLoc<F, DynamicLocProvider<Fn>>;
