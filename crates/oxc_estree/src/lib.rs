#[cfg(feature = "serialize")]
mod serialize;
#[cfg(feature = "serialize")]
pub use serialize::*;

/// Placeholder for real `ESTree` trait when `serialize` feature not enabled.
///
/// Provided to support `#[generate_derive(ESTree)]`, without enabling the feature.
#[cfg(not(feature = "serialize"))]
pub trait ESTree {}
