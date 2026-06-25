use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::require_to_throw_message::{DOCUMENTATION, run_on_jest_node},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct RequireToThrowMessage;

declare_oxc_lint!(
    RequireToThrowMessage,
    jest,
    correctness,
    docs = DOCUMENTATION,
    version = "0.2.9",
);

impl Rule for RequireToThrowMessage {
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

    Tester::new(RequireToThrowMessage::NAME, RequireToThrowMessage::PLUGIN, pass, fail)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
