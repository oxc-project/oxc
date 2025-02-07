#[cfg(feature = "serialize")]
pub mod ser;
#[cfg(feature = "serialize")]
mod serialize;
#[cfg(feature = "serialize")]
pub use serialize::*;

/// Empty trait that will be used later for custom serialization and TypeScript
/// generation for AST nodes.
#[cfg(not(feature = "serialize"))]
pub trait ESTree {}
