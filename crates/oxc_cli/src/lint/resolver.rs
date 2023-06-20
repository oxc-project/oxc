use std::ops::Deref;

pub use nodejs_resolver::ResolveResult;
use nodejs_resolver::{EnforceExtension, Options, Resolver as NodeJSResolver};

#[derive(Debug)]
pub struct Resolver(NodeJSResolver);

impl Default for Resolver {
    fn default() -> Self {
        Self(NodeJSResolver::new(Options {
            enforce_extension: EnforceExtension::Enabled,
            extensions: [".js", ".mjs", ".cjs", ".jsx", ".ts", ".mts", ".cts", ".tsx"]
                .into_iter()
                .map(ToString::to_string)
                .collect(),
            ..Default::default()
        }))
    }
}

impl Deref for Resolver {
    type Target = NodeJSResolver;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
