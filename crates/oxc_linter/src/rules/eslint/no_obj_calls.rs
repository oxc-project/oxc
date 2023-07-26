use oxc_ast::{
    ast::{Expression, IdentifierReference, MemberExpression},
    AstKind,
};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::{AstNode, ScopeId};
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule};

const GLOBAL_THIS: &str = "globalThis";
const NON_CALLABLE_GLOBALS: [&str; 5] = ["Atomics", "Intl", "JSON", "Math", "Reflect"];

#[derive(Debug, Error, Diagnostic)]
#[error("eslint(no-obj-calls): Disallow calling some global objects as functions")]
#[diagnostic(severity(error), help("{0} is not a function."))]
struct NoObjCallsDiagnostic(Atom, #[label] pub Span);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoObjCalls;

impl Default for NoObjCalls {
    fn default() -> Self {
        Self
    }
}

declare_oxc_lint! {
    /// ### What it does
    /// Disallow calling some global objects as functions
    ///
    /// ### Why is this bad?
    /// Some global objects are not intended to be called as functions.
    /// Calling them as functions will usually result in a TypeError being thrown.
    ///
    /// ### Example
    /// ```javascript
    /// // Bad
    /// let math = Math();
    /// let newMath = new Math();
    ///
    /// let json = JSON();
    /// let newJson = new JSON();
    ///
    /// let atomics = Atomics();
    /// let newAtomics = new Atomics();
    ///
    /// let intl = Intl();
    /// let newIntl = new Intl();
    ///
    /// let reflect = Reflect();
    /// let newReflect = new Reflect();
    ///
    /// // Good
    /// let area = r => 2 * Math.PI * r * r;
    /// let object = JSON.parse("{}");
    /// let first = Atomics.load(sharedArray, 0);
    /// let segmenterFrom = Intl.Segmenter("fr", { granularity: "word" });
    /// ```
    NoObjCalls,
    correctness,
}

fn is_global_obj(str: &Atom) -> bool {
    NON_CALLABLE_GLOBALS.iter().any(|&n| str == &n)
}

fn global_this_member(expr: &oxc_allocator::Box<'_, MemberExpression<'_>>) -> Option<Atom> {
    if let Expression::Identifier(static_ident) = expr.object() {
        if static_ident.name == GLOBAL_THIS {
            if let Some(static_member) = expr.static_property_name() {
                Some(static_member.into())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    }
}

fn resolve_global_binding<'a, 'b: 'a>(
    ident: &oxc_allocator::Box<'a, IdentifierReference>,
    scope_id: ScopeId,
    ctx: &LintContext<'a>,
) -> Option<Atom> {
    if ctx.semantic().is_reference_to_global_variable(ident) {
        Some(ident.name.clone())
    } else {
        let scope = ctx.scopes();
        let nodes = ctx.nodes();
        let symbols = ctx.symbols();
        scope.get_binding(scope_id, &ident.name).map_or_else(
            || {
                // panic!("No binding id found for {}, but this IdentifierReference
                // is not a global", &ident.name);
                None
            },
            |binding_id| {
                let decl = nodes.get_node(symbols.get_declaration(binding_id));
                let decl_scope = decl.scope_id();
                match decl.kind() {
                    AstKind::VariableDeclarator(parent_decl) => {
                        match &parent_decl.init {
                            // handles "let a = JSON; let b = a; a();"
                            Some(Expression::Identifier(parent_ident)) => {
                                resolve_global_binding(parent_ident, decl_scope, ctx)
                            }
                            // handles "let a = globalThis.JSON; let b = a; a();"
                            Some(Expression::MemberExpression(parent_expr)) => {
                                global_this_member(parent_expr)
                            }
                            _ => None,
                        }
                    }
                    _ => None,
                }
            },
        )
    }
}

impl Rule for NoObjCalls {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        #[allow(clippy::needless_return)]
        let (callee, span) = match node.kind() {
            AstKind::NewExpression(expr) => (&expr.callee, expr.span),
            AstKind::CallExpression(expr) => (&expr.callee, expr.span),
            _ => return,
        };

        match callee {
            Expression::Identifier(ident) => {
                // handle new Math(), Math(), etc
                if let Some(top_level_reference) =
                    resolve_global_binding(ident, node.scope_id(), ctx)
                {
                    if is_global_obj(&top_level_reference) {
                        ctx.diagnostic(NoObjCallsDiagnostic(ident.name.clone(), span));
                    }
                }
            }

            Expression::MemberExpression(expr) => {
                // handle new globalThis.Math(), globalThis.Math(), etc
                if let Some(global_member) = global_this_member(expr) {
                    if is_global_obj(&global_member) {
                        ctx.diagnostic(NoObjCallsDiagnostic(global_member, span));
                    }
                }
            }
            _ => {
                // noop
            }
        };
    }
}

#[test]
fn test() {
    use crate::tester::Tester;
    // see: https://github.com/eslint/eslint/blob/main/tests/lib/rules/no-obj-calls.js

    let pass = vec![
        ("const m = Math;", None),
        ("let m = foo.Math();", None),
        ("JSON.parse(\"{}\")", None),
        ("Math.PI * 2 * (r * r)", None),
        ("bar.Atomic(foo)", None),
        // reference test cases
        (
            "let j = JSON;
            function foo() {
                let j = x => x;
                return x();
            }",
            None,
        ),
    ];

    let fail = vec![
        ("let newObj = new JSON();", None),
        ("let obj = JSON();", None),
        ("let obj = globalThis.JSON()", None),
        ("new JSON", None),
        ("const foo = x => new JSON()", None),
        ("let newObj = new Math();", None),
        ("let obj = Math();", None),
        ("let obj = new Math().foo;", None),
        ("let obj = new globalThis.Math()", None),
        ("let newObj = new Atomics();", None),
        ("let obj = Atomics();", None),
        ("let newObj = new Intl();", None),
        ("let obj = Intl();", None),
        ("let newObj = new Reflect();", None),
        ("let obj = Reflect();", None),
        ("function() { JSON.parse(Atomics()) }", None),
        // reference test cases
        ("let j = JSON; j();", None),
        ("let a = JSON; let b = a; let c = b; b();", None),
        ("let m = globalThis.Math; new m();", None),
    ];

    Tester::new(NoObjCalls::NAME, pass, fail).test_and_snapshot();
}
