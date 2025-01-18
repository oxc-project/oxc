use std::ops::Deref;

use serde::{Deserialize, Serialize};

use oxc_ast::{
    ast::{
        ArrayExpression, ArrayExpressionElement, CallExpression, Expression, ObjectExpression,
        ObjectPropertyKind, ReturnStatement,
    },
    visit::walk,
    AstKind, Visit,
};
use oxc_diagnostics::{LabeledSpan, OxcDiagnostic};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{ReferenceId, ScopeId, SymbolId};
use oxc_span::{GetSpan, Span};

use crate::{
    ast_util::{is_method_call, leftmost_identifier_reference},
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
    utils::default_true,
    AstNode,
};

fn no_map_spread_diagnostic(
    map_call: Span,
    spread: &Spread<'_, '_>,
    returned_span: Option<Span>,
) -> OxcDiagnostic {
    let spans = spread.spread_spans();
    assert!(!spans.is_empty());
    let mut spread_labels = spread.spread_spans().into_iter();
    let first_message = if spans.len() == 1 {
        "It should be mutated in place"
    } else {
        "They should be mutated in place"
    };
    let first = spread_labels.next().unwrap().label(first_message);
    let others = spread_labels.map(LabeledSpan::from);

    let returned_label = returned_span
        .filter(|span| !span.contains_inclusive(spread.span()))
        .map(|span| span.label("Map returns the spread here"));

    let diagnostic =
        match spread {
            // Obj
            Spread::Object(_) => OxcDiagnostic::warn(
                "Spreading to modify object properties in `map` calls is inefficient",
            )
            .with_labels([map_call.label("This map call spreads an object"), first])
            .with_help("Consider using `Object.assign` instead"),
            // Array
            Spread::Array(_) => OxcDiagnostic::warn(
                "Spreading to modify array elements in `map` calls is inefficient",
            )
            .with_labels([map_call.label("This map call spreads an array"), first])
            .with_help("Consider using `Array.prototype.concat` or `Array.prototype.push` instead"),
        };

    diagnostic.and_labels(others).and_labels(returned_label)
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
    /// Ignore maps on arrays passed as parameters to a function.
    ///
    /// This option is enabled by default to better avoid false positives. It
    /// comes at the cost of potentially missing spreads that are inefficient.
    /// We recommend turning this off in your `.oxlintrc.json` files.
    ///
    /// ### Example
    ///
    /// Examples of **incorrect** code for this rule when `ignoreArgs` is `true`:
    /// ```ts
    /// /* "oxc/no-map-spread": ["error", { "ignoreArgs": true }] */
    /// function foo(arr) {
    ///     let arr2 = arr.filter(x => x.a > 0);
    ///     return arr2.map(x => ({ ...x }));
    /// }
    /// ```
    ///
    /// Examples of **correct** code for this rule when `ignoreArgs` is `true`:
    /// ```ts
    /// /* "oxc/no-map-spread": ["error", { "ignoreArgs": true }] */
    /// function foo(arr) {
    ///     return arr.map(x => ({ ...x }));
    /// }
    /// ```
    ///
    /// Default: `true`
    #[serde(default = "default_true")]
    ignore_args: bool,
    // todo: ignore_arrays?
}

