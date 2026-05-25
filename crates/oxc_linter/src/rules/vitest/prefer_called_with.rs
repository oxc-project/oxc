use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_called_with::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferCalledWith;

declare_oxc_lint!(PreferCalledWith, vitest, style, fix, docs = DOCUMENTATION, version = "0.2.5",);

impl Rule for PreferCalledWith {
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
        ("expect(fn).toBeCalledWith();", None),
        ("expect(fn).toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledWith(expect.anything());", None),
        ("expect(fn).toHaveBeenCalledWith(expect.anything());", None),
        ("expect(fn).not.toBeCalled();", None),
        ("expect(fn).rejects.not.toBeCalled();", None),
        ("expect(fn).not.toHaveBeenCalled();", None),
        ("expect(fn).not.toBeCalledWith();", None),
        ("expect(fn).not.toHaveBeenCalledWith();", None),
        ("expect(fn).resolves.not.toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledTimes(0);", None),
        ("expect(fn).toHaveBeenCalledTimes(0);", None),
        ("expect(fn);", None),
    ];

    let mut fail = vec![
        ("expect(fn).toBeCalled();", None),
        ("expect(fn).resolves.toBeCalled();", None),
        ("expect(fn).toHaveBeenCalled();", None),
    ];

    let mut fix = vec![
        ("expect(fn).toBeCalled();", "expect(fn).toBeCalledWith();", None),
        ("expect(fn).resolves.toBeCalled();", "expect(fn).resolves.toBeCalledWith();", None),
        ("expect(fn).toHaveBeenCalled();", "expect(fn).toHaveBeenCalledWith();", None),
    ];

    let vitest_pass = vec![
        ("expect(fn).toBeCalledWith();", None),
        ("expect(fn).toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledWith(expect.anything());", None),
        ("expect(fn).toHaveBeenCalledWith(expect.anything());", None),
        ("expect(fn).not.toBeCalled();", None),
        ("expect(fn).rejects.not.toBeCalled();", None),
        ("expect(fn).not.toHaveBeenCalled();", None),
        ("expect(fn).not.toBeCalledWith();", None),
        ("expect(fn).not.toHaveBeenCalledWith();", None),
        ("expect(fn).resolves.not.toHaveBeenCalledWith();", None),
        ("expect(fn).toBeCalledTimes(0);", None),
        ("expect(fn).toHaveBeenCalledTimes(0);", None),
        ("expect(fn);", None),
    ];

    let vitest_fail = vec![
        ("expect(fn).toBeCalled();", None),
        ("expect(fn).resolves.toBeCalled();", None),
        ("expect(fn).toHaveBeenCalled();", None),
    ];

    let vitest_fix = vec![
        ("expect(fn).toBeCalled();", "expect(fn).toBeCalledWith();", None),
        ("expect(fn).resolves.toBeCalled();", "expect(fn).resolves.toBeCalledWith();", None),
        ("expect(fn).toHaveBeenCalled();", "expect(fn).toHaveBeenCalledWith();", None),
    ];

    pass.extend(vitest_pass);
    fail.extend(vitest_fail);
    fix.extend(vitest_fix);

    Tester::new(PreferCalledWith::NAME, PreferCalledWith::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_vitest_plugin(true)
        .test_and_snapshot();
}
