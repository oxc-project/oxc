use oxc_semantic::{Reference, ReferenceId, ScopeId, SymbolId};
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use oxc_ast::{
    ast::{
        ArrayExpression, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
        ObjectPropertyKind,
    },
    AstKind, Visit,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::{is_method_call, leftmost_identifier_reference},
    context::LintContext,
    rule::Rule,
    utils::default_true,
    AstNode,
};

fn no_map_spread_diagnostic(map_call: Span, spread: &Spread<'_, '_>) -> OxcDiagnostic {
    let spans = spread.spread_spans();
    assert!(!spans.is_empty());
    let mut spread_labels = spread.spread_spans().into_iter(); // .map(LabeledSpan::from);
    let first_message = if spans.len() == 1 {
        "It should be mutated in place"
    } else {
        "They should be mutated in place"
    };
    let first = spread_labels.next().unwrap().label(first_message);
    let others = spread_labels.map(LabeledSpan::from);

    match spread {
        // Obj
        Spread::Object(_) => OxcDiagnostic::warn(
            "Spreading to modify object properties in `map` calls is inefficient",
        )
        .with_labels([map_call.label("This map call spreads an object"), first])
        .and_labels(others)
        .with_help("Consider using `Object.assign` instead"),
        // Array
        Spread::Array(_) => {
            OxcDiagnostic::warn("Spreading to modify array elements in `map` calls is inefficient")
                .with_labels([map_call.label("This map call spreads an array"), first])
                .and_labels(others)
                .with_help(
                    "Consider using `Array.prototype.concat` or `Array.prototype.push` instead",
                )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoMapSpreadConfig {
    /// Ignore mapped arrays that are re-read after the `map` call.
    ///
    /// Re-used arrays may rely on shallow copying behavior to avoid mutations.
    /// In these cases, `Object.assign` is not really more performant than spreads.
    ///
    /// Default: `true`
    #[serde(default = "default_true")]
    ignore_rereads: bool,
    // todo: ignore_arrays?
}

#[derive(Debug, Default, Clone)]
pub struct NoMapSpread(Box<NoMapSpreadConfig>);
impl Deref for NoMapSpread {
    type Target = NoMapSpreadConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NoMapSpreadConfig> for NoMapSpread {
    fn from(config: NoMapSpreadConfig) -> Self {
        Self(Box::new(config))
    }
}

impl Default for NoMapSpreadConfig {
    fn default() -> Self {
        Self { ignore_rereads: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of object or array spreads in `Array.prototype.map` and
    /// `Array.prototype.flatMap` to add properties/elements to array items.
    ///
    /// This rule only seeks to report cases where the spread operator is used
    /// to merge objects or arrays, not where it is used to copy them.
    ///
    /// ### Why is this bad?
    ///
    /// Spreading is commonly used to add properties to objects in an array or
    /// to combine several objects together. Unfortunately, spreads incur a
    /// re-allocation for a new object, plus `O(n)` memory copies.
    ///
    /// ```ts
    /// // each object in scores gets shallow-copied. Since `scores` is never
    /// // reused, spreading is inefficient.
    /// function getDisplayData() {
    ///     const scores: Array<{ username: string, score: number }> = getScores();
    ///     const displayData = scores.map(score => ({ ...score, rank: getRank(score) }));
    ///     return displayData
    /// }
    /// ```
    ///
    /// Unless you expect objects in the mapped array to be mutated later, it is
    /// better to use [`Object.assign`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign).
    ///
    /// ```ts
    /// // `score` is mutated in place and is more performant.
    /// function getDisplayData() {
    ///     const scores: Array<{ username: string, score: number }> = getScores();
    ///     const displayData = scores.map(score => Object.assign(score, { rank: getRank(score) }));
    ///     return displayData
    /// }
    /// ```
    ///
    /// ### Protecting from Mutations
    /// There are valid use cases for spreading objects in `map` calls,
    /// specifically when you want consumers of returned arrays to be able to
    /// mutate them without affecting the original data. This rule makes a
    /// best-effort attempt to avoid reporting on these cases.
    ///
    /// Spreads on class instance properties are completely ignored:
    /// ```ts
    /// class AuthorsDb {
    ///     #authors = [];
    ///     function getAuthorsWithBooks() {
    ///         return this.#authors.map(author => ({
    ///             // protects against mutations, giving the callee their own
    ///             // deep(ish) copy of the author object.
    ///             ...author,
    ///             books: getBooks(author)
    ///         }));
    ///     }
    /// }
    /// ```
    ///
    /// Spreads on arrays that are re-read after the `map` call are also ignored
    /// by default. Configure this behavior with the `ignoreRereads` option.
    ///
    /// ```
    /// /* "oxc/no-map-spread": ["error", { "ignoreRereads": true }] */
    /// const scores = getScores();
    /// const displayData = scores.map(score => ({ ...score, rank: getRank(score) }));
    /// console.log(scores); // scores is re-read after the map call
    /// ```
    ///
    /// #### Arrays
    ///
    /// In the case of array spreads,
    /// [`Array.prototype.concat`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/concat)
    /// or
    /// [`Array.prototype.push`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/push)
    /// should be used wherever possible. These have slignly different semantics
    /// than array spreads, since spreading works on iterables while `concat`
    /// and `push` work only on arrays.
    ///
    /// ```ts
    /// let arr = [1, 2, 3];
    /// let set = new Set([4])
    ///
    /// let a = [...arr, ...set]; // [1, 2, 3, 4]
    /// let b = arr.concat(set);  // [1, 2, 3, Set(1)]
    ///
    /// // Alternative that is more performant than spreading but still has the
    /// // same semantics. Unfortunately, it is more verbose.
    /// let c = arr.concat(Array.from(set)); // [1, 2, 3, 4]
    ///
    /// // You could also use `Symbol.isConcatSpreadable`
    /// set[Symbol.isConcatSpreadable] = true;
    /// let d = arr.concat(set); // [1, 2, 3, 4]
    /// ```
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// const arr = [{ a: 1 }, { a: 2 }, { a: 3 }];
    /// const arr2 = arr.map(obj => ({ ...obj, b: obj.a * 2 }));
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```ts
    /// const arr = [{ a: 1 }, { a: 2 }, { a: 3 }];
    /// arr.map(obj => Object.assign(obj, { b: obj.a * 2 }));
    ///
    /// // instance properties are ignored
    /// class UsersDb {
    ///   #users = [];
    ///   public get users() {
    ///     // clone users, providing caller with their own deep(ish) copy.
    ///     return this.#users.map(user => ({ ...user }));
    ///   }
    /// }
    /// ```
    ///
    /// ```tsx
    /// function UsersTable({ users }) {
    ///   const usersWithRoles = users.map(user => ({ ...user, role: getRole(user) }));
    ///
    ///   return (
    ///     <table>
    ///       {usersWithRoles.map(user => (
    ///         <tr>
    ///         <td>{user.name}</td>
    ///         <td>{user.role}</td>
    ///         </tr>
    ///       ))}
    ///       <tfoot>
    ///         <tr>
    ///           {/* re-read of users */}
    ///           <td>Total users: {users.length}</td>
    ///         </tr>
    ///       </tfoot>
    ///     </table>
    ///   )
    /// }
    /// ```
    ///
    /// ### References
    /// - [ECMA262 - Object spread evaluation semantics](https://262.ecma-international.org/15.0/index.html#sec-runtime-semantics-propertydefinitionevaluation)
    /// - [JSPerf - `concat` vs array spread performance](https://jsperf.app/pihevu)
    NoMapSpread,
    nursery, // TODO: make this `perf` once we've battle-tested this a bit
    pending  // TODO: dangerous_fix
);

const MAP_FN_NAMES: [&str; 2] = ["map", "flatMap"];

impl Rule for NoMapSpread {
    fn from_configuration(value: serde_json::Value) -> Self {
        let config: NoMapSpreadConfig = value
            .get(0)
            .map(|obj| {
                serde_json::from_value(obj.clone())
                    .expect("Invalid configuration for `oxc/no-map-spread`")
            })
            .unwrap_or_default();

        Self::from(config)
    }

    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        // Find `<expr>.map(<callback>)` calls.
        // look for both `map` and `flatMap`
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };
        let Some(mapper) = get_map_callback(call_expr) else {
            return;
        };

        // object and array literals with spread properties
        let mut spreads = Vec::new();
        // Look for return statements that contain an object or array spread.
        SpreadInReturnVisitor::<'a, '_>::iter_spreads(ctx, mapper, |spread| {
            // SAFETY: references to arena-allocated objects are valid for the
            // lifetime of the arena. Unfortunately, `AsRef` on `Box<'a, T>`
            // returns a reference with a lifetime of 'self instead of 'a.
            spreads.push(unsafe { std::mem::transmute::<Spread<'a, '_>, Spread<'a, 'a>>(spread) });
        });
        if spreads.is_empty() {
            return;
        }

        match leftmost_identifier_reference(&call_expr.callee) {
            Ok(ident) => {
                if let Some(ref_id) = ident.reference_id() {
                    if self.is_ignored_map_call(ctx, ident.name.as_str(), ref_id, call_expr.span) {
                        return;
                    }
                }
            }
            Err(Expression::ThisExpression(_)) => {
                return;
            }
            Err(_) => {}
        }

        let map_call_site = match &call_expr.callee {
            Expression::StaticMemberExpression(mem) => mem.property.span,
            Expression::ComputedMemberExpression(mem) => mem.expression.span(),
            Expression::PrivateFieldExpression(mem) => mem.field.span,
            Expression::ChainExpression(chain) => chain.expression.span(),
            expr => expr.span(),
        };

        for spread in spreads {
            ctx.diagnostic(no_map_spread_diagnostic(map_call_site, &spread));
        }
    }
}
impl NoMapSpread {
    fn is_ignored_map_call(
        &self,
        ctx: &LintContext<'_>,
        name: &str,
        reference_id: ReferenceId,
        call_site: Span,
    ) -> bool {
        let Some(symbol_id) = ctx.semantic().symbols().get_reference(reference_id).symbol_id()
        else {
            return false;
        };
        // Call is to a self-defined `map` call.
        if MAP_FN_NAMES.contains(&name) {
            // TODO: use symbolflags to check if function. Note that
            // SymbolFlags::ArrowFunction was removed.
            // TODO: ignore `const map = Array.prototype.map`
            return true;
        }

        self.ignore_rereads && has_reads_after(ctx, reference_id, symbol_id, call_site)
    }
}

/// Check if a symbol has any read references that occur after `span`. `span` is
/// the location of the reference identified by `reference_id`.
fn has_reads_after(
    ctx: &LintContext<'_>,
    reference_id: ReferenceId,
    symbol_id: SymbolId,
    span: Span,
) -> bool {
    let symbols = ctx.symbols();
    let nodes = ctx.nodes();
    symbols
        // skip the reference within the spread itself
        .get_resolved_reference_ids(symbol_id)
        .iter()
        .filter(|id| **id != reference_id)
        .map(|id| symbols.get_reference(*id))
        // we don't care if the symbol is overwritten
        .filter(|r| r.is_read())
        // Find where the symbol was declared
        .map(|r| nodes.get_node(r.node_id())) //
        .any(|node| node.span().end > span.end)
}

fn get_map_callback<'a, 'b>(call_expr: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    if !is_method_call(call_expr, None, Some(&MAP_FN_NAMES), Some(1), Some(1)) {
        return None;
    }

    let arg = call_expr.arguments.first()?.as_expression()?.get_inner_expression();
    match arg {
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => Some(arg),
        _ => None,
    }
}

enum Spread<'a, 'b> {
    Object(&'b ObjectExpression<'a>),
    Array(&'b ArrayExpression<'a>),
}
impl<'a, 'b> Spread<'a, 'b> {
    fn spread_spans(&self) -> Vec<Span> {
        match self {
            Spread::Object(obj) => obj
                .properties
                .iter()
                .filter_map(|prop| match prop {
                    ObjectPropertyKind::SpreadProperty(spread) => Some(spread.span()),
                    ObjectPropertyKind::ObjectProperty(_) => None,
                })
                .collect(),
            Spread::Array(arr) => arr
                .elements
                .iter()
                .filter_map(|elem| match elem {
                    ArrayExpressionElement::SpreadElement(spread) => Some(spread.span()),
                    _ => None,
                })
                .collect(),
        }
    }
}

impl GetSpan for Spread<'_, '_> {
    fn span(&self) -> Span {
        match self {
            Spread::Object(obj) => obj.span(),
            Spread::Array(arr) => arr.span(),
        }
    }
}

struct SpreadInReturnVisitor<'a, 'ctx, F> {
    cb: F,
    is_in_return: bool,
    cb_scope_id: ScopeId,
    ctx: &'ctx LintContext<'a>,
}

impl<'a, 'ctx, F> SpreadInReturnVisitor<'a, 'ctx, F>
where
    F: FnMut(Spread<'a, '_>),
{
    fn iter_spreads(ctx: &'ctx LintContext<'a>, map_cb: &Expression<'a>, cb: F) {
        let (mut visitor, body) = match map_cb {
            Expression::ArrowFunctionExpression(f) => {
                let v = Self {
                    ctx,
                    is_in_return: f.expression,
                    cb,
                    cb_scope_id: f.scope_id.get().unwrap(),
                };
                (v, f.body.as_ref())
            }
            Expression::FunctionExpression(f) => {
                let v =
                    Self { ctx, is_in_return: false, cb, cb_scope_id: f.scope_id.get().unwrap() };
                let Some(body) = f.body.as_ref().map(AsRef::as_ref) else {
                    return;
                };
                (v, body)
            }
            _ => unreachable!(),
        };

        visitor.visit_function_body(body);
    }

    fn visit_kind(&mut self, kind: AstKind<'a>) {
        match kind {
            AstKind::VariableDeclaration(d) => self.visit_variable_declaration(d),
            AstKind::VariableDeclarator(d) => self.visit_variable_declarator(d),
            _ => { /* not needed for the checks we want */ }
        }
    }
}

impl<'a, F> Visit<'a> for SpreadInReturnVisitor<'a, '_, F>
where
    F: FnMut(Spread<'a, '_>),
{
    #[inline]
    fn enter_node(&mut self, kind: AstKind<'a>) {
        if let AstKind::ReturnStatement(_) = kind {
            self.is_in_return = true;
        }
    }

    #[inline]
    fn leave_node(&mut self, kind: AstKind<'a>) {
        if let AstKind::ReturnStatement(_) = kind {
            self.is_in_return = false;
        }
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        if !self.is_in_return {
            return;
        }
        match expr.get_inner_expression() {
            // base cases
            Expression::ObjectExpression(obj) => self.visit_object_expression(obj),
            Expression::ArrayExpression(arr) => self.visit_array_expression(arr),
            // recursive cases
            Expression::ConditionalExpression(cond) => {
                self.visit_expression(&cond.consequent);
                self.visit_expression(&cond.alternate);
            }
            Expression::SequenceExpression(expr) => {
                if let Some(last) = expr.expressions.last() {
                    self.visit_expression(last);
                }
            }
            Expression::LogicalExpression(expr) => self.visit_logical_expression(expr),

            // check if identifier is a reference to a spread-initialized
            // variable declared within the map callback.
            Expression::Identifier(ident) => {
                let Some(symbol_id) = ident
                    .reference_id()
                    .map(|id| self.ctx.symbols().get_reference(id))
                    .and_then(Reference::symbol_id)
                else {
                    return;
                };
                let declaration_scope = self.ctx.symbols().get_scope_id(symbol_id);

                // symbol is not declared within the mapper callback
                if !self
                    .ctx
                    .scopes()
                    .ancestors(declaration_scope)
                    .any(|parent_id| parent_id == self.cb_scope_id)
                {
                    return;
                }

                // walk the declaration
                let declaration_node =
                    self.ctx.nodes().get_node(self.ctx.symbols().get_declaration(symbol_id));
                self.visit_kind(declaration_node.kind());
            }
            _ => {}
        }
    }

    fn visit_object_expression(&mut self, obj: &ObjectExpression<'a>) {
        if self.is_in_return
            && obj
                .properties
                .iter()
                .any(|prop| matches!(prop, ObjectPropertyKind::SpreadProperty(_)))
        {
            (self.cb)(Spread::Object(obj));
        }
    }

    fn visit_array_expression(&mut self, arr: &ArrayExpression<'a>) {
        if self.is_in_return
            && arr
                .elements
                .iter()
                .any(|prop| matches!(prop, ArrayExpressionElement::SpreadElement(_)))
        {
            (self.cb)(Spread::Array(arr));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    use serde_json::json;

    let pass = vec![
        ("let a = b.map(x => x)", None),
        ("let b = []; let a = b.map(x => ({ ...x })); console.log(b)", None),
        ("let a = b.map(x => Object.assign(x, { foo: { ...x.foo, bar }}))", None),
        (
            "class Foo {
                #arr = []
                id = 0
                public get arr() {
                    return this.#arr.map(x => ({ ...x, id: this.id }))
                }
            }",
            None,
        ),
        (
            // Spread does not occur within `map`
            "let extras = { ...foo, ...bar }; let a = b.map(x => extras)",
            None,
        ),
        ("function map(f) {}; map(x => ({ ...x }))", None),
    ];

    let fail = vec![
        ("let a = b.map(x => ({ ...x }))", None),
        ("let a = b.flatMap(x => ({ ...x }))", None),
        ("let a = b.map(x => ({ ...x, ...y }))", None),
        ("let b = []; let a = b.map(x => ({ ...x }))", None),
        ("let a = b.map(x => { return { ...x } })", None),
        ("let a = b.map(x => [ ...x ])", None),
        ("let a = b.map(x => [ ...x, ...y ])", None),
        ("let a = b.map(x => { return [ ...x ] })", None),
        ("let a = b.map(x => { let x2 = { ...x }; return x2; })", None),
        // rereads
        (
            "let a = b.map(x => ({ ...x })); console.log(b)",
            Some(json!([{ "ignoreRereads": false }])),
        ),
        ("let b = []; console.log(b); let a = b.map(x => ({ ...x }));", None),
        // conditionals and other "outer" expressions
        ("let a = b.map(x => someCond ? { ...x, foo: true } : { ...x, foo: false })", None),
        ("let a = b.map( ({ x, y }) => ({ ...(cond ? x : y) }) )", None),
        ("const b = a.map((x, i) => y ? { ...x, i } : x)", None),
        ("const b = a.map((x, i) => y ? x : { ...x, i })", None),
        ("let a = b.map(x => (0, { ...x }))", None),
        ("let a = b.map(({ x, y }) => (x ?? { ...y }))", None),
        ("let a = b.map((x => ({ ...x }))) as MyCustomMapper", None),
    ];

    Tester::new(NoMapSpread::NAME, pass, fail).test_and_snapshot();
}
