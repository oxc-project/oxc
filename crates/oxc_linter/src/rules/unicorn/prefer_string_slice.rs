use oxc_ast::{ast::MemberExpression, AstKind};
use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::{self, Error},
};
use oxc_macros::declare_oxc_lint;
use oxc_span::{Atom, Span};

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-unicorn(prefer-string-slice): Prefer String#slice() over String#{1}()")]
#[diagnostic(severity(warning))]
struct PreferStringSliceDiagnostic(#[label] pub Span, Atom);

#[derive(Debug, Default, Clone)]
pub struct PreferStringSlice;

declare_oxc_lint!(
    /// ### What it does
    ///
    /// Prefer [`String#slice()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice) over [`String#substr()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr) and [`String#substring()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring).
    ///
    /// ### Why is this bad?
    ///
    /// [`String#substr()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substr) and [`String#substring()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/substring) are the two lesser known legacy ways to slice a string. It's better to use [`String#slice()`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/slice) as it's a more popular option with clearer behavior that has a consistent [`Array` counterpart](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/slice).
    ///
    /// ### Example
    /// ```javascript
    /// ```
    PreferStringSlice,
    pedantic
);

impl Rule for PreferStringSlice {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(member_expr) = call_expr.callee.get_member_expr() else { return };

        let (span, name) = match member_expr {
            MemberExpression::StaticMemberExpression(v) => {
                if !matches!(v.property.name.as_str(), "substr" | "substring") {
                    return;
                }
                (v.property.span, &v.property.name)
            }
            _ => return,
        };

        ctx.diagnostic(PreferStringSliceDiagnostic(span, name.clone()));
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        r"const substr = foo.substr",
        r"const substring = foo.substring",
        r"foo.slice()",
        r"foo.slice(0)",
        r"foo.slice(1, 2)",
        r"foo?.slice(1, 2)",
        r"foo?.slice?.(1, 2)",
        r"foo?.bar.baz.slice(1, 2)",
        r"foo.slice(-3, -2)",
    ];

    let fail = vec![
        r"foo.substr()",
        r"foo?.substr()",
        r"foo.bar?.substring()",
        r"foo?.[0]?.substring()",
        r"foo.bar.substr?.()",
        r"foo.bar?.substring?.()",
        r"foo.bar?.baz?.substr()",
        r"foo.bar?.baz.substring()",
        r"foo.bar.baz?.substr()",
        r#""foo".substr()"#,
        r#""foo".substr(1)"#,
        r#""foo".substr(1, 2)"#,
        r#""foo".substr(bar.length, Math.min(baz, 100))"#,
        r#""foo".substr(1, length)"#,
        r#""foo".substr(1, "abc".length)"#,
        r#""foo".substr("1", 2)"#,
        r#""foo".substr(0, -1)"#,
        r#""foo".substr(0, "foo".length)"#,
        r#""foo".substr(1, length)"#,
        r"foo.substr(start)",
        r#""foo".substr(1)"#,
        r"foo.substr(start, length)",
        r#""foo".substr(1, 2)"#,
        r"foo.substr(1, 2, 3)",
        r#""Sample".substr(0, "Sample".lastIndexOf("/"))"#,
        r"foo.substring()",
        r#""foo".substring()"#,
        r#""foo".substring(1)"#,
        r#""foo".substring(1, 2)"#,
        r#""foo".substring(2, 1)"#,
        r#""foo".substring(-1, -5)"#,
        r#""foo".substring(-1, 2)"#,
        r#""foo".substring(length)"#,
        r#""foobar".substring("foo".length)"#,
        r#""foo".substring(0, length)"#,
        r#""foo".substring(length, 0)"#,
        r"foo.substring(start)",
        r#""foo".substring(1)"#,
        r"foo.substring(start, end)",
        r#""foo".substring(1, 3)"#,
        r"foo.substring(1, 2, 3)",
        r"foo.substr(0, ...bar)",
        r"foo.substr(...bar)",
        r"foo.substr(0, (100, 1))",
        r"foo.substr(0, 1, extraArgument)",
        r"foo.substr((0, bar.length), (0, baz.length))",
        r"foo.substring((10, 1), 0)",
        r"foo.substring(0, (10, 1))",
        r"foo.substring(0, await 1)",
        r"foo.substring((10, bar))",
    ];

    Tester::new_without_config(PreferStringSlice::NAME, pass, fail).test_and_snapshot();
}
