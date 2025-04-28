use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, Vec as ArenaVec};
use oxc_ast::ast::{AssignmentExpression, Decorator};
use oxc_span::Span;

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    pub decorators: ArenaVec<'a, Decorator<'a>>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression`.
    /// Valued by position of the trailing_comma.
    pub trailing_commas: FxHashMap<u32, Span>,
}

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            decorators: ArenaVec::new_in(allocator),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
        }
    }
}
