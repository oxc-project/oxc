use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_mock_promise_shorthand::{DOCUMENTATION, run},
};

#[derive(Debug, Default, Clone)]
pub struct PreferMockPromiseShorthand;

declare_oxc_lint!(
    PreferMockPromiseShorthand,
    vitest,
    style,
    conditional_fix,
    docs = DOCUMENTATION,
    version = "0.2.16",
);

impl Rule for PreferMockPromiseShorthand {
    fn run<'a>(&self, node: &oxc_semantic::AstNode<'a>, ctx: &LintContext<'a>) {
        run(node, ctx);
    }
}

#[test]
fn test() {
    use crate::{rule::RuleMeta, tester::Tester};

    let pass_jest = vec![
        ("describe()", None),
        ("it()", None),
        ("describe.skip()", None),
        ("it.skip()", None),
        ("test()", None),
        ("test.skip()", None),
        ("var appliedOnly = describe.only; appliedOnly.apply(describe)", None),
        ("var calledOnly = it.only; calledOnly.call(it)", None),
        ("it.each()()", None),
        ("it.each`table`()", None),
        ("test.each()()", None),
        ("test.each`table`()", None),
        ("test.concurrent()", None),
        ("jest.fn().mockResolvedValue(42)", None),
        ("jest.fn(() => Promise.resolve(42))", None),
        ("jest.fn(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation", None),
        ("aVariable.mockImplementation()", None),
        ("aVariable.mockImplementation([])", None),
        ("aVariable.mockImplementation(() => {})", None),
        ("aVariable.mockImplementation(() => [])", None),
        ("aVariable.mockReturnValue(() => Promise.resolve(1))", None),
        ("aVariable.mockReturnValue(Promise.resolve(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject().then(() => 1))", None),
        ("aVariable.mockReturnValue(new Promise(resolve => resolve(1)))", None),
        ("aVariable.mockReturnValue(new Promise((_, reject) => reject(1)))", None),
        ("jest.spyOn(Thingy, 'method').mockImplementation(param => Promise.resolve(param));", None),
        (
            "
                aVariable.mockImplementation(() => {
                    const value = new Date();
                    return Promise.resolve(value);
                });
            ",
            None,
        ),
        (
            "
                aVariable.mockImplementation(() => {
                    return Promise.resolve(value)
                        .then(value => value + 1);
                });
            ",
            None,
        ),
        (
            "
                aVariable.mockImplementation(() => {
                    return Promise.all([1, 2, 3]);
                });
            ",
            None,
        ),
        ("aVariable.mockImplementation(() => Promise.all([1, 2, 3]));", None),
        ("aVariable.mockReturnValue(Promise.all([1, 2, 3]));", None),
    ];

    let fail_jest = vec![
        ("aVariable.mockImplementation(() => Promise.reject(42, 25))", None),
        ("jest.fn().mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementation(() => { return Promise.resolve(42); })", None),
        ("aVariable.mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.reject(42),)", None),
        ("aVariable.mockImplementationOnce(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementationOnce(() => Promise.reject(42))", None),
        ("jest.fn().mockReturnValue(Promise.resolve(42))", None),
        ("jest.fn().mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.reject(42))", None),
        (
            "
                aVariable.mockReturnValue(Promise.resolve({
                    target: 'world',
                    message: 'hello'
                }))
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockImplementation(() => Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValue(Promise.reject(42))
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockReturnValueOnce(Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValueOnce(Promise.reject(42))
            ",
            None,
        ),
        (
            "
                aVariable.mockReturnValueOnce(
                    Promise.reject(
                        new Error('oh noes!')
                    )
                )
            ",
            None,
        ),
        ("jest.fn().mockReturnValue(Promise.resolve(42), xyz)", None),
        ("jest.fn().mockImplementation(() => Promise.reject(42), xyz)", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42, xyz))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve())", None),
        (
            "jest.spyOn(fs, \"readFile\").mockReturnValue(Promise.reject(new Error(\"oh noes!\")))",
            None,
        ),
    ];

    let fix_jest = vec![
        (
            "jest.fn().mockImplementation(() => Promise.resolve(42))",
            "jest.fn().mockResolvedValue(42)",
            None,
        ),
        (
            "jest.fn().mockImplementation(() => Promise.reject(42))",
            "jest.fn().mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.resolve(42))",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => {
                return Promise.resolve(42);
            });",
            "aVariable.mockResolvedValue(42);",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42))",
            "aVariable.mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42),)",
            "aVariable.mockRejectedValue(42,)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        ("jest.fn().mockReturnValue(Promise.resolve(42))", "jest.fn().mockResolvedValue(42)", None),
        ("jest.fn().mockReturnValue(Promise.reject(42))", "jest.fn().mockRejectedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", "aVariable.mockResolvedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", "aVariable.mockRejectedValue(42)", None),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))",
        //     "aVariable.mockResolvedValue({ target: 'world', message: 'hello' })",
        //     None,
        // ),
        (
            "
                aVariable
                    .mockImplementation(() => Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValue(Promise.reject(42))
            ",
            "
                aVariable
                    .mockRejectedValue(42)
                    .mockResolvedValue(42)
                    .mockRejectedValue(42)
            ",
            None,
        ),
        (
            "
                aVariable
                    .mockReturnValueOnce(Promise.reject(42))
                    .mockImplementation(() => Promise.resolve(42))
                    .mockReturnValueOnce(Promise.reject(42))
            ",
            "
                aVariable
                    .mockRejectedValueOnce(42)
                    .mockResolvedValue(42)
                    .mockRejectedValueOnce(42)
            ",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))",
            "aVariable.mockRejectedValueOnce(new Error('oh noes!'))",
            None,
        ),
        (
            "jest.fn().mockReturnValue(Promise.resolve(42), xyz)",
            "jest.fn().mockResolvedValue(42, xyz)",
            None,
        ),
        (
            "jest.fn().mockImplementation(() => Promise.reject(42), xyz)",
            "jest.fn().mockRejectedValue(42, xyz)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve())",
            "aVariable.mockResolvedValueOnce(undefined)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "jest.spyOn(fs, \"readFile\").mockReturnValue(Promise.reject(new Error(\"oh noes!\")))",
        //     "jest.spyOn(fs, \"readFile\").mockRejectedValue(new Error(\"oh noes!\"))",
        //     None,
        // ),
    ];

    let mut pass = vec![
        ("describe()", None),
        ("it()", None),
        ("describe.skip()", None),
        ("it.skip()", None),
        ("test()", None),
        ("test.skip()", None),
        ("var appliedOnly = describe.only; appliedOnly.apply(describe)", None),
        ("var calledOnly = it.only; calledOnly.call(it)", None),
        ("it.each()()", None),
        ("it.each`table`()", None),
        ("test.each()()", None),
        ("test.each`table`()", None),
        ("test.concurrent()", None),
        ("vi.fn().mockResolvedValue(42)", None),
        ("vi.fn(() => Promise.resolve(42))", None),
        ("vi.fn(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation", None),
        ("aVariable.mockImplementation()", None),
        ("aVariable.mockImplementation([])", None),
        ("aVariable.mockImplementation(() => {})", None),
        ("aVariable.mockImplementation(() => [])", None),
        ("aVariable.mockReturnValue(() => Promise.resolve(1))", None),
        ("aVariable.mockReturnValue(Promise.resolve(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject(1).then(() => 1))", None),
        ("aVariable.mockReturnValue(Promise.reject().then(() => 1))", None),
        ("aVariable.mockReturnValue(new Promise(resolve => resolve(1)))", None),
        ("aVariable.mockReturnValue(new Promise((_, reject) => reject(1)))", None),
        ("vi.spyOn(Thingy, 'method').mockImplementation(param => Promise.resolve(param));", None),
    ];
    pass.extend(pass_jest);

    let mut fail = vec![
        ("vi.fn().mockImplementation(() => Promise.resolve(42))", None),
        ("vi.fn().mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementation(() => { return Promise.resolve(42) })", None),
        ("aVariable.mockImplementation(() => Promise.reject(42))", None),
        ("aVariable.mockImplementation(() => Promise.reject(42),)", None),
        ("aVariable.mockImplementationOnce(() => Promise.resolve(42))", None),
        ("aVariable.mockImplementationOnce(() => Promise.reject(42))", None),
        ("vi.fn().mockReturnValue(Promise.resolve(42))", None),
        ("vi.fn().mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42))", None),
        ("aVariable.mockReturnValueOnce(Promise.reject(42))", None),
        ("aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))", None),
        (
            "aVariable.mockImplementation(() => Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValue(Promise.reject(42))",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValueOnce(Promise.reject(42))",
            None,
        ),
        ("aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))", None),
        ("vi.fn().mockReturnValue(Promise.resolve(42), xyz)", None),
        ("vi.fn().mockImplementation(() => Promise.reject(42), xyz)", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve(42, xyz))", None),
        ("aVariable.mockReturnValueOnce(Promise.resolve())", None),
    ];
    fail.extend(fail_jest);

    let mut fix = vec![
        (
            "vi.fn().mockImplementation(() => Promise.resolve(42))",
            "vi.fn().mockResolvedValue(42)",
            None,
        ),
        (
            "vi.fn().mockImplementation(() => Promise.reject(42))",
            "vi.fn().mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.resolve(42))",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => { return Promise.resolve(42) })",
            "aVariable.mockResolvedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42))",
            "aVariable.mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42),)",
            "aVariable.mockRejectedValue(42,)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockImplementationOnce(() => Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        ("vi.fn().mockReturnValue(Promise.resolve(42))", "vi.fn().mockResolvedValue(42)", None),
        ("vi.fn().mockReturnValue(Promise.reject(42))", "vi.fn().mockRejectedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.resolve(42))", "aVariable.mockResolvedValue(42)", None),
        ("aVariable.mockReturnValue(Promise.reject(42))", "aVariable.mockRejectedValue(42)", None),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve(42))",
            "aVariable.mockResolvedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42)",
            None,
        ),
        // Todo: Fixed
        // (
        //     "aVariable.mockReturnValue(Promise.resolve({ target: 'world', message: 'hello' }))",
        //     "aVariable.mockResolvedValue({ target: 'world', message: 'hello' })",
        //     None,
        // ),
        (
            "aVariable.mockImplementation(() => Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValue(Promise.reject(42))",
            "aVariable.mockRejectedValue(42).mockResolvedValue(42).mockRejectedValue(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(42)).mockImplementation(() => Promise.resolve(42)).mockReturnValueOnce(Promise.reject(42))",
            "aVariable.mockRejectedValueOnce(42).mockResolvedValue(42).mockRejectedValueOnce(42)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.reject(new Error('oh noes!')))",
            "aVariable.mockRejectedValueOnce(new Error('oh noes!'))",
            None,
        ),
        (
            "vi.fn().mockReturnValue(Promise.resolve(42), xyz)",
            "vi.fn().mockResolvedValue(42, xyz)",
            None,
        ),
        (
            "vi.fn().mockImplementation(() => Promise.reject(42), xyz)",
            "vi.fn().mockRejectedValue(42, xyz)",
            None,
        ),
        (
            "aVariable.mockReturnValueOnce(Promise.resolve())",
            "aVariable.mockResolvedValueOnce(undefined)",
            None,
        ),
    ];
    fix.extend(fix_jest);

    Tester::new(PreferMockPromiseShorthand::NAME, PreferMockPromiseShorthand::PLUGIN, pass, fail)
        .with_vitest_plugin(true)
        .expect_fix(fix)
        .test_and_snapshot();
}
