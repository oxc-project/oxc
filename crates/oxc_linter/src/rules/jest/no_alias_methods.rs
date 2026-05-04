use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::no_alias_methods::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct NoAliasMethods;

declare_oxc_lint!(NoAliasMethods, jest, style, fix, docs = DOCUMENTATION, version = "0.0.12",);

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

    let pass = vec![
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

    let fail = vec![
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

    let fix = vec![
        ("expect(a).toBeCalled()", "expect(a).toHaveBeenCalled()", None),
        ("expect(a).not['toThrowError']()", "expect(a).not['toThrow']()", None),
        ("expect(a).not[`toThrowError`]()", "expect(a).not[`toThrow`]()", None),
    ];

    Tester::new(NoAliasMethods::NAME, NoAliasMethods::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
