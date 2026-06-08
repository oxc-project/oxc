//! JavaScript Parsing Functions

mod grammar;

mod arrow;
mod binding;
mod class;
mod declaration;
mod expression;
mod function;
mod module;
mod object;
mod operator;
mod statement;

/// Classification of a `(`/`<`/`async (` head for parenthesized-arrow disambiguation.
///
/// A `( ... )` after which `=>` may follow is the ECMAScript *cover grammar*
/// `CoverParenthesizedExpressionAndArrowParameterList`, which is refined to either
/// `ArrowFormalParameters` (`=>` follows) or `ParenthesizedExpression` (it does not):
/// ```text
/// ArrowParameters[Yield, Await] :
///     BindingIdentifier[?Yield, ?Await]
///     CoverParenthesizedExpressionAndArrowParameterList[?Yield, ?Await]
///
/// // refined when `=>` follows:           // refined otherwise:
/// ArrowFormalParameters[Yield, Await] :   ParenthesizedExpression[Yield, Await] :
///     ( UniqueFormalParameters )              ( Expression[+In, ?Yield, ?Await] )
/// ```
/// <https://tc39.es/ecma262/#prod-CoverParenthesizedExpressionAndArrowParameterList>
///
/// `Cover` implements that refinement directly (parse the `( Expression )` once, then convert to
/// params iff `=>` follows). The other arms keep oxc's heuristic + speculate-and-rewind, because the
/// cover production's other alternatives — `( )`, `( ... BindingPattern )`,
/// `( Expression , ... BindingIdentifier )` — and TS/async extensions either have no
/// `ParenthesizedExpression` interpretation or need context the single expression-parse can't recover.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ArrowKind {
    /// Not an arrow (was `Tristate::False`). Fall through to expression parse.
    No,
    /// Definitely an arrow (was `Tristate::True`). Parse params directly.
    Yes,
    /// The cover production's `( Expression )` alternative, narrowed to a single identifier
    /// (`( a )`). Parse `( a )` once as an expression, then refine to `ArrowFormalParameters` iff
    /// `=>` follows, else keep it as a `ParenthesizedExpression`.
    Cover,
    /// A cover head with no `ParenthesizedExpression` interpretation or that is context-sensitive:
    /// `()`, `( ... BindingPattern )` (`(...[`), `( Expr , ... )` (any comma-shape `(a,` — a later
    /// `...rest` is invisible to the worker), `( a : T )`, generic/JSX `<...>`, and all `async (`.
    /// Keep the speculate-and-rewind path.
    Speculate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum FunctionKind {
    Constructor,
    ClassMethod,
    ObjectMethod,
    Declaration,
    Expression,
    DefaultExport,
    TSDeclaration,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum VariableDeclarationParent {
    For,
    Statement,
}
