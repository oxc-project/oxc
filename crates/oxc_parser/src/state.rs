use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression` and `ObjectExpression` ending in a spread.
    /// Used for error reporting when covering a rest assignment target.
    /// Keyed by the expression start and consumed when checked.
    pub trailing_commas: FxHashMap<u32, Span>,

    /// The unambiguous parse encountered an `await` identifier before discovering module syntax.
    pub needs_module_reparse: bool,

    /// Flag to track if an `await` identifier was encountered during statement parsing.
    /// Used to determine if a statement needs to be stored for potential reparsing
    /// in unambiguous mode.
    pub encountered_await_identifier: bool,
}

impl ParserState<'_> {
    pub fn new() -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            needs_module_reparse: false,
            encountered_await_identifier: false,
        }
    }
}
