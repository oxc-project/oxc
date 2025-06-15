pub mod serialize;

pub use serialize::{Position, SourceLocation};

/// Placeholder for real `ESTree` trait when `serialize` feature not enabled.
///
/// Provided to support `#[generate_derive(ESTree)]`, without enabling the feature.
#[cfg(not(feature = "serialize"))]
pub trait ESTree {}

// Re-export main serialize types for backward compatibility
#[cfg(feature = "serialize")]
pub use serialize::*;
