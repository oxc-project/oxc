use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Default, Clone)]
pub struct NoUselessCall;

declare_oxc_lint!(
    /// ### What it does
    ///
    ///
    /// ### Why is this bad?
    ///
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUselessCall,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
);

impl Rule for NoUselessCall {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "foo.apply(obj, 1, 2);",
        "obj.foo.apply(null, 1, 2);",
        "obj.foo.apply(otherObj, 1, 2);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, 1, 2);",
        "foo.apply(obj, [1, 2]);",
        "obj.foo.apply(null, [1, 2]);",
        "obj.foo.apply(otherObj, [1, 2]);",
        "a.b(x, y).c.foo.apply(a.b(x, z).c, [1, 2]);",
        "a.b.foo.apply(a.b.c, [1, 2]);",
        "foo.apply(null, args);",
        "obj.foo.apply(obj, args);",
        "var call; foo[call](null, 1, 2);",
        "var apply; foo[apply](null, [1, 2]);",
        "foo.call();",
        "obj.foo.call();",
        "foo.apply();",
        "obj.foo.apply();",
        "obj?.foo.bar.call(obj.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "class C { #call; wrap(foo) { foo.#call(undefined, 1, 2); } }", // { "ecmaVersion": 2022 }
    ];

    let fail = vec![
        "foo.call(undefined, 1, 2);",
        "foo.call(void 0, 1, 2);",
        "foo.call(null, 1, 2);",
        "obj.foo.call(obj, 1, 2);",
        "a.b.c.foo.call(a.b.c, 1, 2);",
        "a.b(x, y).c.foo.call(a.b(x, y).c, 1, 2);",
        "foo.apply(undefined, [1, 2]);",
        "foo.apply(void 0, [1, 2]);",
        "foo.apply(null, [1, 2]);",
        "obj.foo.apply(obj, [1, 2]);",
        "a.b.c.foo.apply(a.b.c, [1, 2]);",
        "a.b(x, y).c.foo.apply(a.b(x, y).c, [1, 2]);",
        "[].concat.apply([ ], [1, 2]);",
        "[].concat.apply([
			/*empty*/
			], [1, 2]);",
        r#"abc.get("foo", 0).concat.apply(abc . get("foo",  0 ), [1, 2]);"#,
        "foo.call?.(undefined, 1, 2);",  // { "ecmaVersion": 2020 },
        "foo?.call(undefined, 1, 2);",   // { "ecmaVersion": 2020 },
        "(foo?.call)(undefined, 1, 2);", // { "ecmaVersion": 2020 },
        "obj.foo.call?.(obj, 1, 2);",    // { "ecmaVersion": 2020 },
        "obj?.foo.call(obj, 1, 2);",     // { "ecmaVersion": 2020 },
        "(obj?.foo).call(obj, 1, 2);",   // { "ecmaVersion": 2020 },
        "(obj?.foo.call)(obj, 1, 2);",   // { "ecmaVersion": 2020 },
        "obj?.foo.bar.call(obj?.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "(obj?.foo).bar.call(obj?.foo, 1, 2);", // { "ecmaVersion": 2020 },
        "obj.foo?.bar.call(obj.foo, 1, 2);", // { "ecmaVersion": 2020 }
    ];

    Tester::new(NoUselessCall::NAME, pass, fail).test_and_snapshot();
}
