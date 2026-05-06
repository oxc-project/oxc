use oxc_macros::declare_oxc_lint;

use crate::{
    context::LintContext,
    rule::Rule,
    rules::shared::prefer_to_contain::{DOCUMENTATION, run},
    utils::PossibleJestNode,
};

#[derive(Debug, Default, Clone)]
pub struct PreferToContain;

declare_oxc_lint!(PreferToContain, jest, style, fix, docs = DOCUMENTATION, version = "0.2.14",);

impl Rule for PreferToContain {
    fn run_on_jest_node<'a, 'c>(
        &self,
        jest_node: &PossibleJestNode<'a, 'c>,
        ctx: &'c LintContext<'a>,
    ) {
        run(jest_node, ctx);
    }
}

#[test]
fn tests() {
    use crate::tester::Tester;

    #[expect(clippy::literal_string_with_formatting_args)]
    let pass = vec![
        ("expect.hasAssertions", None),
        ("expect.hasAssertions()", None),
        ("expect.assertions(1)", None),
        ("expect().toBe(false);", None),
        ("expect(a).toContain(b);", None),
        ("expect(a.name).toBe('b');", None),
        ("expect(a).toBe(true);", None),
        ("expect(a).toEqual(b)", None),
        ("expect(a.test(c)).toEqual(b)", None),
        ("expect(a.includes(b)).toEqual()", None),
        ("expect(a.includes(b)).toEqual(\"test\")", None),
        ("expect(a.includes(b)).toBe(\"test\")", None),
        ("expect(a.includes()).toEqual()", None),
        ("expect(a.includes()).toEqual(true)", None),
        ("expect(a.includes(b,c)).toBe(true)", None),
        ("expect([{a:1}]).toContain({a:1})", None),
        ("expect([1].includes(1)).toEqual", None),
        ("expect([1].includes).toEqual", None),
        ("expect([1].includes).not", None),
        ("expect(a.test(b)).resolves.toEqual(true)", None),
        ("expect(a.test(b)).resolves.not.toEqual(true)", None),
        ("expect(a).not.toContain(b)", None),
        ("expect(a.includes(...[])).toBe(true)", None),
        ("expect(a.includes(b)).toBe(...true)", None),
        ("expect(a);", None),
        // typescript
        (
            "(expect('Model must be bound to an array if the multiple property is true') as any).toHaveBeenTipped()",
            None,
        ),
        ("expect(a.includes(b)).toEqual(0 as boolean);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let fail = vec![
        ("expect(a.includes(b)).toEqual(true);", None),
        ("expect(a.includes(b,),).toEqual(true,)", None),
        ("expect(a['includes'](b)).toEqual(true);", None),
        ("expect(a['includes'](b))['toEqual'](true);", None),
        ("expect(a['includes'](b)).toEqual(false);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", None),
        ("expect(a.includes(b)).toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(false);", None),
        ("expect(a.includes(b)).not.toEqual(true);", None),
        ("expect(a.includes(b)).toBe(true);", None),
        ("expect(a.includes(b)).toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(false);", None),
        ("expect(a.includes(b)).not.toBe(true);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).toEqual(false);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(true);", None),
        ("expect(a.test(t).includes(b.test(p))).not.toEqual(false);", None),
        ("expect([{a:1}].includes({b:1})).toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toBe(false);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).toStrictEqual(false);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(true);", None),
        ("expect([{a:1}].includes({a:1})).not.toStrictEqual(false);", None),
        (
            "
                import { expect as pleaseExpect } from '@jest/globals';
                pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);
            ",
            None,
        ),
        // typescript
        ("expect(a.includes(b)).toEqual(false as boolean);", None),
    ];

    #[expect(clippy::literal_string_with_formatting_args)]
    let fix = vec![
        ("expect(a.includes(b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b,),).toEqual(true,)", "expect(a).toContain(b)", None),
        ("expect(a['includes'](b)).toEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['toEqual'](true);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a['includes'](b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not'].toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a['includes'](b))['not']['toEqual'](false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toEqual(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toBe(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toBe(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toBe(true);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(true);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).toStrictEqual(false);", "expect(a).not.toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(false);", "expect(a).toContain(b);", None),
        ("expect(a.includes(b)).not.toStrictEqual(true);", "expect(a).not.toContain(b);", None),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(true);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).toEqual(false);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(true);",
            "expect(a.test(t)).not.toContain(b.test(p));",
            None,
        ),
        (
            "expect(a.test(t).includes(b.test(p))).not.toEqual(false);",
            "expect(a.test(t)).toContain(b.test(p));",
            None,
        ),
        // Diff with eslint: The default print_expression add a space between key and value, and before and after curly braces, values
        (
            "expect([{a:1}].includes({a:1})).toBe(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toBe(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toBe(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(true);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).toStrictEqual(false);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(true);",
            "expect([{ a: 1 }]).not.toContain({ a: 1 });",
            None,
        ),
        (
            "expect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "expect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        (
            "import { expect as pleaseExpect } from '@jest/globals';
			pleaseExpect([{a:1}].includes({a:1})).not.toStrictEqual(false);",
            "import { expect as pleaseExpect } from '@jest/globals';
			pleaseExpect([{ a: 1 }]).toContain({ a: 1 });",
            None,
        ),
        ("expect(a.includes(b)).toEqual(false as boolean);", "expect(a).not.toContain(b);", None),
    ];

    Tester::new(PreferToContain::NAME, PreferToContain::PLUGIN, pass, fail)
        .expect_fix(fix)
        .with_jest_plugin(true)
        .test_and_snapshot();
}
