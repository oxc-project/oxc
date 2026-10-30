use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::Expression;
use oxc_span::Span;

use crate::cursor::ParserCheckpoint;

/// The initializer and diagnostic span of a shorthand property awaiting cover validation.
pub struct CoverInitializedName<'a> {
    pub span: Span,
    pub initializer: Expression<'a>,
}

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    pub cover_depth: u32,
    pub cover_parameter_start: Option<u32>,
    pub cover_parenthesized: FxHashSet<u32>,
    pub cover_async: bool,
    pub cover_async_await: bool,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, CoverInitializedName<'a>>,

    /// Trailing comma spans for `ArrayExpression` and `ObjectExpression` ending in a spread.
    /// Used for error reporting when covering a rest assignment target.
    /// Keyed by the expression start and consumed when checked.
    pub trailing_commas: FxHashMap<u32, Span>,

    /// Statements that may need reparsing when `sourceType` is `unambiguous`.
    ///
    /// In unambiguous mode, we initially parse top-level `await ...` as
    /// `await(...)` (identifier/function call). But if ESM syntax is detected
    /// later, we need to reparse these as await expressions.
    ///
    /// Each entry contains: (statement_index, checkpoint_before_statement)
    pub potential_await_reparse: Vec<(usize, ParserCheckpoint<'a>)>,

    /// Flag to track if an `await` identifier was encountered during statement parsing.
    /// Used to determine if a statement needs to be stored for potential reparsing
    /// in unambiguous mode.
    pub encountered_await_identifier: bool,
}

impl ParserState<'_> {
    pub fn discard_cover_state_from(&mut self, start: u32) {
        self.cover_initialized_name.retain(|position, _| *position < start);
        self.trailing_commas.retain(|position, _| *position < start);
        self.cover_parenthesized.retain(|position| *position < start);
    }

    pub fn new() -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_depth: 0,
            cover_parameter_start: None,
            cover_parenthesized: FxHashSet::default(),
            cover_async: false,
            cover_async_await: false,
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
        }
    }
}
