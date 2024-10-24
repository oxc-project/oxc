use oxc_ast::AstKind;
use oxc_diagnostics::OxcDiagnostic;
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{
    context::LintContext,
    rule::Rule,
    utils::{parse_expect_jest_fn_call, PossibleJestNode},
};

fn require_to_throw_message_diagnostic(x0: &str, span1: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn(format!("Require a message for {x0:?}."))
        .with_help(format!("Add an error message to {x0:?}"))
        .with_label(span1)
}

#[derive(Debug, Default, Clone)]
pub struct RequireToThrowMessage;

declare_oxc_lint!(
    /// ### What it does
    /// This rule triggers a warning if `toThrow()` or `toThrowError()` is used without an error message.
    ///
    /// ### Example
    /// ```javascript
    /// // invalid
    /// test('all the things', async () => {
    ///     expect(() => a()).toThrow();
    ///     expect(() => a()).toThrowError();
    ///     await expect(a()).rejects.toThrow();
    ///     await expect(a()).rejects.toThrowError();
    /// });
    ///
    /// // valid
    /// test('all the things', async () => {
    ///   expect(() => a()).toThrow('a');
    ///   expect(() => a()).toThrowError('a');
    ///   await expect(a()).rejects.toThrow('a');
    ///   await expect(a()).rejects.toThrowError('a');
    /// });
    /// ```
    ///
    RequireToThrowMessage,
    correctness
);

impl Rule for RequireToThrowMessage {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        Self::run(jest_node, ctx);
    }
}

impl RequireToThrowMessage {
    pub fn run<'a>(possible_jest_node: &PossibleJestNode<'a, '_>, ctx: &LintContext<'a>) {
        let node = possible_jest_node.node;
        let AstKind::CallExpression(call_expr) = node.kind() else {
            return;
        };

        let Some(jest_fn_call) = parse_expect_jest_fn_call(call_expr, possible_jest_node, ctx)
        else {
            return;
        };

        let Some(matcher) = jest_fn_call.matcher() else {
            return;
        };

        let Some(matcher_name) = matcher.name() else {
            return;
        };

        let has_not = jest_fn_call.modifiers().iter().any(|modifier| modifier.is_name_equal("not"));

        if jest_fn_call.args.len() == 0
            && (matcher_name == "toThrow" || matcher_name == "toThrowError")
            && !has_not
        {
            ctx.diagnostic(require_to_throw_message_diagnostic(&matcher_name, matcher.span));
        }
    }
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        // String
        ("expect(() => { throw new Error('a'); }).toThrow('a');", None),
        ("expect(() => { throw new Error('a'); }).toThrowError('a');", None),
        (
            "
                test('string', async () => {
                    const throwErrorAsync = async () => { throw new Error('a') };
                    await expect(throwErrorAsync()).rejects.toThrow('a');
                    await expect(throwErrorAsync()).rejects.toThrowError('a');
                })
            ",
            None,
        ),
        // Template literal
        ("const a = 'a'; expect(() => { throw new Error('a'); }).toThrow(`${a}`);", None),
        ("const a = 'a'; expect(() => { throw new Error('a'); }).toThrowError(`${a}`);", None),
        (
            "
                test('Template literal', async () => {
                    const a = 'a';
                    const throwErrorAsync = async () => { throw new Error('a') };
                    await expect(throwErrorAsync()).rejects.toThrow(`${a}`);
                    await expect(throwErrorAsync()).rejects.toThrowError(`${a}`);
                })
            ",
            None,
        ),
        // Regex
        ("expect(() => { throw new Error('a'); }).toThrow(/^a$/);", None),
        ("expect(() => { throw new Error('a'); }).toThrowError(/^a$/);", None),
        (
            "
                test('Regex', async () => {
                    const throwErrorAsync = async () => { throw new Error('a') };
                    await expect(throwErrorAsync()).rejects.toThrow(/^a$/);
                    await expect(throwErrorAsync()).rejects.toThrowError(/^a$/);
                })
            ",
            None,
        ),
        // Function
        ("expect(() => { throw new Error('a'); }).toThrow((() => { return 'a'; })());", None),
        ("expect(() => { throw new Error('a'); }).toThrowError((() => { return 'a'; })());", None),
        (
            "
                test('Function', async () => {
                    const throwErrorAsync = async () => { throw new Error('a') };
                    const fn = () => { return 'a'; };
                    await expect(throwErrorAsync()).rejects.toThrow(fn());
                    await expect(throwErrorAsync()).rejects.toThrowError(fn());
                })
            ",
            None,
        ),
        // Allow no message for `not`.
        ("expect(() => { throw new Error('a'); }).not.toThrow();", None),
        ("expect(() => { throw new Error('a'); }).not.toThrowError();", None),
        (
            "
                test('Allow no message for `not`', async () => {
                    const throwErrorAsync = async () => { throw new Error('a') };
                    await expect(throwErrorAsync()).resolves.not.toThrow();
                    await expect(throwErrorAsync()).resolves.not.toThrowError();
                })
            ",
            None,
        ),
        ("expect(a);", None),
    ];

    let fail = vec![
        // Empty toThrow
        ("expect(() => { throw new Error('a'); }).toThrow();", None),
        // Empty toThrowError
        ("expect(() => { throw new Error('a'); }).toThrowError();", None),
        // Empty rejects.toThrow / rejects.toThrowError
        (
            "
                test('empty rejects.toThrow', async () => {
                    const throwErrorAsync = async () => { throw new Error('a') };
                    await expect(throwErrorAsync()).rejects.toThrow();
                    await expect(throwErrorAsync()).rejects.toThrowError();
                })
            ",
            None,
        ),
    ];

    Tester::new(RequireToThrowMessage::NAME, pass, fail).with_jest_plugin(true).test_and_snapshot();
}
