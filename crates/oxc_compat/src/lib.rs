//! Browser compatibility and feature detection for Oxc toolchain
//!
//! This crate provides utilities to determine which ECMAScript features
//! are supported by target engines, allowing tools like the transformer
//! and minifier to make consistent compatibility decisions.

mod babel_targets;
mod browserslist_query;
mod engine;
mod engine_targets;
mod es_features;
mod es_target;

pub use babel_targets::BabelTargets;
pub use browserslist_query::BrowserslistQuery;
pub use engine::Engine;
pub use engine_targets::{EngineTargets, Version};
pub use es_features::{ESFeature, features};
pub use es_target::ESVersion;
