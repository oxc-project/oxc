use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::Box;
use oxc_ast::ast::{AssignmentExpression, FormalParameterRest};
use oxc_span::Span;

use crate::cursor::ParserCheckpoint;

pub struct ParserState<'a> {
    pub not_parenthesized_arrow: FxHashSet<u32>,

    /// Temporary storage for `CoverInitializedName` `({ foo = bar })`.
    /// Keyed by `ObjectProperty`'s span.start.
    pub cover_initialized_name: FxHashMap<u32, AssignmentExpression<'a>>,

    /// Trailing comma spans for `ArrayExpression` and `ObjectExpression`.
    /// Used for error reporting.
    /// Keyed by start span of `ArrayExpression` / `ObjectExpression`.
    /// Valued by position of the trailing_comma.
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

    /// Set by the entry before parsing an `ArrowKind::Cover` head. Read-and-cleared by the next
    /// `parse_parenthesized_expression` (the cover paren itself): it switches that call to the cover
    /// path, which parses `( … )` once and — when an arrow `=>` follows the `)` — skips the
    /// `ParenthesizedExpression` wrapper that the arrow refinement would only discard.
    pub cover_paren_arrow: bool,

    /// `allow_return_type_in_arrow_function` for the pending cover head, threaded to the cover paren
    /// (which is reached deep in the precedence climb). Decides whether a `:` after `)` is a TS
    /// arrow return type (params-ok) or an enclosing conditional's `:` (`cond ? (a, b,) : c`, where
    /// the trailing comma is still an error). Set/cleared together with [`Self::cover_paren_arrow`].
    pub cover_paren_allow_return_type: bool,

    /// A `...rest` parsed at the top level of a cover head (`(a, ...rest) =>`). A rest element has
    /// no `Expression` form, so it cannot flow back up the precedence climb with the other
    /// elements; it is stashed here by the cover paren and recombined by `refine_arrow_params` (or
    /// reported as an error if the head turns out not to be an arrow).
    pub cover_paren_rest: Option<Box<'a, FormalParameterRest<'a>>>,

    /// True while parsing the elements of a cover head. A TS `ident?` followed by `:`/`,`/`)`/`=`
    /// is then read as an optional-parameter marker rather than the start of a conditional (which,
    /// having no consequent, would always be a syntax error). Saved/restored around nested cover
    /// heads. See `parse_conditional_expression_rest`.
    pub cover_paren_element: bool,
}

impl ParserState<'_> {
    pub fn new() -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
            cover_paren_arrow: false,
            cover_paren_allow_return_type: false,
            cover_paren_rest: None,
            cover_paren_element: false,
        }
    }
}
