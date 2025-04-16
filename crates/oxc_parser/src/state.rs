use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::{AssignmentExpression, Decorator};
use oxc_span::Span;

#[derive(Default)]
pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    pub decorators: Vec<Decorator<'a>>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression`.
    /// Valued by position of the trailing_comma.
    pub trailing_commas: FxHashMap<u32, Span>,
}
