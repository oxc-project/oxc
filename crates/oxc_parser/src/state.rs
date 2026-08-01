use rustc_hash::{FxHashMap, FxHashSet};

use oxc_ast::ast::AssignmentExpression;
use oxc_span::Span;
use oxc_str::IdentHashSet;

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

    /// Nesting depth for contexts whose enclosing AST is ArkUI DSL.
    ///
    /// ETS is a TypeScript superset, so expression-level ArkUI syntax must not be
    /// enabled for every ETS token stream. This is set only while parsing bodies
    /// that are already known to be ArkUI DSL, such as `struct build()` methods,
    /// ArkUI-decorated functions, and component children.
    pub arkui_dsl_depth: u32,

    /// The next function or arrow function parsed is a configured ArkUI UI callback.
    /// The flag is consumed by the function body parser and restored by the argument parser.
    pub arkui_dsl_next_function: bool,

    /// Nesting depth of `parse_statement_list_item`. Static ETS uses this to
    /// distinguish a declaration directly in a source/namespace body from one
    /// nested in a block or control-flow statement.
    pub ets_statement_depth: u32,

    /// Statement depth at which the current source or namespace declaration
    /// list begins.
    pub ets_declaration_list_depth: Option<u32>,

    /// Whether the statement currently being parsed is a direct member of that
    /// declaration list.
    pub ets_in_declaration_scope: bool,

    /// The class/struct member currently being parsed is non-static and may
    /// therefore use `this` as its return type.
    pub ets_allow_this_return_type: bool,

    /// Active loop/switch nesting for static ETS break/continue validation.
    pub ets_loop_depth: u32,
    pub ets_switch_depth: u32,

    /// Type declarations seen so far. Static ETS does not allow a type object
    /// to be used as an ordinary runtime value.
    pub ets_type_names: IdentHashSet<'a>,
}

impl ParserState<'_> {
    pub fn new() -> Self {
        Self {
            not_parenthesized_arrow: FxHashSet::default(),
            cover_initialized_name: FxHashMap::default(),
            trailing_commas: FxHashMap::default(),
            potential_await_reparse: Vec::new(),
            encountered_await_identifier: false,
            arkui_dsl_depth: 0,
            arkui_dsl_next_function: false,
            ets_statement_depth: 0,
            ets_declaration_list_depth: None,
            ets_in_declaration_scope: false,
            ets_allow_this_return_type: false,
            ets_loop_depth: 0,
            ets_switch_depth: 0,
            ets_type_names: IdentHashSet::default(),
        }
    }
}
