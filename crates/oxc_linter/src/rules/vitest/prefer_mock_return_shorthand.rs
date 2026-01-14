use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    AstNode,
    context::LintContext,
    fixer::{RuleFix, RuleFixer},
    rule::Rule,
};

fn prefer_mock_return_shorthand_diagnostic(span: Span) -> OxcDiagnostic {
    // See <https://oxc.rs/docs/contribute/linter/adding-rules.html#diagnostics> for details
    OxcDiagnostic::warn("Should be an imperative statement about what is wrong.")
        .with_help("Should be a command-like statement that tells the user how to fix the issue.")
        .with_label(span)
}

#[derive(Debug, Default, Clone)]
pub struct PreferMockReturnShorthand;

// See <https://github.com/oxc-project/oxc/issues/6050> for documentation details.
declare_oxc_lint!(
    /// ### What it does
    ///
    /// Briefly describe the rule's purpose.
    ///
    /// ### Why is this bad?
    ///
    /// Explain why violating this rule is problematic.
    ///
    /// ### Examples
    ///
    /// Examples of **incorrect** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    ///
    /// Examples of **correct** code for this rule:
    /// ```js
    /// FIXME: Tests will fail if examples are missing or syntactically incorrect.
    /// ```
    PreferMockReturnShorthand,
    vitest,
    nursery, // TODO: change category to `correctness`, `suspicious`, `pedantic`, `perf`, `restriction`, or `style`
             // See <https://oxc.rs/docs/contribute/linter.html#rule-category> for details
    pending, // TODO: describe fix capabilities. Remove if no fix can be done,
             // keep at 'pending' if you think one could be added but don't know how.
             // Options are 'fix', 'fix_dangerous', 'suggestion', and 'conditional_fix_suggestion'
);

impl Rule for PreferMockReturnShorthand {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        "describe()",
        "it()",
        "describe.skip()",
        "it.skip()",
        "test()",
        "test.skip()",
        "var appliedOnly = describe.only; appliedOnly.apply(describe)",
        "var calledOnly = it.only; calledOnly.call(it)",
        "it.each()()",
        "it.each`table`()",
        "test.each()()",
        "test.each`table`()",
        "test.concurrent()",
        "vi.fn().mockReturnValue(42)",
        "vi.fn(() => Promise.resolve(42))",
        "vi.fn(() => 42)",
        "vi.fn(() => ({}))",
        "aVariable.mockImplementation",
        "aVariable.mockImplementation()",
        "jest.fn().mockImplementation(async () => 1);", // { "parserOptions": { "ecmaVersion": 2017 } },
        "jest.fn().mockImplementation(async function () {});", // { "parserOptions": { "ecmaVersion": 2017 } },
        "
                    jest.fn().mockImplementation(async function () {
                      return 42;
                    });
                  ", // { "parserOptions": { "ecmaVersion": 2017 } },
        "
                  aVariable.mockImplementation(() => {
                    if (true) {
                      return 1;
                    }
            
                    return 2;
                  });
                ",
        "aVariable.mockImplementation(() => value++)",
        "aVariable.mockImplementationOnce(() => --value)",
        "
                  const aValue = 0;
                  aVariable.mockImplementation(() => {
                    return aValue++;
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    aValue += 1;
            
                    return aValue;
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    aValue++;
            
                    return aValue;
                  });
                ",
        "aVariable.mockReturnValue()",
        "aVariable.mockReturnValue(1)",
        r#"aVariable.mockReturnValue("hello world")"#,
        "vi.spyOn(Thingy, 'method').mockImplementation(param => param * 2);",
        "vi.spyOn(Thingy, 'method').mockImplementation(param => true ? param : 0);",
        "
                  aVariable.mockImplementation(() => {
                    const value = new Date();
            
                    return Promise.resolve(value);
                  });
                ",
        "
                  aVariable.mockImplementation(() => {
                    throw new Error('oh noes!');
                  });
                ",
        "aVariable.mockImplementation(() => { /* do something */ });",
        "
                  aVariable.mockImplementation(() => {
                    const x = 1;
            
                    console.log(x + 2);
                  });
                ",
        "aVariable.mockReturnValue(Promise.all([1, 2, 3]));",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
            
                  // stuff happens
            
                  currentX++;
            
                  // more stuff happens
                ",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  let currentX = 0;
                  currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  var currentX = 0;
                  currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  var currentX = 0;
                  var currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => currentX);
                ",
        "
                  let doSomething = () => {};
            
                  jest.spyOn(X, getCount).mockImplementation(() => doSomething);
                ",
        "
                  let currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => {
                    currentX += 1;
            
                    return currentX;
                  });
                ",
        "
                  const currentX = 0;
                  jest.spyOn(X, getCount).mockImplementation(() => {
                    console.log('returning', currentX);
            
                    return currentX;
                  });
                ",
    ];

    let fail = vec![
        r#"vi.fn().mockImplementation(() => "hello sunshine")"#,
        "vi.fn().mockImplementation(() => ({}))",
        "vi.fn().mockImplementation(() => x)",
        "vi.fn().mockImplementation(() => true ? x : y)",
        r#"vi.fn().mockImplementation(() => "hello world")"#,
        r#"aVariable.mockImplementation(() => "hello world")"#,
        r#"vi.fn().mockImplementationOnce(() => "hello world")"#,
        r#"aVariable.mockImplementationOnce(() => "hello world")"#,
        "vi.fn().mockImplementation(() => [], xyz)",
        r#"vi.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
    ];

    let fix = vec![
        (
            r#"vi.fn().mockImplementation(() => "hello sunshine")"#,
            r#"vi.fn().mockReturnValue("hello sunshine")"#,
        ),
        ("vi.fn().mockImplementation(() => ({}))", "vi.fn().mockReturnValue({})"),
        ("vi.fn().mockImplementation(() => x)", "vi.fn().mockReturnValue(x)"),
        ("vi.fn().mockImplementation(() => true ? x : y)", "vi.fn().mockReturnValue(true ? x : y)"),
        (
            r#"vi.fn().mockImplementation(() => "hello world")"#,
            r#"vi.fn().mockReturnValue("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementation(() => "hello world")"#,
            r#"aVariable.mockReturnValue("hello world")"#,
        ),
        (
            r#"vi.fn().mockImplementationOnce(() => "hello world")"#,
            r#"vi.fn().mockReturnValueOnce("hello world")"#,
        ),
        (
            r#"aVariable.mockImplementationOnce(() => "hello world")"#,
            r#"aVariable.mockReturnValueOnce("hello world")"#,
        ),
        ("vi.fn().mockImplementation(() => [], xyz)", "vi.fn().mockReturnValue([], xyz)"),
        (
            r#"vi.spyOn(fs, "readFile").mockImplementation(() => new Error("oh noes!"))"#,
            r#"vi.spyOn(fs, "readFile").mockReturnValue(new Error("oh noes!"))"#,
        ),
    ];

    Tester::new(PreferMockReturnShorthand::NAME, PreferMockReturnShorthand::PLUGIN, pass, fail)
        .expect_fix(fix)
        .test_and_snapshot();
}
