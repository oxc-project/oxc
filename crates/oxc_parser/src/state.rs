use rustc_hash::{FxHashMap, FxHashSet};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_ast::ast::Expression;
use oxc_span::Span;

use crate::{cursor::ParserCheckpoint, js::CoverElement};

/// The initializer and diagnostic span of a shorthand property awaiting cover validation.
pub struct CoverInitializedName<'a> {
    pub span: Span,
    pub initializer: Expression<'a>,
}

pub struct ParserState<'a> {
    /// Positions of `(` tokens whose `( … ) : Type =>` continuation was rejected as an arrow
    /// function, so that enclosing speculation does not try the same return type twice.
    pub not_parenthesized_arrow: FxHashSet<u32>,

    /// Elements of the `CoverParenthesizedExpressionAndArrowParameterList`s being parsed.
    ///
    /// A stack shared by nested cover lists: each list owns the elements above the length it
    /// recorded when it started, and removes them once the token after `)` decides whether
    /// they are arrow function parameters or a parenthesized expression.
    ///
    /// Kept in the arena, like `cover_parenthesized`, so that parsing a small file does not
    /// pay for heap allocations.
    pub cover_elements: ArenaVec<'a, CoverElement<'a>>,

    /// Whether an identifier named `await` was checked where `await` is not reserved.
    /// Tells whether the arguments of `async(…)` have to be parsed again as parameters.
    pub mentions_await: bool,

    /// Number of cover lists whose elements are currently being parsed.
    pub cover_depth: u32,

    /// Start offsets of the expressions directly inside parentheses that were parsed while an
    /// enclosing cover list was still undecided, in source order.
    ///
    /// `([(a)]) => 0` and `([(a) = 1]) => 0` are Syntax Errors, but the parentheses do not
    /// survive in the AST: `ParenthesizedExpression` is omitted without `preserve_parens`,
    /// and assignment targets never keep it.
    /// Like `cover_elements`, each cover list owns the entries above its recorded length.
    pub cover_parenthesized: ArenaVec<'a, u32>,

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

impl<'a> ParserState<'a> {
    pub fn new(allocator: &'a Allocator) -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_elements: ArenaVec::new_in(&allocator),
            mentions_await: false,
            cover_depth: 0,
            cover_parenthesized: ArenaVec::new_in(&allocator),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
        }
    }
}
