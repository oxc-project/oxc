//! Generator for minifier-local traverse runtime.
//!
//! Generates 4 files in `oxc_minifier` crate:
//! * `traverse.rs` - `MinifierTraverse` trait with `enter_*` / `exit_*` methods.
//! * `walk.rs` - Unsafe `walk_*` functions for AST traversal.
//! * `ancestor.rs` - Ancestor tracking types and offset constants.
//! * `visit.rs` - JS-only `Visit` trait for the minifier's internal visitors.
//!
//! The minifier only operates on type-stripped ASTs, so unlike `oxc_traverse` / `oxc_ast_visit`,
//! its traverse runtime and `Visit` trait exclude TS type-level syntax entirely (no `*_ts_*`
//! methods, no TS walks, no TS `Ancestor` variants). TS nodes, if present, are not walked.

use super::traverse::{self, TraverseTraitConfig};
use crate::{
    Codegen, Generator, MINIFIER_CRATE_PATH,
    output::{Output, output_path},
    schema::Schema,
};

use super::define_generator;

pub struct MinifierTraverseGenerator;

define_generator!(MinifierTraverseGenerator);

impl Generator for MinifierTraverseGenerator {
    fn generate_many(&self, schema: &Schema, _codegen: &Codegen) -> Vec<Output> {
        let config = TraverseTraitConfig::minifier();
        vec![
            Output::Rust {
                path: output_path(MINIFIER_CRATE_PATH, "traverse.rs"),
                tokens: traverse::generate_traverse_trait(schema, &config),
            },
            Output::Rust {
                path: output_path(MINIFIER_CRATE_PATH, "walk.rs"),
                tokens: traverse::generate_walk_minifier(schema),
            },
            Output::Rust {
                path: output_path(MINIFIER_CRATE_PATH, "ancestor.rs"),
                tokens: traverse::generate_ancestor(schema, /* include_ts */ false),
            },
            Output::Rust {
                path: output_path(MINIFIER_CRATE_PATH, "visit.rs"),
                tokens: super::visit::generate_visit_js(schema),
            },
        ]
    }
}
