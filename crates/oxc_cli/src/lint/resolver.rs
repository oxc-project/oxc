use std::ops::Deref;

use nodejs_resolver::{EnforceExtension, Options, Resolver as NodeJSResolver};
pub use nodejs_resolver::{ResolveResult, Resource};
use oxc_span::VALID_EXTENSIONS;

#[derive(Debug)]
pub struct Resolver(NodeJSResolver);

impl Default for Resolver {
    fn default() -> Self {
        Self(NodeJSResolver::new(Options {
            enforce_extension: EnforceExtension::Enabled,
            extensions: VALID_EXTENSIONS.into_iter().map(|ext| String::from(".") + ext).collect(),
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
