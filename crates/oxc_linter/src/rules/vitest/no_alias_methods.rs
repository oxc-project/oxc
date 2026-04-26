use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_alias_methods::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoAliasMethods;

declare_oxc_lint!(NoAliasMethods, vitest, style, fix, docs = DOCUMENTATION, version = "0.0.12",);

impl Rule for NoAliasMethods {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run_on_jest_node(jest_node, ctx);
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let mut pass = vec![
        ("expect(a).toHaveBeenCalled()", None),
        ("expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).toHaveBeenCalledWith()", None),
        ("expect(a).toHaveBeenLastCalledWith()", None),
        ("expect(a).toHaveBeenNthCalledWith()", None),
        ("expect(a).toHaveReturned()", None),
        ("expect(a).toHaveReturnedTimes()", None),
        ("expect(a).toHaveReturnedWith()", None),
        ("expect(a).toHaveLastReturnedWith()", None),
        ("expect(a).toHaveNthReturnedWith()", None),
        ("expect(a).toThrow()", None),
        ("expect(a).rejects;", None),
        ("expect(a);", None),
    ];

    let mut fail = vec![
        ("expect(a).toBeCalled()", None),
        ("expect(a).toBeCalledTimes()", None),
        ("expect(a).toBeCalledWith()", None),
        ("expect(a).lastCalledWith()", None),
        ("expect(a).nthCalledWith()", None),
        ("expect(a).toReturn()", None),
        ("expect(a).toReturnTimes()", None),
        ("expect(a).toReturnWith()", None),
        ("expect(a).lastReturnedWith()", None),
        ("expect(a).nthReturnedWith()", None),
        ("expect(a).toThrowError()", None),
        ("expect(a).resolves.toThrowError()", None),
        ("expect(a).rejects.toThrowError()", None),
        ("expect(a).not.toThrowError()", None),
        ("expect(a).not['toThrowError']()", None),
    ];

    let mut fix = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
        ("expect(a).not[`toThrowError`]()", "expect(a).not[`toThrow`]()", None),
    ];

    let pass_vitest = vec![
        "expect(a).toHaveBeenCalled()",
        "expect(a).toHaveBeenCalledTimes()",
        "expect(a).toHaveBeenCalledWith()",
        "expect(a).toHaveBeenLastCalledWith()",
        "expect(a).toHaveBeenNthCalledWith()",
        "expect(a).toHaveReturned()",
        "expect(a).toHaveReturnedTimes()",
        "expect(a).toHaveReturnedWith()",
        "expect(a).toHaveLastReturnedWith()",
        "expect(a).toHaveNthReturnedWith()",
        "expect(a).toThrow()",
        "expect(a).rejects;",
        "expect(a);",
    ];

    let fail_vitest = vec![
        "expect(a).toBeCalled()",
        "expect(a).toBeCalledTimes()",
        r#"expect(a).not["toThrowError"]()"#,
    ];

    let fix_vitest = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).toBeCalledTimes()", "expect(a).toHaveBeenCalledTimes()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
    ];

    pass.extend(pass_vitest.into_iter().map(|x| (x, None)));
    fail.extend(fail_vitest.into_iter().map(|x| (x, None)));
    fix.extend(fix_vitest);

    Tester::new(NoAliasMethods::NAME, NoAliasMethods::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