// NOTE: not boxing the config for now because of how small it is. If we add
// more than 16 bytes of options, we need to add a box back.
#[derive(Debug, Default, Clone)]
pub struct NoMapSpread(NoMapSpreadConfig);
impl Deref for NoMapSpread {
    type Target = NoMapSpreadConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NoMapSpreadConfig> for NoMapSpread {
    fn from(config: NoMapSpreadConfig) -> Self {
        Self(config)
    }
}

impl Default for NoMapSpreadConfig {
    fn default() -> Self {
        Self { ignore_rereads: true, ignore_args: true }
    }
}

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Disallow the use of object or array spreads in
    /// [`Array.prototype.map`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/map)
    /// and
    /// [`Array.prototype.flatMap`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flatMap)
    /// to add properties/elements to array items.
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
    ///     public getAuthorsWithBooks() {
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
    /// ### Automatic Fixing
    /// This rule can automatically fix violations caused by object spreads, but
    /// does not fix arrays. Object spreads will get replaced with
    /// [`Object.assign`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Object/assign).  Array fixing may be added in the future.
    ///
    /// Object expressions with a single element (the spread) are not fixed.
    /// ```js
    /// arr.map(x => ({ ...x })) // not fixed
    /// ```
    ///
    /// A `fix` is available (using `--fix`) for objects with "normal" elements before the
    /// spread. Since `Object.apply` mutates the first argument, and a new
    /// object will be created with those elements, the spread identifier will
    /// not be mutated. In effect, the spread semantics are preserved
    /// ```js
    /// // before
    /// arr.map(({ x, y }) => ({ x, ...y }))
    ///
    /// // after
    /// arr.map(({ x, y }) => (Object.assign({ x }, y)))
    /// ```
    ///
    /// A suggestion (using `--fix-suggestions`) is provided when a spread is
    /// the first property in an object. This fix mutates the spread identifier,
    /// meaning it could have unintended side effects.
    /// ```js
    /// // before
    /// arr.map(({ x, y }) => ({ ...x, y }))
    /// arr.map(({ x, y }) => ({ ...x, y }))
    ///
    /// // after
    /// arr.map(({ x, y }) => (Object.assign(x, { y })))
    /// arr.map(({ x, y }) => (Object.assign(x, y)))
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
    oxc,
    nursery, // TODO: make this `perf` once we've battle-tested this a bit
    conditional_fix_suggestion
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
        let visitor = SpreadInReturnVisitor::<'a, '_>::iter_spreads(ctx, mapper, |spread| {
            // SAFETY: references to arena-allocated objects are valid for the
            // lifetime of the arena. Unfortunately, `AsRef` on `Box<'a, T>`
            // returns a reference with a lifetime of 'self instead of 'a.
            spreads.push(unsafe { std::mem::transmute::<Spread<'a, '_>, Spread<'a, 'a>>(spread) });
        });
        let returned_span = visitor.and_then(|v| v.return_span);
        if spreads.is_empty() {
            return;
        }

        match leftmost_identifier_reference(&call_expr.callee) {
            Ok(ident) => {
                let reference_id = ident.reference_id();
                if self.is_ignored_map_call(ctx, ident.name.as_str(), reference_id, call_expr.span)
                {
                    return;
                }
            }
            // Mapped class properties likely have their elements spread to
            // avoid side effects on the class instance.
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
            let diagnostic = no_map_spread_diagnostic(map_call_site, &spread, returned_span);
            if let Some(obj) = spread.as_object() {
                debug_assert!(!obj.properties.is_empty());
                if obj.properties.first().is_some_and(ObjectPropertyKind::is_spread) {
                    ctx.diagnostic_with_suggestion(diagnostic, |fixer| {
                        fix_spread_to_object_assign(fixer, obj)
                    });
                } else {
                    ctx.diagnostic_with_fix(diagnostic, |fixer| {
                        fix_spread_to_object_assign(fixer, obj)
                    });
                }
            } else {
                ctx.diagnostic(diagnostic);
            }
        }
    }
}

