use oxc_ast::{AstKind, ast::Expression};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;
use oxc_span::{Span, Atom};
use crate::{context::LintContext, rule::Rule};

const NON_CALLABLE_GLOBALS: [&str; 5] = [
    "Atomics",
    "Intl",
    "JSON",
    "Math",
    "Reflect"
];

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

fn is_global_obj<'a>(str: impl PartialEq<&'a str>) -> bool {
    NON_CALLABLE_GLOBALS
        .iter()
        .any(|&n| str == n)
}

impl Rule for NoObjCalls {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let (callee, span) = match node.kind() {
            AstKind::NewExpression(expr) => (&expr.callee, expr.span),
            AstKind::CallExpression(expr) => (&expr.callee, expr.span),
            _ => { return }
        };

        let ident: Atom = match callee {
            // handle new Math(), Math(), etc
            Expression::Identifier(ident) => {
                ident.name.clone()
            },
            // handle new globalThis.Math(), globalThis.Math(), etc
            Expression::MemberExpression(expr) => {
                // let is_static_member = expr.static_property_name()
                // if let MemberExpression::StaticMemberExpression(static_member) = expr.unbox() &&
                if let Expression::Identifier(static_ident) = expr.object() &&
                static_ident.name == "globalThis" &&
                let Some(static_member) = expr.static_property_name()
                {
                    static_member.into()
                } else {
                    return
                }
            }
            _ => { return }
        };

        if is_global_obj(ident.clone()) {
            ctx.diagnostic(NoObjCallsDiagnostic(ident, span));
        }
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
        ("function() { JSON.parse(Atomics()) }", None)
    ];

    Tester::new(NoObjCalls::NAME, pass, fail).test_and_snapshot();
}
