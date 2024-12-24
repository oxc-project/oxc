use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::{AssignmentExpression, Decorator};

#[derive(Default)]
pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    pub decorators: Vec<Decorator<'a>>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,
}
