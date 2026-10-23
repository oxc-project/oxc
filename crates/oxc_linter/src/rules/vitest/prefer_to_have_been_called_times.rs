use oxc_ast::AstKind;
use oxc_macros::declare_oxc_lint;
use oxc_semantic::AstNode;

use crate::{
    context::{ContextHost, LintContext},
    rule::Rule,
    rules::shared::prefer_to_have_been_called_times::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferToHaveBeenCalledTimes;

declare_oxc_lint!(
    PreferToHaveBeenCalledTimes,
    vitest,
    style,
    fix,
    docs = DOCUMENTATION,
    version = "1.34.0",
);

impl Rule for PreferToHaveBeenCalledTimes {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
        let AstKind::CallExpression(_) = node.kind() else { return };
        run(node, ctx);
    }

    fn should_run(&self, ctx: &ContextHost) -> bool {
        ctx.frameworks().is_test()
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "expect.assertions(1)",
        "expect(fn).toHaveBeenCalledTimes",
        "expect(fn.mock.calls).toHaveLength",
        "expect(fn.mock.values).toHaveLength(0)",
        "expect(fn.values.calls).toHaveLength(0)",
        "expect(fn).toHaveBeenCalledTimes(0)",
        "expect(fn).resolves.toHaveBeenCalledTimes(10)",
        "expect(fn).not.toHaveBeenCalledTimes(10)",
        "expect(fn).toHaveBeenCalledTimes(1)",
        "expect(fn).toBeCalledTimes(0);",
        "expect(fn).toHaveBeenCalledTimes(0);",
        "expect(fn);",
        "expect(method.mock.calls[0][0]).toStrictEqual(value);",
        "expect(fn.mock.length).toEqual(1);",
        "expect(fn.mock.calls).toEqual([]);",
        "expect(fn.mock.calls).toContain(1, 2, 3);",
    ];

    let fail = vec![
        "expect(method.mock.calls).toHaveLength(1);",
        "expect(method.mock.calls).resolves.toHaveLength(x);",
        r#"expect(method["mock"].calls).toHaveLength(0);"#,
        "expect(my.method.mock.calls).not.toHaveLength(0);",
    ];

    let fix = vec![
        (
            "expect(method.mock.calls).toHaveLength(1);",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).toHaveLength(
                1,
            );",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).toHaveLength(
                /* number of calls (one) */
                1,
            );",
            "expect(method).toHaveBeenCalledTimes(1);",
            None,
        ),
        (
            "expect(method.mock.calls).resolves.toHaveLength(x);",
            "expect(method).resolves.toHaveBeenCalledTimes(x);",
            None,
        ),
        (
            r#"expect(method["mock"].calls).toHaveLength(0);"#,
            "expect(method).toHaveBeenCalledTimes(0);",
            None,
        ),
        (
            "expect(my.method.mock.calls).not.toHaveLength(0);",
            "expect(my.method).not.toHaveBeenCalledTimes(0);",
            None,
        ),
    ];

    Tester::new(PreferToHaveBeenCalledTimes::NAME, PreferToHaveBeenCalledTimes::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