impl NoMapSpread {
    /// Spreads returned from `map` (et. al.) are not violations if
    /// 1. `map` is a custom user-defined function
    /// 2. the array being mapped is read after mapping and `ignore_rereads` is
    ///    enabled.
    fn is_ignored_map_call(
        &self,
        ctx: &LintContext<'_>,
        name: &str,
        reference_id: ReferenceId,
        call_site: Span,
    ) -> bool {
        let Some(symbol_id) = ctx.symbols().get_reference(reference_id).symbol_id() else {
            return false;
        };
        // Call is to a self-defined `map` call.
        if MAP_FN_NAMES.contains(&name) {
            // TODO: use symbolflags to check if function. Note that
            // SymbolFlags::ArrowFunction was removed.
            // TODO: ignore `const map = Array.prototype.map`
            return true;
        }

        if self.ignore_args {
            let declaration = ctx.nodes().get_node(ctx.symbols().get_declaration(symbol_id));
            if matches!(declaration.kind(), AstKind::FormalParameter(_)) {
                return true;
            }
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

fn fix_spread_to_object_assign<'a>(
    fixer: RuleFixer<'_, 'a>,
    obj: &ObjectExpression<'a>,
) -> RuleFix<'a> {
    use oxc_allocator::{Allocator, CloneIn};
    use oxc_ast::AstBuilder;
    use oxc_codegen::CodegenOptions;
    use oxc_span::SPAN;

    if obj.properties.len() <= 1 {
        return fixer.noop();
    }

    let alloc = Allocator::default();
    let ast = AstBuilder::new(&alloc);

    // almost always overshoots, but will not re-alloc, so it's more performant
    // than creating an empty vec.
    // let mut args = ast.vec_with_capacity::<Argument>(obj.properties.len());
    let mut curr_obj_properties = ast.vec::<ObjectPropertyKind>();
    let mut codegen =
        fixer.codegen().with_options(CodegenOptions { minify: true, ..Default::default() });
    let mut is_first = true;
    codegen.print_str("Object.assign(");

    for prop in &obj.properties {
        match prop {
            ObjectPropertyKind::ObjectProperty(_) => {
                curr_obj_properties.push(prop.clone_in(&alloc));
            }
            ObjectPropertyKind::SpreadProperty(spread) => {
                if !curr_obj_properties.is_empty() {
                    let properties = std::mem::replace(&mut curr_obj_properties, ast.vec());
                    let obj_arg = ast.expression_object(SPAN, properties, None);
                    if is_first {
                        is_first = false;
                    } else {
                        codegen.print_str(", ");
                    }
                    codegen.print_expression(&obj_arg);
                }
                if is_first {
                    is_first = false;
                } else {
                    codegen.print_str(", ");
                }
                codegen.print_expression(&spread.argument);
            }
        }
    }

    if !curr_obj_properties.is_empty() {
        if !is_first {
            codegen.print_str(", ");
        }
        let properties = std::mem::replace(&mut curr_obj_properties, ast.vec());
        let obj_arg = ast.expression_object(SPAN, properties, None);
        codegen.print_expression(&obj_arg);
    }
    codegen.print_ascii_byte(b')');

    // TODO: expand replaced span to outer paren parent to replace wrapped
    // parenthesis in implicit arrow returns. Blocked by AST node IDs not yet in
    // each node.
    fixer.replace(obj.span, codegen.into_source_text())
}

enum Spread<'a, 'b> {
    Object(&'b ObjectExpression<'a>),
    Array(&'b ArrayExpression<'a>),
}

impl<'a, 'b> Spread<'a, 'b> {
    #[inline]
    fn as_object(&self) -> Option<&'b ObjectExpression<'a>> {
        match self {
            Spread::Object(obj) => Some(obj),
            Spread::Array(_) => None,
        }
    }

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
    ctx: &'ctx LintContext<'a>,
    cb: F,
    cb_scope_id: ScopeId,
    is_in_return: bool,
    /// Span covering returned expression. [`None`] when not in a return
    /// statement or no value is being returned (e.g. `return;`, but not `return
    /// undefined;`).
    return_span: Option<Span>,
}

impl<'a, 'ctx, F> SpreadInReturnVisitor<'a, 'ctx, F>
where
    F: FnMut(Spread<'a, '_>),
{
    fn iter_spreads(ctx: &'ctx LintContext<'a>, map_cb: &Expression<'a>, cb: F) -> Option<Self> {
        let (mut visitor, body) = match map_cb {
            Expression::ArrowFunctionExpression(f) => {
                let v = Self {
                    ctx,
                    cb,
                    cb_scope_id: f.scope_id(),
                    is_in_return: f.expression,
                    return_span: f.expression.then(|| f.body.span()),
                };
                (v, f.body.as_ref())
            }
            Expression::FunctionExpression(f) => {
                let v = Self {
                    ctx,
                    cb,
                    cb_scope_id: f.scope_id(),
                    is_in_return: false,
                    return_span: None,
                };
                let body = f.body.as_ref().map(AsRef::as_ref)?;
                (v, body)
            }
            _ => unreachable!(),
        };

        visitor.visit_function_body(body);
        Some(visitor)
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
    fn visit_return_statement(&mut self, stmt: &ReturnStatement<'a>) {
        self.is_in_return = true;
        self.return_span = stmt.argument.as_ref().map(GetSpan::span);

        walk::walk_return_statement(self, stmt);

        self.is_in_return = false;
        // NOTE: do not clear `return_span` here. We want to keep the last
        // encountered `return` for reporting.
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
                let Some(symbol_id) =
                    self.ctx.symbols().get_reference(ident.reference_id()).symbol_id()
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
    use serde_json::json;

    use crate::tester::Tester;

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
        // class members are ignored
        ("this.foo.map(x => ({ ...x }))", None),
        ("this.#foo.map(x => ({ ...x }))", None),
        // indirection
        (
            // Spread does not occur within `map`
            "let extras = { ...foo, ...bar }; let a = b.map(x => extras)",
            None,
        ),
        // user-defined map functions are ignored
        ("function map(f) {}; map(x => ({ ...x }))", None),
        ("function flatMap(f) {}; flatMap(x => ({ ...x }))", None),
        // ignoreArgs
        ("function foo(a) { return a.map(x => ({ ...(x ?? y) })) }", None),
        ("const foo = a => a.map(x => ({ ...(x ?? y) }))", None),
    ];

    let fail = vec![
        // basic objects
        ("let a = b.map(x => ({ ...x }))", None),
        ("let a = b.map(x => ({ ...x, ...y }))", None),
        ("let b = []; let a = b.map(x => ({ ...x }))", None),
        ("let a = b.map(x => { return { ...x } })", None),
        // basic arrays
        ("let a = b.map(x => [ ...x ])", None),
        ("let a = b.map(x => [ ...x, ...y ])", None),
        ("let a = b.map(x => { return [ ...x ] })", None),
        // map (et. al.) variations
        ("a.map?.(x => ({ ...x }))", None),
        ("let a = b.flatMap(x => ({ ...x }))", None),
        // rereads
        (
            "let a = b.map(x => ({ ...x })); console.log(b)",
            Some(json!([{ "ignoreRereads": false }])),
        ),
        ("let b = []; console.log(b); let a = b.map(x => ({ ...x }));", None),
        // indirection
        ("let a = b.map(x => { let x2 = { ...x }; return x2; })", None),
        ("let a = b.map(x => { let x2 = { ...x }; let x3 = x2; return x3; })", None),
        (
            "let a = b.map(x => {
            let y = { ...x };
            if (y.foo) {
                return y;
            } else {
             return x;
            }
        })",
            None,
        ),
        // conditionals and other "outer" expressions
        ("let a = b.map(x => someCond ? { ...x, foo: true } : { ...x, foo: false })", None),
        ("let a = b.map( ({ x, y }) => ({ ...(cond ? x : y) }) )", None),
        ("const b = a.map((x, i) => y ? { ...x, i } : x)", None),
        ("const b = a.map((x, i) => y ? x : { ...x, i })", None),
        ("let a = b.map(x => (0, { ...x }))", None),
        ("let a = b.map(({ x, y }) => (x ?? { ...y }))", None),
        ("let a = b.map((x => ({ ...x }))) as MyCustomMapper", None),
        // make sure reported spans are nice
        // map call variations
        ("foo().map(x => ({ ...x }))", None),
        ("foo[1].map(x => ({ ...x }))", None),
        ("foo?.bar?.map(x => ({ ...x }))", None),
        ("(foo ?? bar).map(x => ({ ...x }))", None),
        ("obj.#foo.map(x => ({ ...x }))", None),
        // spread variations
        ("a.map(x => ({ ...x.y }))", None),
        ("a.map(x => ({ ...x[y] }))", None),
        ("a.map(x => ({ ...(x ?? y) }))", None),
        // ignoreArgs
        (
            "function foo(a) { return a.map(x => ({ ...(x ?? y) })) }",
            Some(json!([{ "ignoreArgs": false }])),
        ),
        ("const foo = a => a.map(x => ({ ...(x ?? y) }))", Some(json!([{ "ignoreArgs": false }]))),
    ];

