#![expect(clippy::missing_panics_doc, clippy::literal_string_with_formatting_args)]
pub mod comments;
pub mod esbuild;
pub mod js;
pub mod optimization_tests;
pub mod performance_tests;
pub mod sourcemap;
pub mod ts;

mod tester;

pub use tester::*;