    let fix = vec![
        // single spreads cannot be fixed with `Object.assign`. We'll assume the
        // spread is happening intentionally, to force a shallow clone. Maybe we
        // shouldn't even report these cases?
        ("let a = b.map(x => ({ ...x }))", "let a = b.map(x => ({ ...x }))"),
        // two
        (
            "let a = b.map(({ x, y }) => ({ ...x, ...y }))",
            "let a = b.map(({ x, y }) => (Object.assign(x, y)))",
        ),
        (
            "let a = b.map(({ x, y }) => { return { ...x, ...y }; })",
            "let a = b.map(({ x, y }) => { return Object.assign(x, y); })",
        ),
        (
            "let a = b.map(({ x, y }) => ({ x, ...y }))",
            "let a = b.map(({ x, y }) => (Object.assign({x}, y)))",
        ),
        (
            "let a = b.map(({ x, y }) => ({ ...x, y }))",
            "let a = b.map(({ x, y }) => (Object.assign(x, {y})))",
        ),
        // three
        (
            "let a = b.map(({ x, y, z }) => ({ ...x, y, z }))",
            "let a = b.map(({ x, y, z }) => (Object.assign(x, {y,z})))",
        ),
        (
            "let a = b.map(({ x, y, z }) => ({ x, ...y, z }))",
            "let a = b.map(({ x, y, z }) => (Object.assign({x}, y, {z})))",
        ),
        (
            "let a = b.map(({ x, y, z }) => ({ x, y, ...z }))",
            "let a = b.map(({ x, y, z }) => (Object.assign({x,y}, z)))",
        ),
    ];

    Tester::new(NoMapSpread::NAME, NoMapSpread::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
